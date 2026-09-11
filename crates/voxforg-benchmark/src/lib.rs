//! # voxforg-benchmark
//!
//! Standardized engine benchmarking for VoxForg.
//!
//! Runs a canonical sentence suite against every registered TTS engine and
//! produces per-engine scorecards: latency percentiles, throughput, and a
//! quality heuristic based on synthesis success rate and timing consistency.
//!
//! ## Usage
//! ```rust,ignore
//! let runner = BenchmarkRunner::new(registry.clone());
//! let results = runner.run_all().await?;
//! for r in &results {
//!     println!("{}: score={:.2}", r.engine_id, r.overall_score);
//! }
//! ```

pub mod runner;
pub mod sentences;
pub mod store;

pub use runner::BenchmarkRunner;
pub use sentences::BENCHMARK_SENTENCES;
pub use store::{BenchmarkResult, BenchmarkStore};
