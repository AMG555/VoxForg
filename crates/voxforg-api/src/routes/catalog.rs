//! REST endpoints for Model Catalog and Weights Management.
//!
//! | Method | Path                             | Description                                  |
//! |--------|----------------------------------|----------------------------------------------|
//! | GET    | `/v1/catalog/models`             | Browse curated model catalogue with filters  |
//! | GET    | `/v1/catalog/models/{id}`        | Get single model details and status          |
//! | POST   | `/v1/catalog/models/{id}/install`| Download & register model weights            |
//! | DELETE | `/v1/catalog/models/{id}`        | Remove installed weights from storage        |

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use voxforg_catalog::{CatalogItem, ModelType};
use voxforg_core::error::ProblemDetails;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CatalogQuery {
    #[serde(rename = "type", default)]
    pub model_type: Option<String>,
    #[serde(default)]
    pub installed_only: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CatalogListResponse {
    pub count: usize,
    pub models: Vec<CatalogItem>,
}

/// `GET /v1/catalog/models` — browse model weights catalogue.
pub async fn list_catalog_models(
    State(state): State<AppState>,
    Query(query): Query<CatalogQuery>,
) -> Json<CatalogListResponse> {
    let filter_type = query
        .model_type
        .as_deref()
        .and_then(|t| match t.to_lowercase().as_str() {
            "tts" => Some(ModelType::Tts),
            "asr" => Some(ModelType::Asr),
            "vad" => Some(ModelType::Vad),
            "diarizer" => Some(ModelType::Diarizer),
            _ => None,
        });

    let installed_only = query.installed_only.unwrap_or(false);
    let models = state.model_catalog.list(filter_type, installed_only).await;

    Json(CatalogListResponse {
        count: models.len(),
        models,
    })
}

/// `GET /v1/catalog/models/{id}` — retrieve single model package information.
pub async fn get_catalog_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CatalogItem>, (StatusCode, Json<ProblemDetails>)> {
    match state.model_catalog.get(&id).await {
        Some(item) => Ok(Json(item)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/model-not-found".to_string(),
                title: "Model Not Found".to_string(),
                status: 404,
                detail: format!("Model '{id}' not found in catalogue"),
                instance: format!("/v1/catalog/models/{id}"),
            }),
        )),
    }
}

/// `POST /v1/catalog/models/{id}/install` — download and install model weights.
pub async fn install_catalog_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CatalogItem>, (StatusCode, Json<ProblemDetails>)> {
    match state
        .model_catalog
        .install(&id, &state.hardware_profile)
        .await
    {
        Ok(item) => Ok(Json(item)),
        Err(e) => {
            let status = match e {
                voxforg_core::error::VoxForgError::HardwareUnsupported(_) => {
                    StatusCode::BAD_REQUEST
                }
                voxforg_core::error::VoxForgError::Engine(ref msg) if msg.contains("not found") => {
                    StatusCode::NOT_FOUND
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((
                status,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/install-failed".to_string(),
                    title: "Model Installation Failed".to_string(),
                    status: status.as_u16(),
                    detail: e.to_string(),
                    instance: format!("/v1/catalog/models/{id}/install"),
                }),
            ))
        }
    }
}

/// `DELETE /v1/catalog/models/{id}` — remove local model weights.
pub async fn uninstall_catalog_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CatalogItem>, (StatusCode, Json<ProblemDetails>)> {
    match state.model_catalog.uninstall(&id).await {
        Ok(item) => Ok(Json(item)),
        Err(e) => {
            let status = match e {
                voxforg_core::error::VoxForgError::Engine(ref msg) if msg.contains("not found") => {
                    StatusCode::NOT_FOUND
                }
                voxforg_core::error::VoxForgError::Engine(ref msg)
                    if msg.contains("not currently installed") =>
                {
                    StatusCode::BAD_REQUEST
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((
                status,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/uninstall-failed".to_string(),
                    title: "Model Uninstallation Failed".to_string(),
                    status: status.as_u16(),
                    detail: e.to_string(),
                    instance: format!("/v1/catalog/models/{id}"),
                }),
            ))
        }
    }
}
