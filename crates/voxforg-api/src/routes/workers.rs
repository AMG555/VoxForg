//! REST endpoints for distributed worker node registration and orchestration.
//!
//! | Method | Path                            | Description                           |
//! |--------|---------------------------------|---------------------------------------|
//! | GET    | `/v1/workers`                   | List all registered cluster workers   |
//! | POST   | `/v1/workers/register`          | Register or update a worker node      |
//! | GET    | `/v1/workers/{id}`              | Get worker details by UUID            |
//! | DELETE | `/v1/workers/{id}`              | Deregister a worker node              |
//! | POST   | `/v1/workers/{id}/heartbeat`    | Report worker heartbeat / liveness    |

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;
use voxforg_core::error::ProblemDetails;
use voxforg_worker::{WorkerNode, WorkerRegistration};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct WorkerListResponse {
    pub count: usize,
    pub workers: Vec<WorkerNode>,
}

/// `GET /v1/workers` — list all cluster worker nodes.
pub async fn list_workers(State(state): State<AppState>) -> Json<WorkerListResponse> {
    let workers = state.worker_pool.list().await;
    Json(WorkerListResponse {
        count: workers.len(),
        workers,
    })
}

/// `POST /v1/workers/register` — self-register an autonomous worker node.
pub async fn register_worker(
    State(state): State<AppState>,
    Json(body): Json<WorkerRegistration>,
) -> Result<(StatusCode, Json<WorkerNode>), (StatusCode, Json<ProblemDetails>)> {
    if body.address.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/invalid-parameter".to_string(),
                title: "Empty Worker Address".to_string(),
                status: 400,
                detail: "The worker 'address' field cannot be empty".to_string(),
                instance: "/v1/workers/register".to_string(),
            }),
        ));
    }

    if body.capacity == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/invalid-parameter".to_string(),
                title: "Invalid Capacity".to_string(),
                status: 400,
                detail: "Worker 'capacity' must be at least 1".to_string(),
                instance: "/v1/workers/register".to_string(),
            }),
        ));
    }

    let existing = state.worker_pool.get(&body.worker_id).await;
    let node = state.worker_pool.register(body).await;

    let status = if existing.is_some() {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };

    Ok((status, Json(node)))
}

/// `GET /v1/workers/{id}` — get details of a specific worker node.
pub async fn get_worker(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkerNode>, (StatusCode, Json<ProblemDetails>)> {
    match state.worker_pool.get(&id).await {
        Some(node) => Ok(Json(node)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/worker-not-found".to_string(),
                title: "Worker Not Found".to_string(),
                status: 404,
                detail: format!("Worker '{id}' not found in cluster pool"),
                instance: format!("/v1/workers/{id}"),
            }),
        )),
    }
}

/// `DELETE /v1/workers/{id}` — deregister a worker node from the cluster.
pub async fn delete_worker(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetails>)> {
    match state.worker_pool.deregister(&id).await {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/worker-not-found".to_string(),
                title: "Worker Not Found".to_string(),
                status: 404,
                detail: format!("Worker '{id}' not found in cluster pool"),
                instance: format!("/v1/workers/{id}"),
            }),
        )),
    }
}

/// `POST /v1/workers/{id}/heartbeat` — report periodic worker liveness.
pub async fn worker_heartbeat(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkerNode>, (StatusCode, Json<ProblemDetails>)> {
    match state.worker_pool.heartbeat(&id).await {
        Ok(node) => Ok(Json(node)),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/worker-not-found".to_string(),
                title: "Worker Not Found".to_string(),
                status: 404,
                detail: format!("Worker '{id}' not found in cluster pool"),
                instance: format!("/v1/workers/{id}/heartbeat"),
            }),
        )),
    }
}
