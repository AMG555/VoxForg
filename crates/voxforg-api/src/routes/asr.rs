//! OpenAI-compatible Audio Transcription (ASR) endpoints.
//!
//! | Method | Path                         | Description                             |
//! |--------|------------------------------|-----------------------------------------|
//! | GET    | `/v1/asr/engines`            | List registered ASR inference engines   |
//! | POST   | `/v1/audio/transcriptions`   | Transcribe speech audio into text/subtitles |

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use voxforg_asr::{AsrEngineInfo, TranscriptionOptions};
use voxforg_core::error::ProblemDetails;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct AsrEnginesListResponse {
    pub count: usize,
    pub engines: Vec<AsrEngineInfo>,
}

/// `GET /v1/asr/engines` — list all registered ASR engines.
pub async fn list_asr_engines(State(state): State<AppState>) -> Json<AsrEnginesListResponse> {
    let engines = state.asr_registry.list().await;
    Json(AsrEnginesListResponse {
        count: engines.len(),
        engines,
    })
}

/// Request payload for `POST /v1/audio/transcriptions`.
#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptionRequestPayload {
    /// Base64 or hex-encoded audio payload (WAV or raw 16-bit PCM).
    #[serde(default)]
    pub audio_base64: Option<String>,

    /// Path to a local audio file on disk.
    #[serde(default)]
    pub audio_path: Option<String>,

    /// Model ID or name (e.g. "whisper-base", "mock-asr"). Defaults to registry default.
    #[serde(default)]
    pub model: Option<String>,

    /// Target language code (e.g. "en", "es").
    #[serde(default)]
    pub language: Option<String>,

    /// Guiding prompt for transcription.
    #[serde(default)]
    pub prompt: Option<String>,

    /// Output format: "json", "text", "srt", "vtt", "verbose_json".
    #[serde(default)]
    pub response_format: Option<String>,

    /// Sampling temperature between 0.0 and 1.0.
    #[serde(default)]
    pub temperature: Option<f32>,

    /// Timestamp granularities: e.g. ["word", "segment"].
    #[serde(default)]
    pub timestamp_granularities: Option<Vec<String>>,
}

/// Default JSON response format for OpenAI compatibility.
#[derive(Debug, Serialize)]
pub struct SimpleTranscriptionResponse {
    pub text: String,
}

/// `POST /v1/audio/transcriptions` — OpenAI-compatible transcription endpoint.
pub async fn transcribe_audio(
    State(state): State<AppState>,
    Json(payload): Json<TranscriptionRequestPayload>,
) -> Result<Response, (StatusCode, Json<ProblemDetails>)> {
    // 1. Decode audio data
    let (pcm_samples, sample_rate) = decode_audio_input(&payload).map_err(|detail| {
        (
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/invalid-audio-input".to_string(),
                title: "Invalid Audio Input".to_string(),
                status: 400,
                detail,
                instance: "/v1/audio/transcriptions".to_string(),
            }),
        )
    })?;

    // 2. Resolve target ASR engine
    let engine = if let Some(model_id) = &payload.model {
        state.asr_registry.get(model_id).await.ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/asr-engine-not-found".to_string(),
                    title: "ASR Engine Not Found".to_string(),
                    status: 404,
                    detail: format!("ASR engine '{model_id}' not found in registry"),
                    instance: "/v1/audio/transcriptions".to_string(),
                }),
            )
        })?
    } else {
        state.asr_registry.default_engine().await.ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/no-asr-engine".to_string(),
                    title: "No Default ASR Engine".to_string(),
                    status: 500,
                    detail: "No default ASR engine registered".to_string(),
                    instance: "/v1/audio/transcriptions".to_string(),
                }),
            )
        })?
    };

    // 3. Configure options
    let wants_words = payload
        .timestamp_granularities
        .as_ref()
        .map(|g| g.iter().any(|s| s == "word"))
        .unwrap_or(true);

    let format_str = payload
        .response_format
        .clone()
        .unwrap_or_else(|| "json".to_string())
        .to_lowercase();

    let options = TranscriptionOptions {
        language: payload.language.clone(),
        temperature: payload.temperature,
        prompt: payload.prompt.clone(),
        word_timestamps: wants_words,
        response_format: format_str.clone(),
    };

    // 4. Perform transcription
    let result = engine
        .transcribe(&pcm_samples, sample_rate, &options)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/transcription-failed".to_string(),
                    title: "Transcription Failed".to_string(),
                    status: 500,
                    detail: e.to_string(),
                    instance: "/v1/audio/transcriptions".to_string(),
                }),
            )
        })?;

    // 5. Serialize output according to response_format
    match format_str.as_str() {
        "text" => {
            let mut res = Response::new(Body::from(result.text));
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            Ok(res)
        }
        "srt" => {
            let srt = result.to_srt();
            let mut res = Response::new(Body::from(srt));
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            Ok(res)
        }
        "vtt" => {
            let vtt = result.to_vtt();
            let mut res = Response::new(Body::from(vtt));
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/vtt; charset=utf-8"),
            );
            Ok(res)
        }
        "verbose_json" => Ok(Json(result).into_response()),
        _ => {
            // Default OpenAI simple json
            let simple = SimpleTranscriptionResponse { text: result.text };
            Ok(Json(simple).into_response())
        }
    }
}

/// Helper function to parse input audio from base64 string or file path.
fn decode_audio_input(payload: &TranscriptionRequestPayload) -> Result<(Vec<i16>, u32), String> {
    if let Some(b64) = &payload.audio_base64 {
        let clean = b64.trim();
        let bytes = if let Ok(decoded) = hex::decode(clean) {
            decoded
        } else {
            // Hex fallback or simple byte mapping
            clean.as_bytes().to_vec()
        };

        if bytes.is_empty() {
            return Err("Decoded audio payload is empty".to_string());
        }

        // Try reading as WAV with hound
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

        // Fallback: parse 16-bit little-endian samples at 16000Hz
        let mut samples = Vec::with_capacity(bytes.len() / 2);
        for chunk in bytes.chunks_exact(2) {
            samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
        }

        if samples.is_empty() {
            return Err("No valid 16-bit PCM audio samples found in payload".to_string());
        }

        return Ok((samples, 16000));
    }

    if let Some(path) = &payload.audio_path {
        if let Ok(mut reader) = hound::WavReader::open(path) {
            let spec = reader.spec();
            let samples: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
            if let Ok(s) = samples {
                return Ok((s, spec.sample_rate));
            }
        }
        return Err(format!("Could not read WAV file at '{path}'"));
    }

    Err("Either 'audio_base64' or 'audio_path' must be provided".to_string())
}
