pub mod middleware;
pub mod routes;
pub mod state;

use axum::{extract::DefaultBodyLimit, middleware::from_fn, middleware::from_fn_with_state, Router};
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
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
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
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use uuid::Uuid;
    use voxforg_core::models::{NodeType, PipelineDefinition, PipelineNode};
    use voxforg_core::store::memory::MemoryStore;
    use voxforg_engine::{EngineRegistry, MockTtsEngine};
    use voxforg_hardware::HardwareProbe;

    async fn setup_test_state(api_key: Option<String>) -> AppState {
        let registry = Arc::new(EngineRegistry::new());
        let mock_engine = Arc::new(MockTtsEngine::new(24000));
        registry.register(mock_engine).await;

        let store = Arc::new(MemoryStore::new());
        let hardware = HardwareProbe::probe();
        AppState::new(registry, store, hardware, api_key)
    }

    #[tokio::test]
    async fn test_health_route() {
        let state = setup_test_state(None).await;
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
        assert_eq!(res.headers().get("x-frame-options").unwrap(), "DENY");
    }

    #[tokio::test]
    async fn test_readiness_route() {
        let state = setup_test_state(None).await;
        let app = create_app(state);
        let req = Request::builder()
            .uri("/health/ready")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ready");
        assert!(json["registered_engines"].is_array());
    }

    #[tokio::test]
    async fn test_models_list_endpoint() {
        let state = setup_test_state(None).await;
        let app = create_app(state);
        let req = Request::builder()
            .uri("/v1/models")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["object"], "list");
        assert!(!json["data"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_voices_list_endpoint() {
        let state = setup_test_state(None).await;
        let app = create_app(state);
        let req = Request::builder()
            .uri("/v1/voices")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["total"].as_u64().unwrap() >= 2);
    }

    #[tokio::test]
    async fn test_openai_speech_endpoint() {
        let state = setup_test_state(None).await;
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
        assert_eq!(res.headers().get("content-type").unwrap(), "audio/wav");
    }

    #[tokio::test]
    async fn test_openai_speech_empty_input_fails() {
        let state = setup_test_state(None).await;
        let app = create_app(state);
        let payload = serde_json::json!({
            "model": "mock-tts",
            "input": "   ",
            "voice": "mock-en-female"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/speech")
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_openai_speech_missing_voice_fails() {
        let state = setup_test_state(None).await;
        let app = create_app(state);
        let payload = serde_json::json!({
            "model": "non-existent-engine",
            "input": "Valid text input",
            "voice": "non-existent-voice"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/speech")
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_qa_ab_test_endpoint() {
        let state = setup_test_state(None).await;
        let app = create_app(state);
        let payload = serde_json::json!({
            "name": "API QA Evaluation",
            "text": "Automated evaluation comparison.",
            "variant_a": {
                "text": "",
                "voice_id": "mock-en-female",
                "speed": 1.0,
                "pitch": 0.0,
                "format": "wav"
            },
            "variant_b": {
                "text": "",
                "voice_id": "mock-en-male",
                "speed": 1.2,
                "pitch": 2.0,
                "format": "wav"
            }
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/qa/ab-test")
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["scenario_name"], "API QA Evaluation");
        assert!(json["variant_a"]["latency_ms"].is_f64());
        assert!(json["variant_b"]["latency_ms"].is_f64());
        assert!(!json["recommended_variant"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_qa_ab_test_empty_text_rejected() {
        let state = setup_test_state(None).await;
        let app = create_app(state);
        let payload = serde_json::json!({
            "name": "Empty Test",
            "text": "  ",
            "variant_a": { "text": "", "voice_id": "mock-en-female" },
            "variant_b": { "text": "", "voice_id": "mock-en-male" }
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/qa/ab-test")
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_pipeline_execute_endpoint() {
        let state = setup_test_state(None).await;
        let app = create_app(state);

        let pipeline = PipelineDefinition {
            id: Uuid::new_v4(),
            name: "API Pipeline Test".to_string(),
            description: None,
            nodes: vec![
                PipelineNode {
                    id: "p1".to_string(),
                    name: "In".to_string(),
                    node_type: NodeType::TextInput,
                    params: serde_json::json!({ "text": "Testing pipeline execution via API" }),
                    position: None,
                },
                PipelineNode {
                    id: "p2".to_string(),
                    name: "Synth".to_string(),
                    node_type: NodeType::Synthesizer,
                    params: serde_json::json!({}),
                    position: None,
                },
                PipelineNode {
                    id: "p3".to_string(),
                    name: "Out".to_string(),
                    node_type: NodeType::OutputSink,
                    params: serde_json::json!({}),
                    position: None,
                },
            ],
            edges: vec![
                voxforg_core::models::PipelineEdge {
                    id: "e1".to_string(),
                    from_node: "p1".to_string(),
                    to_node: "p2".to_string(),
                    from_port: None,
                    to_port: None,
                },
                voxforg_core::models::PipelineEdge {
                    id: "e2".to_string(),
                    from_node: "p2".to_string(),
                    to_node: "p3".to_string(),
                    from_port: None,
                    to_port: None,
                },
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let req = Request::builder()
            .method("POST")
            .uri("/v1/pipeline/execute")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&serde_json::json!({
                "pipeline": pipeline,
                "input_text": null
            })).unwrap()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-type").unwrap(), "audio/wav");
    }

    #[tokio::test]
    async fn test_auth_middleware_flow() {
        let state = setup_test_state(Some("secret-token-123".to_string())).await;
        let app = create_app(state);

        // 1. Missing auth header -> 401 Unauthorized
        let unauth_req = Request::builder()
            .uri("/v1/models")
            .body(Body::empty())
            .unwrap();
        let res1 = app.clone().oneshot(unauth_req).await.unwrap();
        assert_eq!(res1.status(), StatusCode::UNAUTHORIZED);

        // 2. Invalid auth token -> 401 Unauthorized
        let bad_token_req = Request::builder()
            .uri("/v1/models")
            .header("Authorization", "Bearer wrong-token")
            .body(Body::empty())
            .unwrap();
        let res2 = app.clone().oneshot(bad_token_req).await.unwrap();
        assert_eq!(res2.status(), StatusCode::UNAUTHORIZED);

        // 2b. Partial prefix match (timing attack simulation) -> 401 Unauthorized
        let near_token_req = Request::builder()
            .uri("/v1/models")
            .header("Authorization", "Bearer secret-token-124")
            .body(Body::empty())
            .unwrap();
        let res2b = app.clone().oneshot(near_token_req).await.unwrap();
        assert_eq!(res2b.status(), StatusCode::UNAUTHORIZED);

        // 2c. Substring match (shorter length) -> 401 Unauthorized
        let short_token_req = Request::builder()
            .uri("/v1/models")
            .header("Authorization", "Bearer secret-token")
            .body(Body::empty())
            .unwrap();
        let res2c = app.clone().oneshot(short_token_req).await.unwrap();
        assert_eq!(res2c.status(), StatusCode::UNAUTHORIZED);

        // 2d. Superstring match (longer length) -> 401 Unauthorized
        let long_token_req = Request::builder()
            .uri("/v1/models")
            .header("Authorization", "Bearer secret-token-123-extended")
            .body(Body::empty())
            .unwrap();
        let res2d = app.clone().oneshot(long_token_req).await.unwrap();
        assert_eq!(res2d.status(), StatusCode::UNAUTHORIZED);

        // 3. Valid token -> 200 OK
        let valid_token_req = Request::builder()
            .uri("/v1/models")
            .header("Authorization", "Bearer secret-token-123")
            .body(Body::empty())
            .unwrap();
        let res3 = app.clone().oneshot(valid_token_req).await.unwrap();
        assert_eq!(res3.status(), StatusCode::OK);

        // 4. Public route (/health) bypasses auth -> 200 OK
        let health_req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let res4 = app.clone().oneshot(health_req).await.unwrap();
        assert_eq!(res4.status(), StatusCode::OK);

        // 5. Public routes (/docs and /openapi.json) bypass auth -> 200 OK
        let docs_req = Request::builder()
            .uri("/docs")
            .body(Body::empty())
            .unwrap();
        let res5 = app.clone().oneshot(docs_req).await.unwrap();
        assert_eq!(res5.status(), StatusCode::OK);

        let openapi_req = Request::builder()
            .uri("/openapi.json")
            .body(Body::empty())
            .unwrap();
        let res6 = app.oneshot(openapi_req).await.unwrap();
        assert_eq!(res6.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_scalar_docs_html_endpoint() {
        let state = setup_test_state(None).await;
        let app = create_app(state);
        let req = Request::builder()
            .uri("/docs")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );
        assert_eq!(res.headers().get("x-frame-options").unwrap(), "SAMEORIGIN");
        let csp = res.headers().get("content-security-policy").unwrap().to_str().unwrap();
        assert!(csp.contains("https://cdn.jsdelivr.net"));

        let body = res.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("VoxForg API Reference"));
        assert!(html.contains("@scalar/api-reference"));
    }

    #[tokio::test]
    async fn test_openapi_spec_endpoint() {
        let state = setup_test_state(None).await;
        let app = create_app(state);
        let req = Request::builder()
            .uri("/openapi.json")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-type").unwrap(), "application/json");

        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["openapi"], "3.1.0");
        assert_eq!(json["info"]["title"], "VoxForg Neural Audio Engine API");
        assert!(json["paths"]["/v1/audio/speech"].is_object());
        assert!(json["paths"]["/v1/pipeline/execute"].is_object());
        assert!(json["paths"]["/v1/qa/ab-test"].is_object());
        assert!(json["components"]["schemas"]["PipelineDefinition"].is_object());
    }
}
