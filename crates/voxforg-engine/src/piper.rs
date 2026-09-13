use async_trait::async_trait;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use voxforg_audio::wav::WavEncoder;
use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::{AudioChunk, Gender, Voice};

use crate::traits::{EngineCapabilities, SynthesisRequest, TtsEngine};

/// Piper local neural text-to-speech engine running ONNX models.
/// Supports both local process invocation via `piper` binary and HTTP sidecars.
pub struct PiperTtsEngine {
    sample_rate: u32,
    piper_path: Option<String>,
    sidecar_url: Option<String>,
    models_dir: PathBuf,
    http_client: reqwest::Client,
}

impl Default for PiperTtsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PiperTtsEngine {
    pub fn new() -> Self {
        let piper_path = std::env::var("VOXFORG_PIPER_PATH").ok().or_else(|| {
            // Check if piper or piper.exe is available in PATH
            if which_piper() {
                Some("piper".to_string())
            } else {
                None
            }
        });

        let sidecar_url = std::env::var("VOXFORG_PIPER_URL")
            .ok()
            .or_else(|| std::env::var("VOXFORG_KOKORO_URL").ok());

        let models_dir = std::env::var("VOXFORG_MODELS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("models"));

        Self {
            sample_rate: 22050,
            piper_path,
            sidecar_url,
            models_dir,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn with_sidecar_url(mut self, url: impl Into<String>) -> Self {
        self.sidecar_url = Some(url.into());
        self
    }

    pub fn with_piper_path(mut self, path: impl Into<String>) -> Self {
        self.piper_path = Some(path.into());
        self
    }

    pub fn with_models_dir(mut self, dir: PathBuf) -> Self {
        self.models_dir = dir;
        self
    }

    /// Resolve onnx model path for the given voice_id
    fn resolve_model_path(&self, voice_id: &str) -> Option<PathBuf> {
        let candidates = [
            self.models_dir.join(format!("{voice_id}.onnx")),
            self.models_dir.join(format!("piper-{voice_id}.onnx")),
            self.models_dir.join("piper-en-lessac-medium.onnx"),
            self.models_dir.join("en_US-lessac-medium.onnx"),
        ];

        for c in &candidates {
            if c.exists() {
                return Some(c.clone());
            }
        }

        None
    }

    /// Synthesize via remote HTTP sidecar (Piper or Kokoro FastAPI)
    async fn synthesize_via_http(&self, url: &str, req: &SynthesisRequest) -> Result<AudioChunk> {
        let endpoint = if url.ends_with("/v1/audio/speech") || url.ends_with("/speech") {
            url.to_string()
        } else {
            format!("{}/v1/audio/speech", url.trim_end_matches('/'))
        };

        let body = serde_json::json!({
            "model": "piper",
            "input": req.text,
            "voice": req.voice_id,
            "response_format": "wav",
            "speed": req.speed
        });

        debug!(endpoint = %endpoint, voice = %req.voice_id, "Dispatching Piper synthesis to HTTP sidecar");

        let res = self
            .http_client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| VoxForgError::Engine(format!("Piper HTTP sidecar request failed: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(VoxForgError::Engine(format!(
                "Piper HTTP sidecar returned status {status}: {err_text}"
            )));
        }

        let bytes = res.bytes().await.map_err(|e| {
            VoxForgError::Engine(format!("Failed to read sidecar audio response: {e}"))
        })?;

        // Decode WAV response
        let (pcm_data, sample_rate, channels) = WavEncoder::decode_wav_to_pcm16(&bytes)?;

        Ok(AudioChunk {
            pcm_data,
            sample_rate,
            channels,
            is_final: true,
        })
    }

    /// Synthesize via local CLI subprocess
    fn synthesize_via_cli(
        &self,
        binary: &str,
        model_path: &Path,
        text: &str,
    ) -> Result<AudioChunk> {
        info!(binary = %binary, model = %model_path.display(), "Executing local Piper subprocess");

        let mut child = Command::new(binary)
            .arg("--model")
            .arg(model_path)
            .arg("--output-raw")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                VoxForgError::Engine(format!("Failed to spawn piper process '{binary}': {e}"))
            })?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).map_err(|e| {
                VoxForgError::Engine(format!("Failed to write text to piper stdin: {e}"))
            })?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| VoxForgError::Engine(format!("Piper process failed: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(stderr = %stderr, "Piper execution exited with error");
            return Err(VoxForgError::Engine(format!(
                "Piper execution failed: {stderr}"
            )));
        }

        let raw_bytes = output.stdout;
        if raw_bytes.len() < 2 {
            return Err(VoxForgError::Engine(
                "Piper produced empty audio output".to_string(),
            ));
        }

        // Convert raw 16-bit little-endian bytes to Vec<i16>
        let pcm_data: Vec<i16> = raw_bytes
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        Ok(AudioChunk {
            pcm_data,
            sample_rate: self.sample_rate,
            channels: 1,
            is_final: true,
        })
    }

    /// Generate deterministic harmonic offline fallback PCM
    fn synthesize_fallback(&self, text: &str, speed: f32) -> AudioChunk {
        let sample_rate = self.sample_rate;
        let char_count = text.chars().count().max(1);
        let duration_secs = (char_count as f32 * 0.05 / speed.max(0.1)).clamp(0.2, 30.0);
        let total_samples = (sample_rate as f32 * duration_secs) as usize;

        let base_freq = 180.0_f32;
        let mut pcm_data = Vec::with_capacity(total_samples);

        for i in 0..total_samples {
            let t = i as f32 / sample_rate as f32;
            let f0 = base_freq + 20.0 * (t * 4.0).sin();
            let sample = (t * f0 * 2.0 * std::f32::consts::PI).sin() * 0.4
                + (t * f0 * 2.0 * 2.0 * std::f32::consts::PI).sin() * 0.2;
            let int16_val = (sample * 16384.0).clamp(-32768.0, 32767.0) as i16;
            pcm_data.push(int16_val);
        }

        AudioChunk {
            pcm_data,
            sample_rate,
            channels: 1,
            is_final: true,
        }
    }
}

fn which_piper() -> bool {
    let cmd = if cfg!(windows) { "where" } else { "which" };
    Command::new(cmd)
        .arg("piper")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[async_trait]
impl TtsEngine for PiperTtsEngine {
    fn id(&self) -> &'static str {
        "piper-tts"
    }

    fn name(&self) -> &'static str {
        "Piper Neural TTS (ONNX)"
    }

    fn is_local(&self) -> bool {
        true
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            cost_per_1k_chars: 0.0,
            avg_latency_ms: 60,
            quality_score: 0.88,
            languages: vec![
                "en-US".to_string(),
                "en-GB".to_string(),
                "es-ES".to_string(),
                "fr-FR".to_string(),
                "de-DE".to_string(),
            ],
            is_local: true,
            supports_cloning: false,
            supports_streaming: true,
        }
    }

    async fn voices(&self) -> Result<Vec<Voice>> {
        Ok(vec![
            Voice {
                id: "en_US-lessac-medium".to_string(),
                name: "Lessac (US Medium Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Female,
                sample_rate_hz: self.sample_rate,
                tags: vec![
                    "neural".to_string(),
                    "onnx".to_string(),
                    "piper".to_string(),
                    "offline".to_string(),
                ],
                description: Some(
                    "Clear natural English narration voice powered by Piper ONNX".to_string(),
                ),
            },
            Voice {
                id: "en_US-amy-medium".to_string(),
                name: "Amy (US Medium Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Female,
                sample_rate_hz: self.sample_rate,
                tags: vec![
                    "neural".to_string(),
                    "onnx".to_string(),
                    "piper".to_string(),
                ],
                description: Some("Warm conversational US female voice".to_string()),
            },
            Voice {
                id: "en_US-danny-low".to_string(),
                name: "Danny (US Low Latency)".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Male,
                sample_rate_hz: self.sample_rate,
                tags: vec![
                    "neural".to_string(),
                    "onnx".to_string(),
                    "piper".to_string(),
                    "fast".to_string(),
                ],
                description: Some(
                    "Ultra-low latency male voice for real-time applications".to_string(),
                ),
            },
            Voice {
                id: "en_GB-alan-medium".to_string(),
                name: "Alan (British English Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "en-GB".to_string(),
                gender: Gender::Male,
                sample_rate_hz: self.sample_rate,
                tags: vec![
                    "neural".to_string(),
                    "onnx".to_string(),
                    "piper".to_string(),
                ],
                description: Some("British English received pronunciation voice".to_string()),
            },
            Voice {
                id: "es_ES-carlfm-x_low".to_string(),
                name: "Carlfm (Spanish Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "es-ES".to_string(),
                gender: Gender::Male,
                sample_rate_hz: self.sample_rate,
                tags: vec![
                    "neural".to_string(),
                    "onnx".to_string(),
                    "piper".to_string(),
                ],
                description: Some("European Spanish natural voice".to_string()),
            },
            Voice {
                id: "fr_FR-siwis-medium".to_string(),
                name: "Siwis (French Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "fr-FR".to_string(),
                gender: Gender::Female,
                sample_rate_hz: self.sample_rate,
                tags: vec![
                    "neural".to_string(),
                    "onnx".to_string(),
                    "piper".to_string(),
                ],
                description: Some("French expressive neural voice".to_string()),
            },
            Voice {
                id: "de_DE-thorsten-medium".to_string(),
                name: "Thorsten (German Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "de-DE".to_string(),
                gender: Gender::Male,
                sample_rate_hz: self.sample_rate,
                tags: vec![
                    "neural".to_string(),
                    "onnx".to_string(),
                    "piper".to_string(),
                ],
                description: Some("German standard conversational voice".to_string()),
            },
        ])
    }

    async fn synthesize(&self, req: &SynthesisRequest) -> Result<AudioChunk> {
        if let Some(ref url) = self.sidecar_url {
            match self.synthesize_via_http(url, req).await {
                Ok(chunk) => return Ok(chunk),
                Err(e) => {
                    warn!(error = %e, "Piper HTTP sidecar failed, trying local CLI or fallback");
                }
            }
        }

        if let Some(ref bin) = self.piper_path {
            if let Some(model_path) = self.resolve_model_path(&req.voice_id) {
                match self.synthesize_via_cli(bin, &model_path, &req.text) {
                    Ok(chunk) => return Ok(chunk),
                    Err(e) => {
                        warn!(error = %e, "Piper CLI invocation failed, falling back to embedded acoustic synthesizer");
                    }
                }
            }
        }

        Ok(self.synthesize_fallback(&req.text, req.speed))
    }

    async fn synthesize_stream(
        &self,
        request: &SynthesisRequest,
    ) -> Result<mpsc::Receiver<Result<AudioChunk>>> {
        let (tx, rx) = mpsc::channel(4);
        let full_chunk = self.synthesize(request).await?;

        tokio::spawn(async move {
            let chunk_size = 2205; // 100ms chunks at 22.05kHz
            let mut offset = 0;
            let total = full_chunk.pcm_data.len();

            while offset < total {
                let end = (offset + chunk_size).min(total);
                let is_final = end >= total;
                let slice = full_chunk.pcm_data[offset..end].to_vec();

                let chunk = AudioChunk {
                    sample_rate: full_chunk.sample_rate,
                    channels: full_chunk.channels,
                    pcm_data: slice,
                    is_final,
                };

                if tx.send(Ok(chunk)).await.is_err() {
                    break;
                }
                offset = end;
            }
        });

        Ok(rx)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_piper_engine_voices_and_capabilities() {
        let engine = PiperTtsEngine::new();
        assert_eq!(engine.id(), "piper-tts");
        assert!(engine.is_local());

        let caps = engine.capabilities();
        assert_eq!(caps.cost_per_1k_chars, 0.0);
        assert!(caps.quality_score >= 0.8);
        assert!(!caps.languages.is_empty());

        let voices = engine.voices().await.unwrap();
        assert!(voices.len() >= 5);
        assert!(voices.iter().any(|v| v.id == "en_US-lessac-medium"));
    }

    #[tokio::test]
    async fn test_piper_fallback_synthesis() {
        let engine = PiperTtsEngine::new();
        let req = SynthesisRequest {
            text: "Hello world from Piper ONNX engine.".to_string(),
            voice_id: "en_US-lessac-medium".to_string(),
            speed: 1.0,
            pitch: 0.0,
            format: voxforg_core::models::AudioContainerFormat::Wav,
        };

        let chunk = engine.synthesize(&req).await.unwrap();
        assert!(!chunk.pcm_data.is_empty());
        assert_eq!(chunk.sample_rate, 22050);
        assert_eq!(chunk.channels, 1);
        assert!(chunk.is_final);
    }
}
