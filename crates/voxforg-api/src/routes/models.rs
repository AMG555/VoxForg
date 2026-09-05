use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelCard {
    pub id: String,
    pub object: &'static str,
    pub created: u64,
    pub owned_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelListResponse {
    pub object: &'static str,
    pub data: Vec<ModelCard>,
}

pub async fn list_models(State(state): State<AppState>) -> Json<ModelListResponse> {
    let engines = state.engine_registry.list_engines().await;
    let data = engines
        .into_iter()
        .map(|eng| ModelCard {
            id: eng,
            object: "model",
            created: 1700000000,
            owned_by: "voxforg".to_string(),
        })
        .collect();

    Json(ModelListResponse {
        object: "list",
        data,
    })
}
