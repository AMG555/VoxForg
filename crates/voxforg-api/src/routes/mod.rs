pub mod docs;
pub mod health;
pub mod metrics;
pub mod models;
pub mod pipeline;
pub mod qa;
pub mod speech;
pub mod voices;

use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build_api_router() -> Router<AppState> {
    Router::new()
        .route("/docs", get(docs::scalar_docs_html))
        .route("/openapi.json", get(docs::openapi_spec))
        .route("/metrics", get(metrics::get_metrics))
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/v1/models", get(models::list_models))
        .route("/v1/voices", get(voices::list_voices))
        .route("/v1/audio/speech", post(speech::synthesize_speech))
        .route(
            "/v1/audio/speech/stream",
            post(speech::synthesize_speech_stream),
        )
        .route("/v1/audio/speech/ws", get(speech::speech_websocket))
        .route("/v1/pipeline/execute", post(pipeline::execute_pipeline))
        .route("/v1/qa/ab-test", post(qa::run_ab_test))
}
