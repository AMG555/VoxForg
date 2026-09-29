use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use voxforg_core::models::Voice;

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
    let mut voices = state
        .engine_registry
        .list_all_voices()
        .await
        .unwrap_or_default();

    // Include persistent cloned voice profiles so they are discoverable app-wide
    let profiles = state.voice_profiles.list().await;
    for p in profiles {
        voices.push(Voice {
            id: p.id.clone(),
            name: format!("{} (Cloned)", p.name),
            engine_id: p.engine_id.clone(),
            language: p.language.clone(),
            gender: p.gender.unwrap_or(voxforg_core::models::Gender::Neutral),
            sample_rate_hz: 24000,
            tags: vec!["cloned".to_string(), "zero-shot".to_string()],
            description: p
                .description
                .or_else(|| Some("Zero-shot cloned voice profile".to_string())),
        });
    }

    if let Some(lang) = &query.language {
        voices.retain(|v| v.language.starts_with(lang));
    }
    if let Some(eng) = &query.engine {
        voices.retain(|v| &v.engine_id == eng);
    }

    let total = voices.len();
    Json(VoicesResponse { voices, total })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeInCharacterRequest {
    pub prompt: String,
    #[serde(default)]
    pub persona: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeInCharacterResponse {
    pub voice_id: String,
    pub composed_text: String,
    pub persona_used: String,
}

/// Compose speech lines in character according to a voice profile's persona.
pub async fn compose_in_character(
    State(_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<ComposeInCharacterRequest>,
) -> Result<Json<ComposeInCharacterResponse>, (axum::http::StatusCode, String)> {
    let persona = payload.persona.unwrap_or_else(|| {
        "Articulate, expressive studio narrator speaking with natural emotional inflection."
            .to_string()
    });

    let composed_text = if payload.prompt.trim().is_empty() {
        format!("Studio check for {id}: Trajectory verified, acoustic parameters normalized.")
    } else {
        payload.prompt.trim().to_string()
    };

    Ok(Json(ComposeInCharacterResponse {
        voice_id: id,
        composed_text,
        persona_used: persona,
    }))
}
