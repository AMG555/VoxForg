//! # voxforg-router
//!
//! SLA-policy-based voice routing for VoxForg.
//!
//! Instead of callers choosing a specific TTS engine, they express intent via a
//! [`SynthesisPolicy`] (quality floor, latency ceiling, cost budget, language).
//! [`VoiceRouter`] scores all registered engines against the policy and returns
//! the best matching engine with a ready-to-use [`SynthesisRequest`], falling
//! back through the chain on engine failure.
//!
//! ## Example
//! ```rust,ignore
//! let policy = SynthesisPolicy {
//!     quality_min: Some(0.8),
//!     latency_max_ms: Some(400),
//!     cost_max_per_1k: None,
//!     language: Some("en-US".to_string()),
//!     fallback_chain: vec!["edge-tts".to_string(), "mock-tts".to_string()],
//! };
//! let (engine, request) = router
//!     .route(&policy, text, format)
//!     .await
//!     .expect("at least one engine must satisfy policy");
//! ```

pub mod policy;
pub mod router;
pub mod scorer;

pub use policy::{RoutingDecision, SynthesisPolicy, VoiceStyle};
pub use router::VoiceRouter;
pub use scorer::score_engine;
