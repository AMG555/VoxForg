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

fn decode_audio(payload: &str) -> Result<(Vec<i16>, u32), String> {
    let clean = if let Some(idx) = payload.find(',') {
        payload[idx + 1..].trim()
    } else {
        payload.trim()
    };

    let bytes = decode_b64(clean)
        .or_else(|| hex::decode(clean).ok())
        .unwrap_or_else(|| clean.as_bytes().to_vec());

    if bytes.is_empty() {
        return Err("Audio payload is empty".to_string());
    }

    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        let cursor = Cursor::new(&bytes);
        if let Ok(mut reader) = hound::WavReader::new(cursor) {
            let spec = reader.spec();
            let samples: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
            if let Ok(s) = samples {
                return Ok((s, spec.sample_rate));
            }
        }
    }

    // Raw 16-bit PCM fallback
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
        let tempo_ratio = estimated_normal_dur / transcription.duration_seconds;
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
        HeaderValue::from_str(&format!("{:.2}", transcription.duration_seconds)).unwrap(),
    );
    response.headers_mut().insert(
        header::HeaderName::from_static("x-voxforg-target-voice"),
        HeaderValue::from_str(&voice.id).unwrap_or_else(|_| HeaderValue::from_static("unknown")),
    );

    Ok(response)
}
