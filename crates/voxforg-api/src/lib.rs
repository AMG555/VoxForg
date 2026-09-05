pub mod middleware;
pub mod routes;
pub mod state;

use axum::{middleware::from_fn, middleware::from_fn_with_state, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub use state::AppState;

pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    routes::build_api_router()
        .layer(from_fn_with_state(state.clone(), middleware::auth_middleware))
        .layer(from_fn(middleware::security_headers))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use voxforg_core::store::memory::MemoryStore;
    use voxforg_engine::{EngineRegistry, MockTtsEngine};
    use voxforg_hardware::HardwareProbe;

    #[tokio::test]
    async fn test_health_route() {
        let registry = Arc::new(EngineRegistry::new());
        let store = Arc::new(MemoryStore::new());
        let hardware = HardwareProbe::probe();
        let state = AppState::new(registry, store, hardware, None);

        let app = create_app(state);
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get("x-content-type-options").unwrap(),
            "nosniff"
        );
    }

    #[tokio::test]
    async fn test_openai_speech_endpoint() {
        let registry = Arc::new(EngineRegistry::new());
        let mock_engine = Arc::new(MockTtsEngine::new(24000));
        registry.register(mock_engine).await;

        let store = Arc::new(MemoryStore::new());
        let hardware = HardwareProbe::probe();
        let state = AppState::new(registry, store, hardware, None);

        let app = create_app(state);
        let payload = serde_json::json!({
            "model": "mock-tts",
            "input": "Testing OpenAI speech route",
            "voice": "mock-en-female",
            "response_format": "wav",
            "speed": 1.0
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/speech")
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "audio/wav"
        );
    }
}
