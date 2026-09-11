//! In-memory benchmark result store.
//!
//! Results are kept in RAM and keyed by engine ID. The store is designed to be
//! held behind an `Arc<BenchmarkStore>` in `AppState` for concurrent access.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Latency percentile statistics across the benchmark sentence suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    /// Median latency in milliseconds.
    pub p50_ms: u64,
    /// 95th-percentile latency in milliseconds.
    pub p95_ms: u64,
    /// Maximum observed latency in milliseconds.
    pub max_ms: u64,
}

impl LatencyStats {
    /// Compute stats from a sorted (ascending) slice of millisecond values.
    pub fn from_sorted(sorted: &[u64]) -> Option<Self> {
        if sorted.is_empty() {
            return None;
        }
        let n = sorted.len();
        let p50 = sorted[n / 2];
        let p95 = sorted[(n as f64 * 0.95) as usize].min(*sorted.last().unwrap());
        let max = *sorted.last().unwrap();
        Some(Self {
            p50_ms: p50,
            p95_ms: p95,
            max_ms: max,
        })
    }
}

/// Full scorecard for one engine on one benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Engine identifier.
    pub engine_id: String,
    /// Number of sentences successfully synthesized.
    pub sentences_ok: usize,
    /// Number of sentences that failed synthesis.
    pub sentences_failed: usize,
    /// Latency statistics across successful sentences (None if all failed).
    pub latency: Option<LatencyStats>,
    /// Throughput in sentences per second (0.0 if all failed).
    pub throughput_sps: f32,
    /// Overall score: 0.0–1.0. Combines success rate (60%) + latency (40%).
    /// Latency contribution is normalised against 2,000 ms ceiling.
    pub overall_score: f32,
    /// Timestamp when this benchmark was executed.
    pub run_at: DateTime<Utc>,
}

/// Thread-safe in-memory store for benchmark results, keyed by engine ID.
#[derive(Clone, Default)]
pub struct BenchmarkStore {
    results: Arc<RwLock<HashMap<String, BenchmarkResult>>>,
}

impl BenchmarkStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Persist (or overwrite) a result for an engine.
    pub async fn insert(&self, result: BenchmarkResult) {
        let mut lock = self.results.write().await;
        lock.insert(result.engine_id.clone(), result);
    }

    /// Return all stored results sorted by overall_score descending.
    pub async fn list(&self) -> Vec<BenchmarkResult> {
        let lock = self.results.read().await;
        let mut results: Vec<_> = lock.values().cloned().collect();
        results.sort_by(|a, b| {
            b.overall_score
                .partial_cmp(&a.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Return the latest result for a specific engine.
    pub async fn get(&self, engine_id: &str) -> Option<BenchmarkResult> {
        let lock = self.results.read().await;
        lock.get(engine_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latency_stats_from_sorted_slice() {
        // 9-element slice: median (index 4) = 200, p95 (index 8) = 1100, max = 1100
        let sorted = vec![10u64, 50, 100, 150, 200, 400, 600, 800, 1100];
        let stats = LatencyStats::from_sorted(&sorted).unwrap();
        assert_eq!(stats.p50_ms, 200); // index 9/2 = 4 → 200
        assert_eq!(stats.max_ms, 1100);
        assert!(
            stats.p95_ms >= 800,
            "p95 should be near max: {}",
            stats.p95_ms
        );
    }

    #[test]
    fn latency_stats_single_element() {
        let stats = LatencyStats::from_sorted(&[42]).unwrap();
        assert_eq!(stats.p50_ms, 42);
        assert_eq!(stats.p95_ms, 42);
        assert_eq!(stats.max_ms, 42);
    }

    #[test]
    fn latency_stats_empty_returns_none() {
        assert!(LatencyStats::from_sorted(&[]).is_none());
    }

    #[tokio::test]
    async fn store_insert_and_list() {
        let store = BenchmarkStore::new();
        let r = BenchmarkResult {
            engine_id: "mock-tts".to_string(),
            sentences_ok: 10,
            sentences_failed: 0,
            latency: Some(LatencyStats {
                p50_ms: 10,
                p95_ms: 15,
                max_ms: 20,
            }),
            throughput_sps: 50.0,
            overall_score: 0.95,
            run_at: Utc::now(),
        };
        store.insert(r.clone()).await;
        let list = store.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].engine_id, "mock-tts");
    }
}
