use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use futures::stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::warn;

use voxforg_audio::WavEncoder;
use voxforg_core::error::ProblemDetails;
use voxforg_core::models::{AudioContainerFormat, Voice};
use voxforg_engine::{SynthesisRequest, TtsEngine};
use voxforg_router::SynthesisPolicy;

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
    /// Optional SLA-policy-based routing. When present the engine is chosen
    /// automatically by `VoiceRouter`; `model` and `voice` still select the
    /// voice *within* the chosen engine as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<SynthesisPolicy>,
}

fn default_speed() -> f32 {
    1.0
}

fn validate_speech_request(
    payload: &OpenAiSpeechRequest,
) -> Result<(), (StatusCode, Json<ProblemDetails>)> {
    if payload.input.trim().is_empty() {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/empty-input".to_string(),
            title: "Empty Input Text".to_string(),
            status: StatusCode::BAD_REQUEST.as_u16(),
            detail: "The 'input' parameter must contain at least 1 non-whitespace character"
                .to_string(),
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

    if !payload.speed.is_finite() || !(0.25..=4.0).contains(&payload.speed) {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/invalid-parameter".to_string(),
            title: "Invalid Speed Parameter".to_string(),
            status: StatusCode::BAD_REQUEST.as_u16(),
            detail: "The 'speed' parameter must be a finite number between 0.25 and 4.0"
                .to_string(),
            instance: "/v1/audio/speech".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(err)));
    }

    if let Some(p) = payload.pitch {
        if !p.is_finite() || !(-50.0..=50.0).contains(&p) {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/invalid-parameter".to_string(),
                title: "Invalid Pitch Parameter".to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
                detail:
                    "The 'pitch' parameter must be a finite number between -50.0 and 50.0 semitones"
                        .to_string(),
                instance: "/v1/audio/speech".to_string(),
            };
            return Err((StatusCode::BAD_REQUEST, Json(err)));
        }
    }

    Ok(())
}

/// Resolve engine + voice, honouring `SynthesisPolicy` when provided.
/// When no policy is present the legacy `resolve_voice` → `model` fallback path
/// is used unchanged, so all existing callers continue to work.
async fn resolve_engine_and_voice(
    state: &AppState,
    payload: &OpenAiSpeechRequest,
) -> Result<(Arc<dyn TtsEngine>, Voice), (StatusCode, Json<ProblemDetails>)> {
    // ── Policy-based routing path ─────────────────────────────────────────
    if let Some(policy) = &payload.policy {
        return match state
            .voice_router
            .route(
                policy,
                &payload.input,
                &payload.voice,
                payload.response_format,
            )
            .await
        {
            Ok((decision, _synth_req)) => {
                // Build a synthetic Voice so the rest of the handler stays unchanged
                let voice = Voice {
                    id: payload.voice.clone(),
                    name: payload.voice.clone(),
                    engine_id: decision.engine_id.clone(),
                    language: policy
                        .language
                        .clone()
                        .unwrap_or_else(|| "en-US".to_string()),
                    gender: voxforg_core::models::Gender::Neutral,
                    sample_rate_hz: 24000,
                    tags: vec![],
                    description: None,
                };
                Ok((decision.engine, voice))
            }
            Err(e) => {
                let err = ProblemDetails {
                    problem_type: "https://voxforg.org/errors/no-engine-for-policy".to_string(),
                    title: "No Engine Satisfies Policy".to_string(),
                    status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                    detail: e.to_string(),
                    instance: "/v1/audio/speech".to_string(),
                };
                Err((StatusCode::UNPROCESSABLE_ENTITY, Json(err)))
            }
        };
    }

    // ── Voice Identity resolution path (portable voice abstraction) ─────
    if let Ok(res) = state
        .voice_identities
        .resolve(&payload.voice, &state.engine_registry)
        .await
    {
        return Ok(res);
    }

    // ── Legacy path (concrete voice_id / model fallback) ───────────────────
    match state.engine_registry.resolve_voice(&payload.voice).await {
        Ok(res) => Ok(res),
        Err(_) => {
            if let Some(engine) = state.engine_registry.get(&payload.model).await {
                let voices = engine.voices().await.unwrap_or_default();
                let chosen_voice = voices.into_iter().next().unwrap_or(Voice {
                    id: payload.voice.clone(),
                    name: payload.voice.clone(),
                    engine_id: engine.id().to_string(),
                    language: "en-US".to_string(),
                    gender: voxforg_core::models::Gender::Neutral,
                    sample_rate_hz: 24000,
                    tags: vec![],
                    description: None,
                });
                Ok((engine, chosen_voice))
            } else {
                let err = ProblemDetails {
                    problem_type: "https://voxforg.org/errors/voice-not-found".to_string(),
                    title: "Voice or Engine Not Found".to_string(),
                    status: StatusCode::NOT_FOUND.as_u16(),
                    detail: format!(
                        "Neither voice '{}' nor engine '{}' could be found",
                        payload.voice, payload.model
                    ),
                    instance: "/v1/audio/speech".to_string(),
                };
                Err((StatusCode::NOT_FOUND, Json(err)))
            }
        }
    }
}

pub async fn synthesize_speech(
    State(state): State<AppState>,
    Json(payload): Json<OpenAiSpeechRequest>,
) -> Result<Response, (StatusCode, Json<ProblemDetails>)> {
    validate_speech_request(&payload)?;

    // ── Distributed worker dispatch path ──────────────────────────────────
    let is_explicit_worker = payload.model.starts_with("worker:");
    let worker_target = payload
        .model
        .strip_prefix("worker:")
        .unwrap_or(&payload.model);

    if is_explicit_worker || state.engine_registry.get(&payload.model).await.is_none() {
        if let Some(worker) = state.worker_pool.select_worker(worker_target).await {
            if let Err(e) = state.worker_pool.acquire_lease(&worker.worker_id).await {
                let err = ProblemDetails {
                    problem_type: "https://voxforg.org/errors/worker-busy".to_string(),
                    title: "Worker Node Busy".to_string(),
                    status: StatusCode::TOO_MANY_REQUESTS.as_u16(),
                    detail: format!("Worker '{}' at capacity: {}", worker.worker_id, e),
                    instance: "/v1/audio/speech".to_string(),
                };
                return Err((StatusCode::TOO_MANY_REQUESTS, Json(err)));
            }

            let mut worker_payload = serde_json::to_value(&payload).unwrap();
            if let Some(obj) = worker_payload.as_object_mut() {
                obj.insert(
                    "model".to_string(),
                    serde_json::Value::String(worker_target.to_string()),
                );
            }

            let dispatch_res = state
                .worker_client
                .dispatch_raw(&worker, &worker_payload)
                .await;
            let _ = state.worker_pool.release_lease(&worker.worker_id).await;

            return match dispatch_res {
                Ok(audio_bytes) => {
                    let content_type = payload.response_format.mime_type();
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, content_type)
                        .header("x-voxforg-dispatched-worker", worker.worker_id.to_string())
                        .body(Body::from(audio_bytes))
                        .map_err(|e| {
                            let err = ProblemDetails {
                                problem_type: "https://voxforg.org/errors/internal".to_string(),
                                title: "Response Build Failed".to_string(),
                                status: 500,
                                detail: e.to_string(),
                                instance: "/v1/audio/speech".to_string(),
                            };
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
                        })?;
                    Ok(response)
                }
                Err(e) => {
                    let err = ProblemDetails {
                        problem_type: "https://voxforg.org/errors/worker-dispatch-failed"
                            .to_string(),
                        title: "Worker Dispatch Failed".to_string(),
                        status: StatusCode::BAD_GATEWAY.as_u16(),
                        detail: e.to_string(),
                        instance: "/v1/audio/speech".to_string(),
                    };
                    Err((StatusCode::BAD_GATEWAY, Json(err)))
                }
            };
        } else if is_explicit_worker {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/no-worker-available".to_string(),
                title: "No Worker Available".to_string(),
                status: StatusCode::NOT_FOUND.as_u16(),
                detail: format!(
                    "No available cluster worker node supports model '{worker_target}'"
                ),
                instance: "/v1/audio/speech".to_string(),
            };
            return Err((StatusCode::NOT_FOUND, Json(err)));
        }
    }

    let (engine, voice) = resolve_engine_and_voice(&state, &payload).await?;

    let synth_req = SynthesisRequest {
        // Apply pronunciation normalization before synthesis:
        // symbols (₹→rupees), scale suffixes (1K→1000), dictionary overrides (SQL→sequel)
        text: state.pronunciation.process(&payload.input),
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

    let (audio_bytes, content_type) = match payload.response_format {
        AudioContainerFormat::Pcm => {
            let bytes: Vec<u8> = audio_chunk
                .pcm_data
                .iter()
                .flat_map(|s| s.to_le_bytes())
                .collect();
            (bytes, "audio/pcm")
        }
        _ => {
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
            (wav_bytes, "audio/wav")
        }
    };

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static(content_type))
        .header(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"))
        .body(Body::from(audio_bytes))
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

pub async fn synthesize_speech_stream(
    State(state): State<AppState>,
    Json(payload): Json<OpenAiSpeechRequest>,
) -> Result<Response, (StatusCode, Json<ProblemDetails>)> {
    validate_speech_request(&payload)?;
    let (engine, voice) = resolve_engine_and_voice(&state, &payload).await?;

    let synth_req = SynthesisRequest {
        text: payload.input,
        voice_id: voice.id,
        speed: payload.speed,
        pitch: payload.pitch.unwrap_or(0.0),
        format: payload.response_format,
    };

    let rx = engine.synthesize_stream(&synth_req).await.map_err(|e| {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/synthesis-failure".to_string(),
            title: "Streaming Synthesis Error".to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            detail: e.to_string(),
            instance: "/v1/audio/speech/stream".to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
    })?;

    let stream = stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Some(Ok(chunk)) => {
                let bytes: Vec<u8> = chunk
                    .pcm_data
                    .iter()
                    .flat_map(|s| s.to_le_bytes())
                    .collect();
                Some((Ok::<_, std::io::Error>(Bytes::from(bytes)), rx))
            }
            Some(Err(e)) => {
                warn!("Streaming synthesis chunk error: {e}");
                None
            }
            None => None,
        }
    });

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static("audio/pcm"))
        .header(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"))
        .header(
            header::TRANSFER_ENCODING,
            HeaderValue::from_static("chunked"),
        )
        .body(Body::from_stream(stream))
        .map_err(|e| {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/internal".to_string(),
                title: "Internal Error".to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                detail: e.to_string(),
                instance: "/v1/audio/speech/stream".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
        })?;

    Ok(response)
}

pub async fn speech_websocket(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_speech_socket(socket, state))
}

async fn handle_speech_socket(mut socket: WebSocket, state: AppState) {
    while let Some(msg_res) = socket.recv().await {
        let msg = match msg_res {
            Ok(m) => m,
            Err(_) => break,
        };

        match msg {
            Message::Text(text) => {
                let req: OpenAiSpeechRequest = match serde_json::from_str(&text) {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = socket
                            .send(Message::Text(
                                json!({ "error": format!("Invalid JSON: {e}") }).to_string(),
                            ))
                            .await;
                        continue;
                    }
                };

                let (engine, voice) = match state.engine_registry.resolve_voice(&req.voice).await {
                    Ok(res) => res,
                    Err(_) => {
                        let _ = socket
                            .send(Message::Text(
                                json!({ "error": format!("Voice '{}' not found", req.voice) })
                                    .to_string(),
                            ))
                            .await;
                        continue;
                    }
                };

                let synth_req = SynthesisRequest {
                    text: req.input,
                    voice_id: voice.id,
                    speed: req.speed.clamp(0.25, 4.0),
                    pitch: req.pitch.unwrap_or(0.0).clamp(-50.0, 50.0),
                    format: req.response_format,
                };

                let mut rx = match engine.synthesize_stream(&synth_req).await {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = socket
                            .send(Message::Text(json!({ "error": e.to_string() }).to_string()))
                            .await;
                        continue;
                    }
                };

                let mut total_samples = 0;
                while let Some(chunk_res) = rx.recv().await {
                    match chunk_res {
                        Ok(chunk) => {
                            total_samples += chunk.pcm_data.len();
                            let bytes: Vec<u8> = chunk
                                .pcm_data
                                .iter()
                                .flat_map(|s| s.to_le_bytes())
                                .collect();
                            if socket.send(Message::Binary(bytes)).await.is_err() {
                                return;
                            }
                        }
                        Err(e) => {
                            let _ = socket
                                .send(Message::Text(json!({ "error": e.to_string() }).to_string()))
                                .await;
                            break;
                        }
                    }
                }

                let _ = socket
                    .send(Message::Text(
                        json!({ "event": "done", "total_samples": total_samples }).to_string(),
                    ))
                    .await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
