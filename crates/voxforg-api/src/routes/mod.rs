pub mod health;
pub mod models;
pub mod pipeline;
pub mod qa;
pub mod speech;
pub mod voices;

use axum::{
    routing::{get, post},
    Router,
};
use crate::state::AppState;

pub fn build_api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/v1/models", get(models::list_models))
        .route("/v1/voices", get(voices::list_voices))
        .route("/v1/audio/speech", post(speech::synthesize_speech))
        .route("/v1/pipeline/execute", post(pipeline::execute_pipeline))
        .route("/v1/qa/ab-test", post(qa::run_ab_test))
}
