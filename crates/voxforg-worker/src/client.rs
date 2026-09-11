use std::time::Duration;
use voxforg_core::error::{Result, VoxForgError};

use crate::types::WorkerNode;

/// Client used by coordinator to dispatch synthesis requests to remote worker nodes.
#[derive(Clone)]
pub struct WorkerClient {
    client: reqwest::Client,
}

impl Default for WorkerClient {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// Dispatch a synthesis payload to the worker node at its registered address.
    pub async fn dispatch_raw(
        &self,
        worker: &WorkerNode,
        payload: &serde_json::Value,
    ) -> Result<Vec<u8>> {
        let url = format!("{}/v1/audio/speech", worker.address.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .json(payload)
            .send()
            .await
            .map_err(|e| {
                VoxForgError::Engine(format!(
                    "Failed to connect to worker {} at {}: {}",
                    worker.worker_id, url, e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VoxForgError::Engine(format!(
                "Worker {} returned error {}: {}",
                worker.worker_id, status, body
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| {
                VoxForgError::Engine(format!(
                    "Failed to read audio stream from worker {}: {}",
                    worker.worker_id, e
                ))
            })?
            .to_vec();

        Ok(bytes)
    }
}
