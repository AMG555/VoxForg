//! Speech-to-Speech (Voice Conversion / Neural Voice Changer) endpoints.
//!
//! | Method | Path                         | Description                             |
//! |--------|------------------------------|-----------------------------------------|
//! | POST   | `/v1/audio/speech-to-speech` | Convert source voice audio into target voice |
//! | POST   | `/v1/audio/voice-conversion` | Alias for speech-to-speech              |

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use serde::Deserialize;
use std::io::Cursor;
use voxforg_asr::TranscriptionOptions;
use voxforg_audio::WavEncoder;
use voxforg_core::error::ProblemDetails;
use voxforg_core::models::AudioContainerFormat;
use voxforg_engine::SynthesisRequest;

use crate::state::AppState;

#[derive(Debug, Clone, Deserialize)]
pub struct SpeechToSpeechRequest {
    /// Base64-encoded audio (WAV, WebM, MP3).
    pub audio_base64: String,
    /// Target voice profile or voice ID to convert into.
    pub target_voice: String,
    /// Synthesis speed factor (0.5 - 2.5). Default: 1.0.
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Whether to adjust tempo to match the source audio spoken duration.
    #[serde(default)]
    pub preserve_tempo: bool,
    /// Output container format.
    #[serde(default)]
    pub response_format: AudioContainerFormat,
}

pub const MAX_AUDIO_PAYLOAD_CHARS: usize = 32 * 1024 * 1024;

fn default_speed() -> f32 {
    1.0
}

fn decode_b64(input: &str) -> Option<Vec<u8>> {
    let mut table = [0xFFu8; 256];
    for (i, &b) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        .iter()
        .enumerate()
    {
        table[b as usize] = i as u8;
    }
    table[b'-' as usize] = 62;
    table[b'_' as usize] = 63;

    let filtered: Vec<u8> = input
        .bytes()
        .filter(|&b| !b.is_ascii_whitespace())
        .collect();
    if filtered.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(filtered.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0;

    for &b in &filtered {
        if b == b'=' {
            break;
        }
        let val = table[b as usize];
        if val == 0xFF {
            continue;
        }
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

#[allow(clippy::chunks_exact_to_as_chunks)]
fn decode_audio(payload: &str) -> Result<(Vec<i16>, u32), String> {
    let clean = if let Some(idx) = payload.find(',') {
        payload[idx + 1..].trim()
    } else {
        payload.trim()
    };

    if clean.is_empty() {
        return Err("Audio payload is empty".to_string());
    }

    let bytes = decode_b64(clean)
        .or_else(|| hex::decode(clean).ok())
        .ok_or_else(|| {
            "Invalid audio payload: expected valid base64 or hex encoding".to_string()
        })?;

    if bytes.is_empty() {
        return Err("Audio payload is empty".to_string());
    }

    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        let cursor = Cursor::new(&bytes);
        match hound::WavReader::new(cursor) {
            Ok(mut reader) => {
                let spec = reader.spec();
                let sample_rate = spec.sample_rate.clamp(8000, 192000);
                let samples: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
                match samples {
                    Ok(s) => {
                        if spec.channels == 2 {
                            let mono: Vec<i16> = s
                                .chunks_exact(2)
                                .map(|ch| ((ch[0] as i32 + ch[1] as i32) / 2) as i16)
                                .collect();
                            return Ok((mono, sample_rate));
                        } else if spec.channels == 1 {
                            return Ok((s, sample_rate));
                        } else if spec.channels > 2 {
                            let ch = spec.channels as usize;
                            let mono: Vec<i16> = s
                                .chunks_exact(ch)
                                .map(|frame| {
                                    let sum: i32 = frame.iter().map(|&x| x as i32).sum();
                                    (sum / ch as i32) as i16
                                })
                                .collect();
                            return Ok((mono, sample_rate));
                        } else {
                            return Err("Invalid WAV: 0 channels reported".to_string());
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to read 16-bit PCM samples from WAV: {}", e));
                    }
                }
            }
            Err(e) => {
                return Err(format!("Malformed WAV container: {}", e));
            }
        }
    }

    // Raw 16-bit PCM fallback
    if bytes.len() % 2 != 0 {
        return Err("Raw PCM payload byte length must be an even number (16-bit)".to_string());
    }
    let samples = bytes
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    Ok((samples, 16000))
}

/// Convert spoken audio into target voice.
pub async fn speech_to_speech(
    State(state): State<AppState>,
    Json(payload): Json<SpeechToSpeechRequest>,
) -> Result<Response, (StatusCode, Json<ProblemDetails>)> {
    if payload.audio_base64.trim().is_empty() {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/empty-payload".to_string(),
            title: "Missing Audio Payload".to_string(),
            status: StatusCode::BAD_REQUEST.as_u16(),
            detail: "The 'audio_base64' parameter must not be empty".to_string(),
            instance: "/v1/audio/speech-to-speech".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(err)));
    }

    if payload.audio_base64.len() > MAX_AUDIO_PAYLOAD_CHARS {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/payload-too-large".to_string(),
            title: "Audio Payload Too Large".to_string(),
            status: StatusCode::PAYLOAD_TOO_LARGE.as_u16(),
            detail: format!(
                "The 'audio_base64' payload exceeds the 32MB limit (was {} bytes)",
                payload.audio_base64.len()
            ),
            instance: "/v1/audio/speech-to-speech".to_string(),
        };
        return Err((StatusCode::PAYLOAD_TOO_LARGE, Json(err)));
    }

    let (pcm_samples, sample_rate) = decode_audio(&payload.audio_base64).map_err(|detail| {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/invalid-audio-input".to_string(),
            title: "Invalid Audio Input".to_string(),
            status: StatusCode::BAD_REQUEST.as_u16(),
            detail,
            instance: "/v1/audio/speech-to-speech".to_string(),
        };
        (StatusCode::BAD_REQUEST, Json(err))
    })?;

    // 1. Transcribe source speech via ASR
    let asr_engine = state.asr_registry.default_engine().await.ok_or_else(|| {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/no-asr-engine".to_string(),
            title: "No Default ASR Engine".to_string(),
            status: 500,
            detail: "No default speech recognition engine registered".to_string(),
            instance: "/v1/audio/speech-to-speech".to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
    })?;

    let transcription = asr_engine
        .transcribe(&pcm_samples, sample_rate, &TranscriptionOptions::default())
        .await
        .map_err(|e| {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/transcription-failure".to_string(),
                title: "ASR Transcription Failed".to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                detail: format!("Failed to transcribe input voice: {}", e),
                instance: "/v1/audio/speech-to-speech".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
        })?;

    let text = transcription.text.trim();
    if text.is_empty() {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/no-speech".to_string(),
            title: "No Speech Detected".to_string(),
            status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
            detail: "No intelligible speech detected in source audio".to_string(),
            instance: "/v1/audio/speech-to-speech".to_string(),
        };
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(err)));
    }

    // 2. Resolve target voice and engine
    let (engine, voice) = state
        .engine_registry
        .resolve_voice(&payload.target_voice)
        .await
        .map_err(|e| {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/voice-not-found".to_string(),
                title: "Target Voice Not Found".to_string(),
                status: StatusCode::NOT_FOUND.as_u16(),
                detail: e.to_string(),
                instance: "/v1/audio/speech-to-speech".to_string(),
            };
            (StatusCode::NOT_FOUND, Json(err))
        })?;

    // 3. Compute speed adjustment if preserve_tempo is requested
    let target_speed = if payload.preserve_tempo && transcription.duration_seconds > 0.5 {
        let word_count = text.split_whitespace().count().max(1);
        let estimated_normal_dur = word_count as f64 / 2.8;
        let tempo_ratio = estimated_normal_dur / (transcription.duration_seconds as f64);
        (tempo_ratio as f32 * payload.speed).clamp(0.6, 2.2)
    } else {
        payload.speed
    };

    // 4. Synthesize with target voice
    let synth_req = SynthesisRequest {
        text: state.pronunciation.process(text),
        voice_id: voice.id.clone(),
        speed: target_speed,
        pitch: 0.0,
        format: payload.response_format,
    };

    let chunk = state
        .engine_registry
        .synthesize_cached(engine, &synth_req)
        .await
        .map_err(|e| {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/synthesis-failure".to_string(),
                title: "Voice Conversion Synthesis Failed".to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                detail: e.to_string(),
                instance: "/v1/audio/speech-to-speech".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
        })?;

    // 5. Encode to output audio format
    let wav_bytes =
        WavEncoder::encode_pcm16_to_wav(&chunk.pcm_data, chunk.sample_rate, chunk.channels)
            .map_err(|e| {
                let err = ProblemDetails {
                    problem_type: "https://voxforg.org/errors/encoding-failure".to_string(),
                    title: "Audio Encoding Error".to_string(),
                    status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    detail: e.to_string(),
                    instance: "/v1/audio/speech-to-speech".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
            })?;

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/wav")
        .body(Body::from(wav_bytes))
        .map_err(|e| {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/internal".to_string(),
                title: "Response Build Failed".to_string(),
                status: 500,
                detail: e.to_string(),
                instance: "/v1/audio/speech-to-speech".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
        })?;

    response.headers_mut().insert(
        header::HeaderName::from_static("x-voxforg-source-duration-sec"),
        HeaderValue::from_str(&format!("{:.2}", transcription.duration_seconds))
            .unwrap_or_else(|_| HeaderValue::from_static("0.00")),
    );
    response.headers_mut().insert(
        header::HeaderName::from_static("x-voxforg-target-voice"),
        HeaderValue::from_str(&voice.id).unwrap_or_else(|_| HeaderValue::from_static("unknown")),
    );

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_b64_standard() {
        let res = decode_b64("SGVsbG8gV29ybGQ=");
        assert_eq!(res, Some(b"Hello World".to_vec()));
    }

    #[test]
    fn test_decode_b64_url_safe() {
        // - and _ in place of + and /
        let res = decode_b64("-_==");
        assert!(res.is_some());
    }

    #[test]
    fn test_decode_b64_whitespace_and_padding() {
        let res = decode_b64("  SGVsbG8g\nV29ybGQ= \n");
        assert_eq!(res, Some(b"Hello World".to_vec()));
    }

    #[test]
    fn test_decode_b64_empty_and_invalid() {
        assert_eq!(decode_b64(""), None);
        assert_eq!(decode_b64("   "), None);
    }

    #[test]
    fn test_decode_audio_wav_payload() {
        let wav_bytes = WavEncoder::encode_pcm16_to_wav(&[100, 200, -200, -100], 16000, 1).unwrap();
        let b64 = format!("data:audio/wav;base64,{}", hex_sim_or_b64(&wav_bytes));
        let (samples, sr) = decode_audio(&b64).unwrap();
        assert_eq!(sr, 16000);
        assert_eq!(samples.len(), 4);
        assert_eq!(samples, vec![100, 200, -200, -100]);
    }

    #[test]
    fn test_decode_audio_raw_pcm_fallback() {
        // 2 raw 16-bit samples: 512, 1024
        let pcm_bytes = vec![0x00, 0x02, 0x00, 0x04];
        let b64 = hex_sim_or_b64(&pcm_bytes);
        let (samples, sr) = decode_audio(&b64).unwrap();
        assert_eq!(sr, 16000);
        assert_eq!(samples, vec![512, 1024]);
    }

    #[test]
    fn test_decode_audio_stereo_wav_downmix() {
        // Interleaved stereo samples: (100, 200), (-100, -200) -> mono: (150, -150)
        let wav_bytes = WavEncoder::encode_pcm16_to_wav(&[100, 200, -100, -200], 16000, 2).unwrap();
        let b64 = format!("data:audio/wav;base64,{}", hex_sim_or_b64(&wav_bytes));
        let (samples, sr) = decode_audio(&b64).unwrap();
        assert_eq!(sr, 16000);
        assert_eq!(samples.len(), 2);
        assert_eq!(samples, vec![150, -150]);
    }

    #[test]
    fn test_decode_audio_odd_raw_pcm_rejected() {
        // 3 bytes cannot form 16-bit PCM
        let odd_pcm = vec![0x00, 0x02, 0x00];
        let b64 = hex_sim_or_b64(&odd_pcm);
        assert!(decode_audio(&b64).is_err());
    }

    #[test]
    fn test_decode_audio_empty_err() {
        assert!(decode_audio("").is_err());
        assert!(decode_audio("   ").is_err());
    }

    fn hex_sim_or_b64(bytes: &[u8]) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let n = (b0 << 16) | (b1 << 8) | b2;
            out.push(CHARS[((n >> 18) & 63) as usize] as char);
            out.push(CHARS[((n >> 12) & 63) as usize] as char);
            if chunk.len() > 1 {
                out.push(CHARS[((n >> 6) & 63) as usize] as char);
            } else {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(CHARS[(n & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
        out
    }
}
