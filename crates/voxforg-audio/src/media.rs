use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};
use voxforg_core::error::VoxForgError;

/// High-performance media extraction and muxing processor for video dubbing workflows.
pub struct MediaProcessor;

impl MediaProcessor {
    /// Detect the path to the ffmpeg executable.
    pub fn ffmpeg_path() -> Option<PathBuf> {
        if let Ok(path) = std::env::var("VOXFORG_FFMPEG_PATH") {
            let p = PathBuf::from(path);
            if p.exists() {
                return Some(p);
            }
        }

        // Check system path
        let cmd = if cfg!(windows) { "where.exe" } else { "which" };
        if let Ok(output) = Command::new(cmd).arg("ffmpeg").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().next() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        return Some(PathBuf::from(trimmed));
                    }
                }
            }
        }

        None
    }

    /// Check whether FFmpeg is available on the host system.
    pub fn is_available() -> bool {
        Self::ffmpeg_path().is_some()
    }

    /// Extract audio stream from a video file into a standard 16-bit mono WAV container.
    pub fn extract_audio(
        video_path: impl AsRef<Path>,
        out_wav_path: impl AsRef<Path>,
        sample_rate: u32,
    ) -> Result<(), VoxForgError> {
        let v_path = video_path.as_ref();
        let out_path = out_wav_path.as_ref();

        if !v_path.exists() {
            return Err(VoxForgError::AudioProcessing(format!(
                "Source video file not found: {}",
                v_path.display()
            )));
        }

        let ffmpeg = Self::ffmpeg_path().ok_or_else(|| {
            VoxForgError::AudioProcessing(
                "FFmpeg is required for video audio extraction but was not found in PATH or VOXFORG_FFMPEG_PATH".to_string(),
            )
        })?;

        let sample_rate_str = sample_rate.to_string();
        let args = [
            "-y",
            "-i",
            v_path.to_str().unwrap_or_default(),
            "-vn",
            "-acodec",
            "pcm_s16le",
            "-ar",
            &sample_rate_str,
            "-ac",
            "1",
            out_path.to_str().unwrap_or_default(),
        ];

        info!(
            video = %v_path.display(),
            output = %out_path.display(),
            "Extracting audio stream using FFmpeg"
        );

        let output = Command::new(&ffmpeg).args(args).output().map_err(|e| {
            VoxForgError::AudioProcessing(format!("Failed to spawn FFmpeg process: {e}"))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(stderr = %stderr, "FFmpeg audio extraction failed");
            return Err(VoxForgError::AudioProcessing(format!(
                "FFmpeg extraction failed: {stderr}"
            )));
        }

        Ok(())
    }

    /// Mux synthesized master audio into an existing video stream without re-encoding video.
    pub fn mux_video(
        video_path: impl AsRef<Path>,
        audio_path: impl AsRef<Path>,
        out_video_path: impl AsRef<Path>,
    ) -> Result<(), VoxForgError> {
        let v_path = video_path.as_ref();
        let a_path = audio_path.as_ref();
        let out_path = out_video_path.as_ref();

        if !v_path.exists() {
            return Err(VoxForgError::AudioProcessing(format!(
                "Source video file not found: {}",
                v_path.display()
            )));
        }
        if !a_path.exists() {
            return Err(VoxForgError::AudioProcessing(format!(
                "Master audio file not found: {}",
                a_path.display()
            )));
        }

        let ffmpeg = Self::ffmpeg_path().ok_or_else(|| {
            VoxForgError::AudioProcessing(
                "FFmpeg is required for video muxing but was not found in PATH or VOXFORG_FFMPEG_PATH".to_string(),
            )
        })?;

        let args = [
            "-y",
            "-i",
            v_path.to_str().unwrap_or_default(),
            "-i",
            a_path.to_str().unwrap_or_default(),
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-shortest",
            out_path.to_str().unwrap_or_default(),
        ];

        info!(
            video = %v_path.display(),
            audio = %a_path.display(),
            output = %out_path.display(),
            "Muxing master audio into video container using FFmpeg"
        );

        let output = Command::new(&ffmpeg).args(args).output().map_err(|e| {
            VoxForgError::AudioProcessing(format!("Failed to spawn FFmpeg process: {e}"))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(stderr = %stderr, "FFmpeg video muxing failed");
            return Err(VoxForgError::AudioProcessing(format!(
                "FFmpeg muxing failed: {stderr}"
            )));
        }

        Ok(())
    }

    /// Extract audio from video directly into raw PCM16 samples and sample rate.
    pub fn extract_audio_to_pcm(
        video_path: impl AsRef<Path>,
        sample_rate: u32,
    ) -> Result<(Vec<i16>, u32, u16), VoxForgError> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp_wav = std::env::temp_dir().join(format!("voxforg_demux_{}_{nanos}.wav", std::process::id()));

        Self::extract_audio(video_path, &temp_wav, sample_rate)?;
        let wav_bytes = std::fs::read(&temp_wav).map_err(|e| {
            let _ = std::fs::remove_file(&temp_wav);
            VoxForgError::AudioProcessing(format!("Failed reading extracted audio temp file: {e}"))
        })?;
        let _ = std::fs::remove_file(&temp_wav);

        crate::wav::WavEncoder::decode_wav_to_pcm16(&wav_bytes)
    }

    /// Mux raw WAV bytes into video container using FFmpeg.
    pub fn mux_video_bytes(
        video_path: impl AsRef<Path>,
        wav_bytes: &[u8],
        out_video_path: impl AsRef<Path>,
    ) -> Result<(), VoxForgError> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp_wav = std::env::temp_dir().join(format!("voxforg_mux_{}_{nanos}.wav", std::process::id()));

        std::fs::write(&temp_wav, wav_bytes).map_err(|e| {
            VoxForgError::AudioProcessing(format!("Failed writing temp audio for video mux: {e}"))
        })?;

        let res = Self::mux_video(video_path, &temp_wav, out_video_path);
        let _ = std::fs::remove_file(&temp_wav);
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_processor_missing_file_error() {
        let res = MediaProcessor::extract_audio(
            Path::new("non_existent_file.mp4"),
            Path::new("target/out.wav"),
            24000,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_media_processor_availability_probe() {
        let _avail = MediaProcessor::is_available();
    }
}
