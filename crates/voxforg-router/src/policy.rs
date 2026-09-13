//! `SynthesisPolicy` — intent declaration for engine selection.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use voxforg_engine::TtsEngine;

/// Voice style hint passed to the router.
/// Engines that explicitly support a style get a small scoring bonus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VoiceStyle {
    #[default]
    Any,
    Conversational,
    Narration,
    News,
    Assistant,
}

/// Caller-declared synthesis intent.
/// All fields are optional — omitted constraints are treated as unconstrained.
///
/// When a `fallback_chain` is provided it is tried in order if the best-scored
/// engine is unavailable. If empty, ALL registered engines are candidates.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SynthesisPolicy {
    /// Minimum quality score a candidate engine must meet (0.0–1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_min: Option<f32>,

    /// Maximum acceptable synthesis latency in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_max_ms: Option<u64>,

    /// Maximum acceptable cost in USD per 1,000 characters.
    /// Set to 0.0 to restrict to free / local engines only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_max_per_1k: Option<f32>,

    /// Prefer an engine that supports this BCP-47 language code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Voice style preference (informational; engines may ignore).
    #[serde(default)]
    pub style: VoiceStyle,

    /// Enforce voice cloning capability. Candidates without zero-shot cloning are strictly rejected.
    #[serde(default)]
    pub require_cloning: bool,

    /// Enforce streaming capability. Candidates without chunked streaming are strictly rejected.
    #[serde(default)]
    pub require_streaming: bool,

    /// Explicit fallback chain of engine IDs to attempt in order.
    /// Empty = auto-select from all registered engines.
    #[serde(default)]
    pub fallback_chain: Vec<String>,
}

/// Result of a successful routing decision.
#[derive(Clone)]
pub struct RoutingDecision {
    /// The engine selected by the router.
    pub engine: Arc<dyn TtsEngine>,
    /// Engine ID for logging / observability.
    pub engine_id: String,
    /// Score the engine received (higher is better).
    pub score: f32,
}

impl std::fmt::Debug for RoutingDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoutingDecision")
            .field("engine_id", &self.engine_id)
            .field("score", &self.score)
            .finish()
    }
}
