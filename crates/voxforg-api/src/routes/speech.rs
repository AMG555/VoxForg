use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use voxforg_audio::WavEncoder;
use voxforg_core::error::ProblemDetails;
use voxforg_core::models::AudioContainerFormat;
use voxforg_engine::SynthesisRequest;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiSpeechRequest {
    pub model: String,
    pub input: String,
    pub voice: String,
    #[serde(default)]
    pub response_format: AudioContainerFormat,
    #[serde(default = "default_speed")]
    pub speed: f32,
    #[serde(default)]
    pub pitch: Option<f32>,
}

fn default_speed() -> f32 {
    1.0
}

pub async fn synthesize_speech(
    State(state): State<AppState>,
    Json(payload): Json<OpenAiSpeechRequest>,
) -> Result<Response, (StatusCode, Json<ProblemDetails>)> {
    if payload.input.trim().is_empty() {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/empty-input".to_string(),
            title: "Empty Input Text".to_string(),
            status: StatusCode::BAD_REQUEST.as_u16(),
            detail: "The 'input' parameter must contain at least 1 non-whitespace character".to_string(),
            instance: "/v1/audio/speech".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(err)));
    }

    if payload.input.len() > 10_000 {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/input-too-large".to_string(),
            title: "Input Text Exceeds Limit".to_string(),
            status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
            detail: "The 'input' parameter cannot exceed 10,000 characters per request".to_string(),
            instance: "/v1/audio/speech".to_string(),
        };
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(err)));
    }

    if !payload.speed.is_finite() || payload.speed < 0.25 || payload.speed > 4.0 {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/invalid-parameter".to_string(),
            title: "Invalid Speed Parameter".to_string(),
            status: StatusCode::BAD_REQUEST.as_u16(),
            detail: "The 'speed' parameter must be a finite number between 0.25 and 4.0".to_string(),
            instance: "/v1/audio/speech".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(err)));
    }

    if let Some(p) = payload.pitch {
        if !p.is_finite() || p < -50.0 || p > 50.0 {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/invalid-parameter".to_string(),
                title: "Invalid Pitch Parameter".to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
                detail: "The 'pitch' parameter must be a finite number between -50.0 and 50.0 semitones".to_string(),
                instance: "/v1/audio/speech".to_string(),
            };
            return Err((StatusCode::BAD_REQUEST, Json(err)));
        }
    }

    let (engine, voice) = match state.engine_registry.resolve_voice(&payload.voice).await {
        Ok(res) => res,
        Err(_) => {
            if let Some(engine) = state.engine_registry.get(&payload.model).await {
                let voices = engine.voices().await.unwrap_or_default();
                let chosen_voice = voices.into_iter().next().unwrap_or(voxforg_core::models::Voice {
                    id: payload.voice.clone(),
                    name: payload.voice.clone(),
                    engine_id: engine.id().to_string(),
                    language: "en-US".to_string(),
                    gender: voxforg_core::models::Gender::Neutral,
                    sample_rate_hz: 24000,
                    tags: vec![],
                    description: None,
                });
                (engine, chosen_voice)
            } else {
                let err = ProblemDetails {
                    problem_type: "https://voxforg.org/errors/voice-not-found".to_string(),
                    title: "Voice or Engine Not Found".to_string(),
                    status: StatusCode::NOT_FOUND.as_u16(),
                    detail: format!("Neither voice '{}' nor engine '{}' could be found", payload.voice, payload.model),
                    instance: "/v1/audio/speech".to_string(),
                };
                return Err((StatusCode::NOT_FOUND, Json(err)));
            }
        }
    };

    let synth_req = SynthesisRequest {
        text: payload.input,
        voice_id: voice.id,
        speed: payload.speed,
        pitch: payload.pitch.unwrap_or(0.0),
        format: payload.response_format,
    };

    let start = std::time::Instant::now();
    let audio_chunk = state
        .engine_registry
        .synthesize_cached(engine, &synth_req)
        .await
        .map_err(|e| {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/synthesis-failure".to_string(),
                title: "Synthesis Error".to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                detail: e.to_string(),
                instance: "/v1/audio/speech".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
        })?;

    let elapsed = start.elapsed();
    state
        .metrics
        .record_synthesis(elapsed.as_millis() as u64, audio_chunk.pcm_data.len());

    let wav_bytes = WavEncoder::encode_pcm16_to_wav(
        &audio_chunk.pcm_data,
        audio_chunk.sample_rate,
        audio_chunk.channels,
    )
    .map_err(|e| {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/encoding-failure".to_string(),
            title: "Audio Encoding Error".to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            detail: e.to_string(),
            instance: "/v1/audio/speech".to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
    })?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static("audio/wav"))
        .header(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"))
        .body(Body::from(wav_bytes))
        .map_err(|e| {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/internal".to_string(),
                title: "Internal Error".to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                detail: e.to_string(),
                instance: "/v1/audio/speech".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
        })?;

    Ok(response)
}
