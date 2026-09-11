use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use voxforg_core::error::{Result, VoxForgError};

use crate::types::{WorkerNode, WorkerRegistration, WorkerStatus};

/// Coordinator-side pool managing distributed worker registrations,
/// health status, load distribution, and lease tracking.
#[derive(Clone, Default)]
pub struct WorkerPool {
    workers: Arc<RwLock<HashMap<Uuid, WorkerNode>>>,
}

impl WorkerPool {
    pub fn new() -> Self {
        Self {
            workers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new worker or update an existing worker node.
    pub async fn register(&self, reg: WorkerRegistration) -> WorkerNode {
        let node = WorkerNode::from_registration(reg);
        let mut lock = self.workers.write().await;
        lock.insert(node.worker_id, node.clone());
        node
    }

    /// Record a heartbeat from a worker, updating its last_heartbeat timestamp
    /// and restoring status to Ready if it was marked Offline.
    pub async fn heartbeat(&self, id: &Uuid) -> Result<WorkerNode> {
        let mut lock = self.workers.write().await;
        if let Some(node) = lock.get_mut(id) {
            node.last_heartbeat = Utc::now();
            if node.status == WorkerStatus::Offline {
                node.status = if node.active_jobs >= node.capacity {
                    WorkerStatus::Busy
                } else {
                    WorkerStatus::Ready
                };
            }
            Ok(node.clone())
        } else {
            Err(VoxForgError::Storage(format!(
                "Worker '{id}' not found in pool"
            )))
        }
    }

    /// Deregister / remove a worker node from the pool.
    pub async fn deregister(&self, id: &Uuid) -> Option<WorkerNode> {
        let mut lock = self.workers.write().await;
        lock.remove(id)
    }

    /// Retrieve a worker by ID.
    pub async fn get(&self, id: &Uuid) -> Option<WorkerNode> {
        let lock = self.workers.read().await;
        lock.get(id).cloned()
    }

    /// List all registered workers sorted by ID.
    pub async fn list(&self) -> Vec<WorkerNode> {
        let lock = self.workers.read().await;
        let mut list: Vec<_> = lock.values().cloned().collect();
        list.sort_by_key(|w| w.worker_id);
        list
    }

    /// Select the best-fit available worker for a given model.
    /// Chooses the worker with the lowest active load ratio (`active_jobs as f32 / capacity`).
    pub async fn select_worker(&self, model: &str) -> Option<WorkerNode> {
        let lock = self.workers.read().await;
        lock.values()
            .filter(|w| w.can_accept(model))
            .min_by(|a, b| {
                let load_a = a.active_jobs as f32 / a.capacity as f32;
                let load_b = b.active_jobs as f32 / b.capacity as f32;
                load_a
                    .partial_cmp(&load_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    /// Acquire a job lease on a worker node.
    pub async fn acquire_lease(&self, id: &Uuid) -> Result<()> {
        let mut lock = self.workers.write().await;
        if let Some(node) = lock.get_mut(id) {
            if node.active_jobs >= node.capacity {
                return Err(VoxForgError::RateLimited {
                    retry_after_secs: 1,
                });
            }
            node.active_jobs += 1;
            if node.active_jobs >= node.capacity {
                node.status = WorkerStatus::Busy;
            }
            Ok(())
        } else {
            Err(VoxForgError::Storage(format!("Worker '{id}' not found")))
        }
    }

    /// Release a job lease on a worker node and increment total processed counter.
    pub async fn release_lease(&self, id: &Uuid) -> Result<()> {
        let mut lock = self.workers.write().await;
        if let Some(node) = lock.get_mut(id) {
            node.active_jobs = node.active_jobs.saturating_sub(1);
            node.total_jobs_processed += 1;
            if node.status == WorkerStatus::Busy && node.active_jobs < node.capacity {
                node.status = WorkerStatus::Ready;
            }
            Ok(())
        } else {
            Err(VoxForgError::Storage(format!("Worker '{id}' not found")))
        }
    }

    /// Scan workers and transition any node whose heartbeat is older than
    /// `max_idle_secs` to `WorkerStatus::Offline`. Returns number of staled nodes.
    pub async fn mark_stale_workers(&self, max_idle_secs: i64) -> usize {
        let now = Utc::now();
        let mut count = 0;
        let mut lock = self.workers.write().await;
        for node in lock.values_mut() {
            if node.status != WorkerStatus::Offline
                && (now - node.last_heartbeat).num_seconds() >= max_idle_secs
            {
                node.status = WorkerStatus::Offline;
                count += 1;
            }
        }
        count
    }

    #[cfg(test)]
    pub async fn set_last_heartbeat_for_test(&self, id: &Uuid, time: chrono::DateTime<Utc>) {
        let mut lock = self.workers.write().await;
        if let Some(node) = lock.get_mut(id) {
            node.last_heartbeat = time;
        }
    }
}
