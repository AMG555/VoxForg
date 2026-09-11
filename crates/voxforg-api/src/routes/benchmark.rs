//! REST endpoints for engine benchmarking.
//!
//! | Method | Path                    | Description                              |
//! |--------|-------------------------|------------------------------------------|
//! | GET    | `/v1/benchmark/results` | List latest benchmark results per engine |
//! | POST   | `/v1/benchmark/run`     | Trigger an async benchmark run           |

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use voxforg_benchmark::{BenchmarkResult, BenchmarkRunner};

use crate::state::AppState;

#[derive(Serialize)]
pub struct BenchmarkListResponse {
    pub count: usize,
    pub results: Vec<BenchmarkResult>,
}

/// `GET /v1/benchmark/results` — return all stored benchmark scorecards.
pub async fn list_results(State(state): State<AppState>) -> Json<BenchmarkListResponse> {
    let results = state.benchmark_store.list().await;
    Json(BenchmarkListResponse {
        count: results.len(),
        results,
    })
}

/// `POST /v1/benchmark/run` — trigger a full benchmark run in the background.
///
/// Returns 202 Accepted immediately; results become available via GET after
/// the run completes (typically a few seconds for local engines).
pub async fn trigger_run(State(state): State<AppState>) -> StatusCode {
    let registry = state.engine_registry.clone();
    let store = state.benchmark_store.clone();

    tokio::spawn(async move {
        let runner = BenchmarkRunner::new(registry, store);
        runner.run_all().await;
    });

    StatusCode::ACCEPTED
}
