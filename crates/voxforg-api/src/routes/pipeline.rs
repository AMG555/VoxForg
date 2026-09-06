use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use serde::Deserialize;
use voxforg_core::error::ProblemDetails;
use voxforg_core::models::PipelineDefinition;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PipelineExecuteRequest {
    pub pipeline: PipelineDefinition,
    pub input_text: Option<String>,
}

pub async fn execute_pipeline(
    State(state): State<AppState>,
    Json(payload): Json<PipelineExecuteRequest>,
) -> Result<Response, (StatusCode, Json<ProblemDetails>)> {
    if let Some(text) = &payload.input_text {
        if text.len() > 50_000 {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/input-too-large".to_string(),
                title: "Input Text Exceeds Limit".to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
                detail: "The 'input_text' parameter cannot exceed 50,000 characters".to_string(),
                instance: "/v1/pipeline/execute".to_string(),
            };
            return Err((StatusCode::BAD_REQUEST, Json(err)));
        }
    }
    let wav_bytes = state
        .pipeline_executor
        .execute(&payload.pipeline, payload.input_text)
        .await
        .map_err(|e| {
            let problem = e.to_problem_details("/v1/pipeline/execute");
            (StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(problem))
        })?;

    let _ = state.data_store.save_pipeline(&payload.pipeline).await;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static("audio/wav"))
        .body(Body::from(wav_bytes))
        .map_err(|e| {
            let err = ProblemDetails {
                problem_type: "https://voxforg.org/errors/internal".to_string(),
                title: "Internal Error".to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                detail: e.to_string(),
                instance: "/v1/pipeline/execute".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
        })?;

    Ok(response)
}
