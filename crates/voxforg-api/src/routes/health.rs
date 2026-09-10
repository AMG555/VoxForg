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
        "version": "0.1.0"
    }))
}

pub async fn readiness_check(State(state): State<AppState>) -> Response {
    let engine_ids = state.engine_registry.list_engines().await;
    let mut health_map = HashMap::new();
    let mut healthy_count = 0;
    let total_count = engine_ids.len();

    for id in &engine_ids {
        if let Some(engine) = state.engine_registry.get(id).await {
            match engine.health_check().await {
                Ok(true) => {
                    health_map.insert(id.clone(), "healthy");
                    healthy_count += 1;
                }
                Ok(false) => {
                    health_map.insert(id.clone(), "circuit_open");
                }
                Err(_) => {
                    health_map.insert(id.clone(), "unreachable");
                }
            }
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
