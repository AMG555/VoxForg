use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use voxforg_hardware::HardwareProfile;

/// Status lifecycle of a registered worker node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    #[default]
    Ready,
    Busy,
    Draining,
    Offline,
}

/// Self-registration payload submitted by an autonomous worker node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRegistration {
    #[serde(default = "Uuid::new_v4")]
    pub worker_id: Uuid,
    pub hardware: HardwareProfile,
    pub models: Vec<String>,
    #[serde(default = "default_capacity")]
    pub capacity: u32,
    pub address: String,
}

fn default_capacity() -> u32 {
    4
}

/// Tracked state of a cluster worker node inside the coordinator pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerNode {
    pub worker_id: Uuid,
    pub hardware: HardwareProfile,
    pub models: Vec<String>,
    pub capacity: u32,
    pub address: String,
    pub status: WorkerStatus,
    pub active_jobs: u32,
    pub total_jobs_processed: u64,
    pub registered_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

impl WorkerNode {
    pub fn from_registration(reg: WorkerRegistration) -> Self {
        let now = Utc::now();
        Self {
            worker_id: reg.worker_id,
            hardware: reg.hardware,
            models: reg.models,
            capacity: reg.capacity,
            address: reg.address,
            status: WorkerStatus::Ready,
            active_jobs: 0,
            total_jobs_processed: 0,
            registered_at: now,
            last_heartbeat: now,
        }
    }

    /// Check if this worker can accept a job for the given engine/model ID.
    pub fn can_accept(&self, model: &str) -> bool {
        (self.status == WorkerStatus::Ready || self.status == WorkerStatus::Busy)
            && self.active_jobs < self.capacity
            && (model == "auto" || self.models.iter().any(|m| m == model))
    }
}
