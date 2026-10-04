use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use voxforg_core::error::ProblemDetails;
use voxforg_pipeline::checkpoint::JobCheckpointManifest;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct ListJobsResponse {
    pub jobs: Vec<JobCheckpointManifest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub job_id: String,
    pub job_type: String,
    pub title: String,
    pub chapters: Vec<CreateChapterPayload>,
}

#[derive(Debug, Deserialize)]
pub struct CreateChapterPayload {
    pub title: String,
    pub text: String,
    pub voice_id: String,
    pub speed: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct ResumeJobResponse {
    pub manifest: JobCheckpointManifest,
    pub remaining_indices: Vec<usize>,
    pub is_finished: bool,
}

fn get_work_dir(job_id: &str) -> PathBuf {
    std::env::temp_dir()
        .join("voxforg_jobs")
        .join(job_id.replace("..", "").replace("/", "").replace("\\", ""))
}

/// `GET /v1/jobs/checkpoints` — list recent job checkpoints
pub async fn list_checkpoints_handler(
    State(_state): State<AppState>,
) -> Json<ListJobsResponse> {
    let base_dir = std::env::temp_dir().join("voxforg_jobs");
    let mut jobs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Ok(Some(manifest)) = JobCheckpointManifest::load_from_dir(&entry.path()) {
                    jobs.push(manifest);
                }
            }
        }
    }

    Json(ListJobsResponse { jobs })
}

/// `POST /v1/jobs/checkpoint` — create or register an audiobook/longform render job
pub async fn create_checkpoint_handler(
    State(_state): State<AppState>,
    Json(body): Json<CreateJobRequest>,
) -> Result<(StatusCode, Json<JobCheckpointManifest>), (StatusCode, Json<ProblemDetails>)> {
    let work_dir = get_work_dir(&body.job_id);

    let mut manifest = JobCheckpointManifest::new(body.job_id, body.job_type, body.title);
    for ch in body.chapters {
        manifest.add_chapter(ch.title, ch.text, ch.voice_id, ch.speed.unwrap_or(1.0));
    }

    manifest.save_to_dir(&work_dir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/io-error".to_string(),
                title: "Failed to Save Checkpoint".to_string(),
                status: 500,
                detail: e.to_string(),
                instance: "/v1/jobs/checkpoint".to_string(),
            }),
        )
    })?;

    Ok((StatusCode::CREATED, Json(manifest)))
}

/// `POST /v1/jobs/{id}/resume` — inspect and resume an interrupted render job
pub async fn resume_job_handler(
    State(_state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<ResumeJobResponse>, (StatusCode, Json<ProblemDetails>)> {
    let work_dir = get_work_dir(&job_id);

    let manifest = JobCheckpointManifest::load_from_dir(&work_dir)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/io-error".to_string(),
                    title: "Failed to Read Checkpoint".to_string(),
                    status: 500,
                    detail: e.to_string(),
                    instance: format!("/v1/jobs/{job_id}/resume"),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/not-found".to_string(),
                    title: "Job Checkpoint Not Found".to_string(),
                    status: 404,
                    detail: format!("No resume manifest found for job ID '{job_id}'"),
                    instance: format!("/v1/jobs/{job_id}/resume"),
                }),
            )
        })?;

    let remaining_indices = manifest.get_remaining_indices();
    let is_finished = manifest.is_finished();

    Ok(Json(ResumeJobResponse {
        manifest,
        remaining_indices,
        is_finished,
    }))
}
