use std::collections::HashMap;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "voxforg",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn readiness_check(State(state): State<AppState>) -> Response {
    let engine_ids = state.engine_registry.list_engines().await;
    let mut health_map = HashMap::new();
    let mut healthy_count = 0;
    let total_count = engine_ids.len();

    let check_futures = engine_ids.iter().map(|id| {
        let registry = state.engine_registry.clone();
        let id = id.clone();
        async move {
            if let Some(engine) = registry.get(&id).await {
                let status = match engine.health_check().await {
                    Ok(true) => "healthy",
                    Ok(false) => "circuit_open",
                    Err(_) => "unreachable",
                };
                (id, Some(status))
            } else {
                (id, None)
            }
        }
    });

    let results = futures::future::join_all(check_futures).await;
    for (id, status) in results {
        if let Some(s) = status {
            if s == "healthy" {
                healthy_count += 1;
            }
            health_map.insert(id, s);
        }
    }

    let overall_status = if total_count == 0 || healthy_count == total_count {
        "ready"
    } else if healthy_count > 0 {
        "degraded"
    } else {
        "unhealthy"
    };

    let status_code = if overall_status == "unhealthy" && total_count > 0 {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    let body = json!({
        "status": overall_status,
        "healthy_engines": healthy_count,
        "total_engines": total_count,
        "registered_engines": engine_ids,
        "engine_health": health_map,
        "hardware_tier": state.hardware_profile.assigned_tier.to_string(),
        "arch": state.hardware_profile.arch,
        "os": state.hardware_profile.os
    });

    (status_code, Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use voxforg_core::store::memory::MemoryStore;
    use voxforg_engine::{EngineRegistry, MockTtsEngine};
    use voxforg_hardware::HardwareProbe;

    #[tokio::test]
    async fn test_health_check_payload() {
        let res = health_check().await;
        assert_eq!(res.0["status"], "healthy");
        assert_eq!(res.0["service"], "voxforg");
        assert!(!res.0["version"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_readiness_check_concurrent_evaluation() {
        let registry = Arc::new(EngineRegistry::new());
        let mock_engine = Arc::new(MockTtsEngine::new(24000));
        registry.register(mock_engine).await;

        let store = Arc::new(MemoryStore::new());
        let hardware = HardwareProbe::probe();
        let state = AppState::new(registry, store, hardware, None);

        let resp = readiness_check(State(state)).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
