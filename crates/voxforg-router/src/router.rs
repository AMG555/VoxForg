//! `VoiceRouter` — scores registered engines against a `SynthesisPolicy`
//! and returns the best-fit engine, with automatic fallback.

use std::sync::Arc;
use tracing::{debug, warn};
use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::AudioContainerFormat;
use voxforg_engine::{EngineRegistry, SynthesisRequest, TtsEngine};

use crate::policy::{RoutingDecision, SynthesisPolicy};
use crate::scorer::score_engine;

/// Policy-based voice router.
///
/// `VoiceRouter` wraps an [`EngineRegistry`] and selects the best engine for a
/// given [`SynthesisPolicy`]. When `policy.fallback_chain` is non-empty the
/// engines are tried in that order (highest-score first within the chain).
/// Otherwise every registered engine is a candidate.
///
/// Routing is fully backward-compatible: if no policy is supplied callers
/// continue using the registry's `resolve_voice` directly.
#[derive(Clone)]
pub struct VoiceRouter {
    registry: EngineRegistry,
}

impl VoiceRouter {
    pub fn new(registry: EngineRegistry) -> Self {
        Self { registry }
    }

    /// Select the best engine for `policy` and construct a [`SynthesisRequest`].
    ///
    /// Returns `(RoutingDecision, SynthesisRequest)` or an error when no
    /// registered engine satisfies all hard constraints.
    pub async fn route(
        &self,
        policy: &SynthesisPolicy,
        text: impl Into<String>,
        voice_id: impl Into<String>,
        format: AudioContainerFormat,
    ) -> Result<(RoutingDecision, SynthesisRequest)> {
        let text = text.into();
        let voice_id = voice_id.into();

        let candidates = self.collect_candidates(policy).await;

        if candidates.is_empty() {
            return Err(VoxForgError::Engine(
                "No registered engine satisfies the synthesis policy constraints".to_string(),
            ));
        }

        // Pick highest-scoring candidate, attempt health check, fall back on failure
        for (engine, score) in candidates {
            let id = engine.id().to_string();
            match engine.health_check().await {
                Ok(true) => {
                    debug!(engine_id = %id, score, "router selected engine");
                    let request = SynthesisRequest {
                        text: text.clone(),
                        voice_id: voice_id.clone(),
                        speed: 1.0,
                        pitch: 0.0,
                        format,
                    };
                    let decision = RoutingDecision {
                        engine,
                        engine_id: id,
                        score,
                    };
                    return Ok((decision, request));
                }
                Ok(false) | Err(_) => {
                    warn!(engine_id = %id, "router skipping unhealthy engine, trying next");
                }
            }
        }

        Err(VoxForgError::Engine(
            "All candidate engines failed health check".to_string(),
        ))
    }

    /// Collect scored candidates sorted best-first.
    ///
    /// When `policy.fallback_chain` is non-empty, only those engine IDs are
    /// considered (in the declared order, then sorted by score within that set).
    async fn collect_candidates(&self, policy: &SynthesisPolicy) -> Vec<(Arc<dyn TtsEngine>, f32)> {
        let engine_ids: Vec<String> = if policy.fallback_chain.is_empty() {
            self.registry.list_engines().await
        } else {
            policy.fallback_chain.clone()
        };

        let mut scored: Vec<(Arc<dyn TtsEngine>, f32)> = Vec::new();

        for id in &engine_ids {
            if let Some(engine) = self.registry.get(id).await {
                let caps = engine.capabilities();
                match score_engine(&caps, policy) {
                    Some(score) => scored.push((engine, score)),
                    None => debug!(engine_id = %id, "engine eliminated by policy hard constraints"),
                }
            } else {
                debug!(engine_id = %id, "engine in fallback_chain not registered");
            }
        }

        // Best score first
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use voxforg_core::models::AudioContainerFormat;
    use voxforg_engine::{EngineRegistry, MockTtsEngine};

    async fn make_router() -> VoiceRouter {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockTtsEngine::new(24000))).await;
        VoiceRouter::new(registry)
    }

    #[tokio::test]
    async fn router_selects_mock_with_open_policy() {
        let router = make_router().await;
        let policy = SynthesisPolicy::default();
        let result = router
            .route(
                &policy,
                "hello world",
                "mock-en-male",
                AudioContainerFormat::Wav,
            )
            .await;
        assert!(result.is_ok(), "open policy should always find mock engine");
        let (decision, _req) = result.unwrap();
        assert_eq!(decision.engine_id, "mock-tts");
    }

    #[tokio::test]
    async fn router_rejects_when_quality_floor_too_high() {
        let router = make_router().await;
        // mock engine quality is 0.3; demanding 0.9 should fail
        let policy = SynthesisPolicy {
            quality_min: Some(0.9),
            ..Default::default()
        };
        let result = router
            .route(&policy, "hello", "mock-en-male", AudioContainerFormat::Wav)
            .await;
        assert!(
            result.is_err(),
            "high quality floor must eliminate mock engine"
        );
    }

    #[tokio::test]
    async fn router_respects_fallback_chain_order() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockTtsEngine::new(24000))).await;
        let router = VoiceRouter::new(registry);

        let policy = SynthesisPolicy {
            fallback_chain: vec!["mock-tts".to_string()],
            ..Default::default()
        };
        let (decision, _) = router
            .route(&policy, "test", "mock-en-male", AudioContainerFormat::Wav)
            .await
            .expect("explicit chain with mock must succeed");
        assert_eq!(decision.engine_id, "mock-tts");
    }
}
