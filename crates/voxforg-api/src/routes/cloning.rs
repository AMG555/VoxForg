//! REST endpoints for voice cloning and voice profile management.
//!
//! | Method | Path                         | Description                             |
//! |--------|------------------------------|-----------------------------------------|
//! | GET    | `/v1/voices/profiles`        | List all persistent voice profiles      |
//! | POST   | `/v1/voices/clone`           | Clone a voice from reference audio      |
//! | GET    | `/v1/voices/profiles/{id}`   | Retrieve a specific voice profile       |
//! | DELETE | `/v1/voices/profiles/{id}`   | Delete a voice profile                  |

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use voxforg_core::error::ProblemDetails;
use voxforg_core::models::{CloneVoiceRequest, VoiceProfile};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct VoiceProfilesListResponse {
    pub count: usize,
    pub profiles: Vec<VoiceProfile>,
}

/// `GET /v1/voices/profiles` — list all stored voice profiles.
pub async fn list_profiles(State(state): State<AppState>) -> Json<VoiceProfilesListResponse> {
    let profiles = state.voice_profiles.list().await;
    Json(VoiceProfilesListResponse {
        count: profiles.len(),
        profiles,
    })
}

/// `GET /v1/voices/profiles/{id}` — retrieve a specific voice profile.
pub async fn get_profile(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<VoiceProfile>, (StatusCode, Json<ProblemDetails>)> {
    match state.voice_profiles.get(&id).await {
        Some(profile) => Ok(Json(profile)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/profile-not-found".to_string(),
                title: "Voice Profile Not Found".to_string(),
                status: 404,
                detail: format!("Voice profile '{id}' not found"),
                instance: format!("/v1/voices/profiles/{id}"),
            }),
        )),
    }
}

/// `POST /v1/voices/clone` — clone a voice from reference audio.
pub async fn clone_voice(
    State(state): State<AppState>,
    Json(body): Json<CloneVoiceRequest>,
) -> Result<(StatusCode, Json<VoiceProfile>), (StatusCode, Json<ProblemDetails>)> {
    if body.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/invalid-parameter".to_string(),
                title: "Invalid Voice Name".to_string(),
                status: 400,
                detail: "The 'name' field cannot be empty".to_string(),
                instance: "/v1/voices/clone".to_string(),
            }),
        ));
    }

    if body.reference_audio_base64.is_none() && body.reference_audio_path.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/missing-audio".to_string(),
                title: "Missing Reference Audio".to_string(),
                status: 400,
                detail: "Either 'reference_audio_base64' or 'reference_audio_path' is required"
                    .to_string(),
                instance: "/v1/voices/clone".to_string(),
            }),
        ));
    }

    // Resolve the target cloning engine
    let engine = state
        .engine_registry
        .get(&body.engine_id)
        .await
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/engine-not-found".to_string(),
                    title: "Engine Not Found".to_string(),
                    status: 404,
                    detail: format!("Engine '{}' not found in registry", body.engine_id),
                    instance: "/v1/voices/clone".to_string(),
                }),
            )
        })?;

    if !engine.supports_cloning() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/cloning-unsupported".to_string(),
                title: "Cloning Unsupported".to_string(),
                status: 400,
                detail: format!(
                    "Engine '{}' does not support zero-shot voice cloning",
                    body.engine_id
                ),
                instance: "/v1/voices/clone".to_string(),
            }),
        ));
    }

    let profile = engine.clone_voice(&body).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/cloning-failed".to_string(),
                title: "Voice Cloning Failed".to_string(),
                status: 500,
                detail: e.to_string(),
                instance: "/v1/voices/clone".to_string(),
            }),
        )
    })?;

    state.voice_profiles.insert(profile.clone()).await;

    Ok((StatusCode::CREATED, Json(profile)))
}

/// `DELETE /v1/voices/profiles/{id}` — remove a stored voice profile.
pub async fn delete_profile(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetails>)> {
    match state.voice_profiles.remove(&id).await {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/profile-not-found".to_string(),
                title: "Voice Profile Not Found".to_string(),
                status: 404,
                detail: format!("Voice profile '{id}' not found"),
                instance: format!("/v1/voices/profiles/{id}"),
            }),
        )),
    }
}
