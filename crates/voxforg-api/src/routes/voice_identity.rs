//! REST endpoints for portable Voice Identity management.
//!
//! | Method | Path                         | Description                        |
//! |--------|------------------------------|------------------------------------|
//! | GET    | `/v1/voice-identities`       | List all voice identities          |
//! | POST   | `/v1/voice-identities`       | Register or update a voice identity|
//! | GET    | `/v1/voice-identities/{id}`  | Get voice identity by ID           |
//! | DELETE | `/v1/voice-identities/{id}`  | Remove voice identity by ID        |

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use voxforg_core::error::ProblemDetails;
use voxforg_core::models::VoiceIdentity;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct VoiceIdentitiesResponse {
    pub count: usize,
    pub identities: Vec<VoiceIdentity>,
}

/// `GET /v1/voice-identities` — list all registered voice identities.
pub async fn list_identities(State(state): State<AppState>) -> Json<VoiceIdentitiesResponse> {
    let identities = state.voice_identities.list().await;
    Json(VoiceIdentitiesResponse {
        count: identities.len(),
        identities,
    })
}

/// `GET /v1/voice-identities/{id}` — get a single voice identity.
pub async fn get_identity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<VoiceIdentity>, (StatusCode, Json<ProblemDetails>)> {
    match state.voice_identities.get(&id).await {
        Some(identity) => Ok(Json(identity)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/identity-not-found".to_string(),
                title: "Voice Identity Not Found".to_string(),
                status: 404,
                detail: format!("Voice identity '{id}' not found"),
                instance: format!("/v1/voice-identities/{id}"),
            }),
        )),
    }
}

/// `POST /v1/voice-identities` — register or update a voice identity.
pub async fn register_identity(
    State(state): State<AppState>,
    Json(body): Json<VoiceIdentity>,
) -> Result<(StatusCode, Json<VoiceIdentity>), (StatusCode, Json<ProblemDetails>)> {
    if body.id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/invalid-parameter".to_string(),
                title: "Empty Identity ID".to_string(),
                status: 400,
                detail: "The 'id' field cannot be empty".to_string(),
                instance: "/v1/voice-identities".to_string(),
            }),
        ));
    }

    if body.display_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/invalid-parameter".to_string(),
                title: "Empty Display Name".to_string(),
                status: 400,
                detail: "The 'display_name' field cannot be empty".to_string(),
                instance: "/v1/voice-identities".to_string(),
            }),
        ));
    }

    let existing = state.voice_identities.get(&body.id).await;
    state.voice_identities.register(body.clone()).await;

    let status = if existing.is_some() {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };

    Ok((status, Json(body)))
}

/// `DELETE /v1/voice-identities/{id}` — remove a voice identity.
pub async fn delete_identity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetails>)> {
    match state.voice_identities.remove(&id).await {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/identity-not-found".to_string(),
                title: "Voice Identity Not Found".to_string(),
                status: 404,
                detail: format!("Voice identity '{id}' not found"),
                instance: format!("/v1/voice-identities/{id}"),
            }),
        )),
    }
}
