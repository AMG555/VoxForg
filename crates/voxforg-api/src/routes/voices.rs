use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use voxforg_core::models::Voice;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct VoiceFilterQuery {
    pub language: Option<String>,
    pub engine: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VoicesResponse {
    pub voices: Vec<Voice>,
    pub total: usize,
}

pub async fn list_voices(
    State(state): State<AppState>,
    Query(query): Query<VoiceFilterQuery>,
) -> Json<VoicesResponse> {
    let mut voices = state.engine_registry.list_all_voices().await.unwrap_or_default();

    if let Some(lang) = &query.language {
        voices.retain(|v| v.language.starts_with(lang));
    }
    if let Some(eng) = &query.engine {
        voices.retain(|v| &v.engine_id == eng);
    }

    let total = voices.len();
    Json(VoicesResponse { voices, total })
}
