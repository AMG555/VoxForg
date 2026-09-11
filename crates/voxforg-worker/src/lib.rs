pub mod client;
pub mod pool;
pub mod types;

pub use client::WorkerClient;
pub use pool::WorkerPool;
pub use types::{WorkerNode, WorkerRegistration, WorkerStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use voxforg_hardware::HardwareProfile;

    fn mock_hardware() -> HardwareProfile {
        HardwareProfile {
            os: "Linux".to_string(),
            arch: "x86_64".to_string(),
            cpu_brand: "Mock CPU".to_string(),
            cpu_physical_cores: 8,
            cpu_logical_threads: 16,
            total_memory_mb: 16384,
            available_memory_mb: 12288,
            simd: voxforg_hardware::SimdSupport {
                avx: true,
                avx2: true,
                avx512: false,
                neon: false,
            },
            gpus: vec![],
            assigned_tier: voxforg_hardware::HardwareTier::Tier2Standard,
            recommended_engines: vec!["edge-tts".to_string(), "piper".to_string()],
        }
    }

    #[tokio::test]
    async fn test_worker_registration_and_selection() {
        let pool = WorkerPool::new();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        pool.register(WorkerRegistration {
            worker_id: id1,
            hardware: mock_hardware(),
            models: vec!["piper".to_string()],
            capacity: 2,
            address: "http://10.0.0.1:8080".to_string(),
        })
        .await;

        pool.register(WorkerRegistration {
            worker_id: id2,
            hardware: mock_hardware(),
            models: vec!["piper".to_string(), "kokoro".to_string()],
            capacity: 4,
            address: "http://10.0.0.2:8080".to_string(),
        })
        .await;

        assert_eq!(pool.list().await.len(), 2);

        // Select for "piper" - both have 0 jobs, but id2 has higher capacity so lower/equal load
        let selected = pool
            .select_worker("piper")
            .await
            .expect("Worker must be selected");
        assert!(selected.worker_id == id1 || selected.worker_id == id2);

        // Select for "kokoro" - only id2 supports it
        let selected_kokoro = pool.select_worker("kokoro").await.expect("Must select id2");
        assert_eq!(selected_kokoro.worker_id, id2);

        // Select non-existent engine
        assert!(pool.select_worker("non-existent").await.is_none());
    }

    #[tokio::test]
    async fn test_lease_acquisition_and_release() {
        let pool = WorkerPool::new();
        let id = Uuid::new_v4();

        pool.register(WorkerRegistration {
            worker_id: id,
            hardware: mock_hardware(),
            models: vec!["piper".to_string()],
            capacity: 1,
            address: "http://10.0.0.1:8080".to_string(),
        })
        .await;

        // Acquire lease
        pool.acquire_lease(&id)
            .await
            .expect("Lease 1 should succeed");
        let worker = pool.get(&id).await.unwrap();
        assert_eq!(worker.active_jobs, 1);
        assert_eq!(worker.status, WorkerStatus::Busy);

        // Capacity is 1, second lease should fail with rate limit
        assert!(pool.acquire_lease(&id).await.is_err());

        // Release lease
        pool.release_lease(&id)
            .await
            .expect("Release should succeed");
        let worker = pool.get(&id).await.unwrap();
        assert_eq!(worker.active_jobs, 0);
        assert_eq!(worker.status, WorkerStatus::Ready);
        assert_eq!(worker.total_jobs_processed, 1);
    }

    #[tokio::test]
    async fn test_heartbeat_and_stale_detection() {
        let pool = WorkerPool::new();
        let id = Uuid::new_v4();

        pool.register(WorkerRegistration {
            worker_id: id,
            hardware: mock_hardware(),
            models: vec!["piper".to_string()],
            capacity: 2,
            address: "http://10.0.0.1:8080".to_string(),
        })
        .await;

        // Heartbeat updates successfully
        assert!(pool.heartbeat(&id).await.is_ok());

        // Heartbeat for non-existent worker fails
        assert!(pool.heartbeat(&Uuid::new_v4()).await.is_err());

        // Simulate stale heartbeat (60 seconds ago)
        let past = chrono::Utc::now() - chrono::Duration::seconds(60);
        pool.set_last_heartbeat_for_test(&id, past).await;

        let staled = pool.mark_stale_workers(30).await;
        assert_eq!(staled, 1);

        let worker = pool.get(&id).await.unwrap();
        assert_eq!(worker.status, WorkerStatus::Offline);

        // Heartbeat restores status to Ready
        pool.heartbeat(&id).await.unwrap();
        let worker = pool.get(&id).await.unwrap();
        assert_eq!(worker.status, WorkerStatus::Ready);
    }
}
