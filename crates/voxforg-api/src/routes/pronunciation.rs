//! REST endpoints for runtime pronunciation dictionary management.
//!
//! | Method | Path                                  | Description                   |
//! |--------|---------------------------------------|-------------------------------|
//! | GET    | `/v1/pronunciation/dictionary`        | List all dictionary entries   |
//! | POST   | `/v1/pronunciation/dictionary`        | Add or update an entry        |
//! | DELETE | `/v1/pronunciation/dictionary/{term}` | Remove an entry               |

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use voxforg_core::error::ProblemDetails;
use voxforg_pronunciation::DictionaryEntry;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct DictionaryListResponse {
    pub count: usize,
    pub entries: Vec<DictionaryEntry>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertEntryRequest {
    pub term: String,
    pub replacement: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// `GET /v1/pronunciation/dictionary` — return all current entries sorted by term.
pub async fn list_dictionary(State(state): State<AppState>) -> Json<DictionaryListResponse> {
    let entries = state.pronunciation.dictionary().list();
    Json(DictionaryListResponse {
        count: entries.len(),
        entries,
    })
}

/// `POST /v1/pronunciation/dictionary` — add or update a pronunciation entry.
///
/// Returns 201 Created on insert, 200 OK on update.
pub async fn upsert_entry(
    State(state): State<AppState>,
    Json(body): Json<UpsertEntryRequest>,
) -> Result<(StatusCode, Json<DictionaryEntry>), (StatusCode, Json<ProblemDetails>)> {
    if body.term.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/invalid-parameter".to_string(),
                title: "Empty Term".to_string(),
                status: 400,
                detail: "The 'term' field must not be empty".to_string(),
                instance: "/v1/pronunciation/dictionary".to_string(),
            }),
        ));
    }
    if body.replacement.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/invalid-parameter".to_string(),
                title: "Empty Replacement".to_string(),
                status: 400,
                detail: "The 'replacement' field must not be empty".to_string(),
                instance: "/v1/pronunciation/dictionary".to_string(),
            }),
        ));
    }

    let entry = DictionaryEntry {
        term: body.term,
        replacement: body.replacement,
        note: body.note,
    };
    let previous = state.pronunciation.dictionary().upsert(entry.clone());
    let status = if previous.is_some() {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((status, Json(entry)))
}

/// `DELETE /v1/pronunciation/dictionary/{term}` — remove an entry.
///
/// Returns 204 No Content on success, 404 if the term was not found.
pub async fn delete_entry(
    State(state): State<AppState>,
    Path(term): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetails>)> {
    match state.pronunciation.dictionary().remove(&term) {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/term-not-found".to_string(),
                title: "Term Not Found".to_string(),
                status: 404,
                detail: format!("Term '{term}' not found in pronunciation dictionary"),
                instance: format!("/v1/pronunciation/dictionary/{term}"),
            }),
        )),
    }
}
