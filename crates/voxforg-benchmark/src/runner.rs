//! `BenchmarkRunner` — runs the canonical sentence suite against all registered engines.

use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use tracing::{info, warn};
use voxforg_core::models::AudioContainerFormat;
use voxforg_engine::{EngineRegistry, SynthesisRequest};

use crate::sentences::BENCHMARK_SENTENCES;
use crate::store::{BenchmarkResult, BenchmarkStore, LatencyStats};

/// Runs the benchmark suite against all registered engines and persists results
/// in the provided [`BenchmarkStore`].
pub struct BenchmarkRunner {
    registry: Arc<EngineRegistry>,
    store: Arc<BenchmarkStore>,
}

impl BenchmarkRunner {
    pub fn new(registry: Arc<EngineRegistry>, store: Arc<BenchmarkStore>) -> Self {
        Self { registry, store }
    }

    /// Benchmark every registered engine and return the results.
    ///
    /// Errors from individual sentence synthesis are logged as warnings and
    /// counted in `sentences_failed`; they do not abort the benchmark.
    pub async fn run_all(&self) -> Vec<BenchmarkResult> {
        let engine_ids = self.registry.list_engines().await;
        let mut results = Vec::with_capacity(engine_ids.len());

        for engine_id in engine_ids {
            let result = self.run_engine(&engine_id).await;
            info!(
                engine_id = %result.engine_id,
                score = result.overall_score,
                ok = result.sentences_ok,
                failed = result.sentences_failed,
                "benchmark complete"
            );
            self.store.insert(result.clone()).await;
            results.push(result);
        }

        // Sort best-first before returning
        results.sort_by(|a, b| {
            b.overall_score
                .partial_cmp(&a.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    async fn run_engine(&self, engine_id: &str) -> BenchmarkResult {
        let engine = match self.registry.get(engine_id).await {
            Some(e) => e,
            None => {
                return BenchmarkResult {
                    engine_id: engine_id.to_string(),
                    sentences_ok: 0,
                    sentences_failed: BENCHMARK_SENTENCES.len(),
                    latency: None,
                    throughput_sps: 0.0,
                    overall_score: 0.0,
                    run_at: Utc::now(),
                };
            }
        };

        let mut latencies_ms: Vec<u64> = Vec::with_capacity(BENCHMARK_SENTENCES.len());
        let mut failed = 0usize;
        let wall_start = Instant::now();

        for (id, text) in BENCHMARK_SENTENCES {
            let req = SynthesisRequest {
                text: text.to_string(),
                voice_id: String::new(), // engines may ignore for benchmarking
                speed: 1.0,
                pitch: 0.0,
                format: AudioContainerFormat::Wav,
            };

            let t0 = Instant::now();
            match engine.synthesize(&req).await {
                Ok(_) => latencies_ms.push(t0.elapsed().as_millis() as u64),
                Err(e) => {
                    warn!(engine_id, sentence_id = id, error = %e, "synthesis failed in benchmark");
                    failed += 1;
                }
            }
        }

        let total_secs = wall_start.elapsed().as_secs_f32().max(f32::EPSILON);
        let ok = latencies_ms.len();
        let throughput_sps = ok as f32 / total_secs;

        latencies_ms.sort_unstable();
        let latency = LatencyStats::from_sorted(&latencies_ms);

        // Score: 60% success rate + 40% latency (lower is better, ceiling 2000 ms)
        let success_rate = ok as f32 / BENCHMARK_SENTENCES.len() as f32;
        let latency_score = latency
            .as_ref()
            .map_or(0.0, |l| 1.0 - (l.p50_ms as f32 / 2_000.0).min(1.0));
        let overall_score = success_rate * 0.60 + latency_score * 0.40;

        BenchmarkResult {
            engine_id: engine_id.to_string(),
            sentences_ok: ok,
            sentences_failed: failed,
            latency,
            throughput_sps,
            overall_score,
            run_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxforg_engine::MockTtsEngine;

    #[tokio::test]
    async fn benchmark_runner_produces_result_for_mock_engine() {
        let registry = Arc::new(EngineRegistry::new());
        registry.register(Arc::new(MockTtsEngine::new(24000))).await;

        let store = Arc::new(BenchmarkStore::new());
        let runner = BenchmarkRunner::new(registry, store.clone());

        let results = runner.run_all().await;
        assert_eq!(results.len(), 1);

        let r = &results[0];
        assert_eq!(r.engine_id, "mock-tts");
        assert_eq!(r.sentences_failed, 0);
        assert_eq!(r.sentences_ok, BENCHMARK_SENTENCES.len());
        assert!(
            r.overall_score > 0.5,
            "mock engine should score reasonably: {}",
            r.overall_score
        );

        // Result should also be in the store
        let stored = store.get("mock-tts").await;
        assert!(stored.is_some());
    }

    #[tokio::test]
    async fn benchmark_runner_empty_registry_returns_empty_results() {
        let registry = Arc::new(EngineRegistry::new());
        let store = Arc::new(BenchmarkStore::new());
        let runner = BenchmarkRunner::new(registry, store);
        let results = runner.run_all().await;
        assert!(results.is_empty());
    }
}
