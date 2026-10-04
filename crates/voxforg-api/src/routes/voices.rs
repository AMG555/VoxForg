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

/// Design a custom voice from natural language prompt and Director AI controls.
pub async fn design_voice(
    State(state): State<AppState>,
    Json(payload): Json<voxforg_engine::VoiceDesignRequest>,
) -> Result<Json<voxforg_core::models::VoiceProfile>, (axum::http::StatusCode, String)> {
    if payload.prompt.trim().is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Design prompt cannot be empty".to_string(),
        ));
    }

    let bank = voxforg_engine::ArchetypeBank::new();
    let profile = bank.design_voice(&payload);

    state
        .voice_profiles
        .save(profile.clone())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(profile))
}

/// Export a voice profile as a portable persona bundle.
pub async fn export_persona(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<voxforg_engine::PersonaBundle>, (axum::http::StatusCode, String)> {
    let profile = state
        .voice_profiles
        .get(&id)
        .await
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Voice profile not found".to_string()))?;

    let bundle = voxforg_engine::PersonaBundle::from_profile(profile);
    Ok(Json(bundle))
}

/// Import a portable persona bundle into the persistent voice profiles store.
pub async fn import_persona(
    State(state): State<AppState>,
    Json(bundle): Json<voxforg_engine::PersonaBundle>,
) -> Result<Json<voxforg_core::models::VoiceProfile>, (axum::http::StatusCode, String)> {
    let profile = bundle.unpack_profile();
    state
        .voice_profiles
        .save(profile.clone())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(profile))
}

