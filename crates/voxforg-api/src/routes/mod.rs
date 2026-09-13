pub mod benchmark;
pub mod cloning;
pub mod docs;
pub mod health;
pub mod metrics;
pub mod models;
pub mod pipeline;
pub mod pronunciation;
pub mod qa;
pub mod speech;
pub mod voice_ci;
pub mod voice_identity;
pub mod voices;
pub mod workers;

use crate::state::AppState;
use axum::{
    routing::{delete, get, post},
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
        // Pronunciation dictionary management
        .route(
            "/v1/pronunciation/dictionary",
            get(pronunciation::list_dictionary).post(pronunciation::upsert_entry),
        )
        .route(
            "/v1/pronunciation/dictionary/:term",
            delete(pronunciation::delete_entry),
        )
        // Voice identities (portable voice abstraction)
        .route(
            "/v1/voice-identities",
            get(voice_identity::list_identities).post(voice_identity::register_identity),
        )
        .route(
            "/v1/voice-identities/:id",
            get(voice_identity::get_identity).delete(voice_identity::delete_identity),
        )
        // Voice cloning and persistent profiles
        .route("/v1/voices/profiles", get(cloning::list_profiles))
        .route("/v1/voices/clone", post(cloning::clone_voice))
        .route(
            "/v1/voices/profiles/:id",
            get(cloning::get_profile).delete(cloning::delete_profile),
        )
        // Engine benchmarking
        .route("/v1/benchmark/results", get(benchmark::list_results))
        .route("/v1/benchmark/run", post(benchmark::trigger_run))
        // Voice CI/CD regression
        .route(
            "/v1/voice-ci/profiles",
            get(voice_ci::list_profiles).post(voice_ci::create_profile),
        )
        .route("/v1/voice-ci/compare", post(voice_ci::compare))
        // Distributed cluster workers
        .route("/v1/workers", get(workers::list_workers))
        .route("/v1/workers/register", post(workers::register_worker))
        .route(
            "/v1/workers/:id",
            get(workers::get_worker).delete(workers::delete_worker),
        )
        .route("/v1/workers/:id/heartbeat", post(workers::worker_heartbeat))
}
