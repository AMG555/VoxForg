//! Engine scoring logic for `SynthesisPolicy` matching.
//!
//! `score_engine` returns `None` when the engine **hard-fails** a constraint
//! (e.g. latency ceiling exceeded), or `Some(score)` where a higher score
//! means a better match. Callers select the engine with the highest score.

use voxforg_engine::EngineCapabilities;

use crate::policy::SynthesisPolicy;

/// Score `caps` against `policy`.
///
/// Returns `None` if the engine fails a hard constraint.
/// Returns `Some(score)` (higher = better) otherwise.
///
/// Scoring formula:
/// - quality_score  contributes 40 % weight
/// - latency (inverted, normalised to 2000 ms ceiling) contributes 40 %
/// - cost    (inverted, normalised to $0.10 / 1K ceiling) contributes 20 %
/// - +0.05 bonus when language matches
pub fn score_engine(caps: &EngineCapabilities, policy: &SynthesisPolicy) -> Option<f32> {
    // ── Hard constraints (fail-fast) ─────────────────────────────────────────
    if let Some(q_min) = policy.quality_min {
        if caps.quality_score < q_min {
            return None;
        }
    }
    if let Some(lat_max) = policy.latency_max_ms {
        if caps.avg_latency_ms > lat_max {
            return None;
        }
    }
    if let Some(cost_max) = policy.cost_max_per_1k {
        if caps.cost_per_1k_chars > cost_max {
            return None;
        }
    }

    // ── Soft scoring ─────────────────────────────────────────────────────────
    const MAX_LATENCY_MS: f32 = 2_000.0;
    const MAX_COST: f32 = 0.10;

    let latency_score = 1.0 - (caps.avg_latency_ms as f32 / MAX_LATENCY_MS).min(1.0);
    let cost_score = 1.0 - (caps.cost_per_1k_chars / MAX_COST).min(1.0);

    let mut score = caps.quality_score * 0.40 + latency_score * 0.40 + cost_score * 0.20;

    // Language match bonus
    if let Some(ref lang) = policy.language {
        if caps.languages.iter().any(|l| l == lang) {
            score += 0.05;
        }
    }

    Some(score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::SynthesisPolicy;
    use voxforg_engine::EngineCapabilities;

    fn make_caps(quality: f32, latency_ms: u64, cost: f32) -> EngineCapabilities {
        EngineCapabilities {
            quality_score: quality,
            avg_latency_ms: latency_ms,
            cost_per_1k_chars: cost,
            languages: vec!["en-US".to_string()],
            is_local: cost == 0.0,
        }
    }

    #[test]
    fn hard_quality_constraint_rejects_engine() {
        let caps = make_caps(0.5, 100, 0.0);
        let policy = SynthesisPolicy {
            quality_min: Some(0.8),
            ..Default::default()
        };
        assert!(score_engine(&caps, &policy).is_none());
    }

    #[test]
    fn hard_latency_constraint_rejects_engine() {
        let caps = make_caps(0.9, 1000, 0.0);
        let policy = SynthesisPolicy {
            latency_max_ms: Some(500),
            ..Default::default()
        };
        assert!(score_engine(&caps, &policy).is_none());
    }

    #[test]
    fn hard_cost_constraint_rejects_paid_engine() {
        let caps = make_caps(0.9, 200, 0.02);
        let policy = SynthesisPolicy {
            cost_max_per_1k: Some(0.0),
            ..Default::default()
        };
        assert!(score_engine(&caps, &policy).is_none());
    }

    #[test]
    fn free_local_engine_passes_zero_cost_policy() {
        let caps = make_caps(0.7, 50, 0.0);
        let policy = SynthesisPolicy {
            cost_max_per_1k: Some(0.0),
            ..Default::default()
        };
        assert!(score_engine(&caps, &policy).is_some());
    }

    #[test]
    fn language_match_bonus_increases_score() {
        let caps = make_caps(0.7, 300, 0.0);
        let policy_no_lang = SynthesisPolicy::default();
        let policy_with_lang = SynthesisPolicy {
            language: Some("en-US".to_string()),
            ..Default::default()
        };
        let score_no_lang = score_engine(&caps, &policy_no_lang).unwrap();
        let score_with_lang = score_engine(&caps, &policy_with_lang).unwrap();
        assert!(score_with_lang > score_no_lang);
    }

    #[test]
    fn higher_quality_engine_scores_higher() {
        let low = make_caps(0.3, 100, 0.0);
        let high = make_caps(0.9, 100, 0.0);
        let policy = SynthesisPolicy::default();
        let s_low = score_engine(&low, &policy).unwrap();
        let s_high = score_engine(&high, &policy).unwrap();
        assert!(s_high > s_low);
    }
}
