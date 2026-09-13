pub mod metrics;
pub mod middleware;
pub mod routes;
pub mod state;

use axum::{
    extract::DefaultBodyLimit, middleware::from_fn, middleware::from_fn_with_state, Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub use state::AppState;

pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    routes::build_api_router()
        .layer(from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ))
        .layer(from_fn(middleware::security_headers))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::request_id_and_metrics,
        ))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use std::sync::Arc;
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
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "pipeline": pipeline,
                    "input_text": null
                }))
                .unwrap(),
            ))
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
        let docs_req = Request::builder().uri("/docs").body(Body::empty()).unwrap();
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
        let req = Request::builder().uri("/docs").body(Body::empty()).unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );
        assert_eq!(res.headers().get("x-frame-options").unwrap(), "SAMEORIGIN");
        let csp = res
            .headers()
            .get("content-security-policy")
            .unwrap()
            .to_str()
            .unwrap();
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
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["openapi"], "3.1.0");
        assert_eq!(json["info"]["title"], "VoxForg Neural Audio Engine API");
        assert!(json["paths"]["/v1/audio/speech"].is_object());
        assert!(json["paths"]["/v1/pipeline/execute"].is_object());
        assert!(json["paths"]["/v1/qa/ab-test"].is_object());
        assert!(json["components"]["schemas"]["PipelineDefinition"].is_object());
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let state = setup_test_state(None).await;
        let app = create_app(state);
        let req = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("text/plain"));

        let body = res.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("voxforg_active_requests"));
        assert!(text.contains("voxforg_synthesis_total"));
        assert!(text.contains("voxforg_cache_hits_total"));
    }

    #[tokio::test]
    async fn test_speech_input_limit_and_request_id() {
        let state = setup_test_state(None).await;
        let app = create_app(state);

        // Input exceeding 10,000 chars should return 422
        let huge_text = "a".repeat(10_001);
        let payload = serde_json::json!({
            "model": "mock-tts",
            "voice": "mock-en-female",
            "input": huge_text
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/speech")
            .header("content-type", "application/json")
            .header("x-request-id", "test-trace-12345")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            res.headers().get("x-request-id").unwrap(),
            "test-trace-12345"
        );
    }

    #[tokio::test]
    async fn test_raw_pcm_synthesis_format() {
        let state = setup_test_state(None).await;
        let app = create_app(state);

        let payload = serde_json::json!({
            "model": "mock-tts",
            "voice": "mock-en-female",
            "input": "Testing raw PCM format delivery",
            "response_format": "pcm"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/speech")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-type").unwrap(), "audio/pcm");

        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert!(!body.is_empty());
        // Raw PCM has no "RIFF" WAV header
        assert!(!body.starts_with(b"RIFF"));
    }

    #[tokio::test]
    async fn test_readiness_deep_health_probing() {
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
        assert_eq!(json["engine_health"]["mock-tts"], "healthy");
        assert_eq!(json["healthy_engines"], 1);
    }

    #[tokio::test]
    async fn test_streaming_speech_endpoint() {
        let state = setup_test_state(None).await;
        let app = create_app(state);

        let payload = serde_json::json!({
            "model": "mock-tts",
            "voice": "mock-en-female",
            "input": "Testing real-time chunked transfer encoding stream"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/speech/stream")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-type").unwrap(), "audio/pcm");

        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert!(!body.is_empty());
    }

    #[tokio::test]
    async fn test_voice_identity_crud() {
        let state = setup_test_state(None).await;
        let app = create_app(state);

        // 1. List initial identities (contains defaults)
        let req = Request::builder()
            .uri("/v1/voice-identities")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["count"].as_u64().unwrap() >= 2);

        // 2. Create custom voice identity
        let custom_identity = serde_json::json!({
            "id": "narrator-en",
            "display_name": "Studio Narrator",
            "quality": "high",
            "style": "narration",
            "accent": "en-US",
            "gender": "male",
            "engine_mappings": [
                { "engine_id": "mock-tts", "voice_id": "mock-en-male", "priority": 1 }
            ]
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/voice-identities")
            .header("content-type", "application/json")
            .body(Body::from(custom_identity.to_string()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        // 3. Fetch single identity
        let req = Request::builder()
            .uri("/v1/voice-identities/narrator-en")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["display_name"], "Studio Narrator");

        // 4. Delete identity
        let req = Request::builder()
            .method("DELETE")
            .uri("/v1/voice-identities/narrator-en")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_speech_with_voice_identity_resolution() {
        let state = setup_test_state(None).await;
        let app = create_app(state);

        // "default-female" is registered by default with mock-tts fallback
        let payload = serde_json::json!({
            "model": "auto",
            "voice": "default-female",
            "input": "Portable voice identity test through speech endpoint"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/speech")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-type").unwrap(), "audio/wav");
        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert!(!body.is_empty());
    }

    #[tokio::test]
    async fn test_worker_registration_and_lifecycle() {
        let state = setup_test_state(None).await;
        let app = create_app(state);

        let worker_id = Uuid::new_v4();
        let payload = serde_json::json!({
            "worker_id": worker_id,
            "hardware": HardwareProbe::probe(),
            "models": ["qwen3-tts", "piper"],
            "capacity": 4,
            "address": "http://127.0.0.1:9099"
        });

        // 1. Register worker
        let req = Request::builder()
            .method("POST")
            .uri("/v1/workers/register")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        // 2. List workers
        let req = Request::builder()
            .uri("/v1/workers")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["count"], 1);

        // 3. Get single worker
        let req = Request::builder()
            .uri(format!("/v1/workers/{worker_id}"))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // 4. Heartbeat
        let req = Request::builder()
            .method("POST")
            .uri(format!("/v1/workers/{worker_id}/heartbeat"))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // 5. Deregister worker
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("/v1/workers/{worker_id}"))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_speech_explicit_worker_missing() {
        let state = setup_test_state(None).await;
        let app = create_app(state);

        let payload = serde_json::json!({
            "model": "worker:non-existent-cluster-engine",
            "voice": "mock-en-female",
            "input": "Testing explicit worker node routing"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/speech")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["title"], "No Worker Available");
    }

    #[tokio::test]
    async fn test_voice_cloning_and_profile_crud() {
        let state = setup_test_state(None).await;
        let app = create_app(state);

        // 1. List initial profiles (has seeded base profiles)
        let req = Request::builder()
            .uri("/v1/voices/profiles")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["count"].as_u64().unwrap() >= 2);

        // 2. Clone a new voice using mock-tts engine
        let fake_pcm: Vec<i16> = (0..1600).map(|i| ((i % 50) * 200) as i16).collect();
        let bytes: Vec<u8> = fake_pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
        let b64 = hex::encode(bytes);

        let clone_payload = serde_json::json!({
            "name": "David Reporter",
            "engine_id": "mock-tts",
            "reference_audio_base64": b64,
            "language": "en-US",
            "description": "Studio investigative reporter voice",
            "gender": "male"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/voices/clone")
            .header("content-type", "application/json")
            .body(Body::from(clone_payload.to_string()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let profile_id = created["id"].as_str().unwrap();
        assert_eq!(created["name"], "David Reporter");

        // 3. Retrieve created profile
        let req = Request::builder()
            .uri(format!("/v1/voices/profiles/{profile_id}"))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // 4. Synthesize speech using the cloned profile ID
        let synth_payload = serde_json::json!({
            "model": "auto",
            "voice": profile_id,
            "input": "Synthesizing audio conditioned on cloned voice profile"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/speech")
            .header("content-type", "application/json")
            .body(Body::from(synth_payload.to_string()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-type").unwrap(), "audio/wav");
        let audio_bytes = res.into_body().collect().await.unwrap().to_bytes();
        assert!(!audio_bytes.is_empty());

        // 5. Delete profile
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("/v1/voices/profiles/{profile_id}"))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_asr_transcription_endpoints() {
        let state = setup_test_state(None).await;
        let app = create_app(state);

        // 1. List registered ASR engines
        let req = Request::builder()
            .uri("/v1/asr/engines")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["count"].as_u64().unwrap() >= 2);

        // 2. Synthesize test PCM audio
        let fake_pcm: Vec<i16> = (0..16000).map(|i| ((i % 50) * 150) as i16).collect();
        let bytes: Vec<u8> = fake_pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
        let hex_audio = hex::encode(bytes);

        // 3. Simple JSON transcription with default engine
        let payload = serde_json::json!({
            "audio_base64": hex_audio,
            "prompt": "Testing speech recognition pipeline",
            "response_format": "json"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/transcriptions")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let res_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(!res_json["text"].as_str().unwrap().is_empty());

        // 4. Verbose JSON with word and segment timestamps
        let payload = serde_json::json!({
            "audio_base64": hex_audio,
            "model": "mock-asr",
            "response_format": "verbose_json",
            "timestamp_granularities": ["word", "segment"]
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/transcriptions")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let verbose: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(verbose["task"], "transcribe");
        assert!(verbose["duration_seconds"].as_f64().unwrap() > 0.0);
        assert!(!verbose["segments"].as_array().unwrap().is_empty());
        assert!(!verbose["words"].as_array().unwrap().is_empty());

        // 5. SubRip (.srt) subtitle formatting
        let payload = serde_json::json!({
            "audio_base64": hex_audio,
            "response_format": "srt"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/transcriptions")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let srt_text = String::from_utf8(body.to_vec()).unwrap();
        assert!(srt_text.contains("-->"));

        // 6. Non-existent engine returns 404
        let payload = serde_json::json!({
            "audio_base64": hex_audio,
            "model": "non-existent-asr"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/v1/audio/transcriptions")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_catalog_management_endpoints() {
        let state = setup_test_state(None).await;
        let app = create_app(state);

        // 1. List all catalog models
        let req = Request::builder()
            .uri("/v1/catalog/models")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["count"].as_u64().unwrap() >= 6);

        // 2. Filter by type=tts
        let req = Request::builder()
            .uri("/v1/catalog/models?type=tts")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        for m in json["models"].as_array().unwrap() {
            assert_eq!(m["model_type"], "tts");
        }

        // 3. Install kokoro-v0_19
        let req = Request::builder()
            .method("POST")
            .uri("/v1/catalog/models/kokoro-v0_19/install")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "installed");
        assert!(json["local_path"].as_str().is_some());

        // 4. Retrieve single model
        let req = Request::builder()
            .uri("/v1/catalog/models/kokoro-v0_19")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "installed");

        // 5. Uninstall model
        let req = Request::builder()
            .method("DELETE")
            .uri("/v1/catalog/models/kokoro-v0_19")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "available");

        // 6. Non-existent model returns 404
        let req = Request::builder()
            .uri("/v1/catalog/models/non-existent-model")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
