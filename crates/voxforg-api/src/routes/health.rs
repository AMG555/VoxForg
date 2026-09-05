use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use crate::state::AppState;

pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "voxforg",
        "version": "0.1.0"
    }))
}

pub async fn readiness_check(State(state): State<AppState>) -> Json<Value> {
    let engines = state.engine_registry.list_engines().await;
    Json(json!({
        "status": "ready",
        "hardware_tier": state.hardware_profile.assigned_tier.to_string(),
        "registered_engines": engines,
        "arch": state.hardware_profile.arch,
        "os": state.hardware_profile.os
    }))
}
