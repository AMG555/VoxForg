use std::sync::Arc;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use voxforg_audio::{AudioAnalyzer, AudioQualityMetrics};
use voxforg_core::error::Result;

use crate::registry::EngineRegistry;
use crate::traits::SynthesisRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbTestScenario {
    pub name: String,
    pub text: String,
    pub variant_a: SynthesisRequest,
    pub variant_b: SynthesisRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VariantResult {
    pub engine_id: String,
    pub voice_id: String,
    pub latency_ms: f64,
    pub audio_duration_seconds: f64,
    pub realtime_factor: f64,
    pub metrics: AudioQualityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AbTestComparison {
    pub scenario_name: String,
    pub variant_a: VariantResult,
    pub variant_b: VariantResult,
    pub latency_delta_ms: f64,
    pub speedup_ratio: f64,
    pub rms_delta_db: f32,
    pub faster_variant: String,
    pub recommended_variant: String,
    pub summary: String,
}

pub struct AbTestRunner {
    engine_registry: Arc<EngineRegistry>,
}

impl AbTestRunner {
    pub fn new(engine_registry: Arc<EngineRegistry>) -> Self {
        Self { engine_registry }
    }

    pub async fn run_comparison(&self, scenario: &AbTestScenario) -> Result<AbTestComparison> {
        let (engine_a, _) = self.engine_registry.resolve_voice(&scenario.variant_a.voice_id).await?;
        let (engine_b, _) = self.engine_registry.resolve_voice(&scenario.variant_b.voice_id).await?;

        // Execute Variant A
        let mut req_a = scenario.variant_a.clone();
        req_a.text = scenario.text.clone();
        let start_a = Instant::now();
        let chunk_a = engine_a.synthesize(&req_a).await?;
        let elapsed_a = start_a.elapsed();
        let latency_a_ms = elapsed_a.as_secs_f64() * 1000.0;
        let metrics_a = AudioAnalyzer::analyze_pcm16(&chunk_a.pcm_data, chunk_a.sample_rate, chunk_a.channels);
        let rtf_a = if metrics_a.duration_seconds > 0.0 {
            elapsed_a.as_secs_f64() / metrics_a.duration_seconds
        } else {
            0.0
        };

        // Execute Variant B
        let mut req_b = scenario.variant_b.clone();
        req_b.text = scenario.text.clone();
        let start_b = Instant::now();
        let chunk_b = engine_b.synthesize(&req_b).await?;
        let elapsed_b = start_b.elapsed();
        let latency_b_ms = elapsed_b.as_secs_f64() * 1000.0;
        let metrics_b = AudioAnalyzer::analyze_pcm16(&chunk_b.pcm_data, chunk_b.sample_rate, chunk_b.channels);
        let rtf_b = if metrics_b.duration_seconds > 0.0 {
            elapsed_b.as_secs_f64() / metrics_b.duration_seconds
        } else {
            0.0
        };

        let latency_delta_ms = latency_b_ms - latency_a_ms;
        let speedup_ratio = if latency_b_ms > 0.0 {
            latency_a_ms / latency_b_ms
        } else {
            1.0
        };
        let rms_delta_db = metrics_b.rms_dbfs - metrics_a.rms_dbfs;

        let faster_variant = if (latency_a_ms - latency_b_ms).abs() < 0.1 {
            "EQUAL".to_string()
        } else if latency_a_ms < latency_b_ms {
            "A".to_string()
        } else {
            "B".to_string()
        };

        // Determine recommended variant (zero clipping preferred, then lower latency, non-silent)
        let recommended_variant = if metrics_a.is_silent && !metrics_b.is_silent {
            "B".to_string()
        } else if metrics_b.is_silent && !metrics_a.is_silent {
            "A".to_string()
        } else if metrics_a.clipping_samples_count == 0 && metrics_b.clipping_samples_count > 0 {
            "A".to_string()
        } else if metrics_b.clipping_samples_count == 0 && metrics_a.clipping_samples_count > 0 {
            "B".to_string()
        } else if latency_a_ms <= latency_b_ms {
            "A".to_string()
        } else {
            "B".to_string()
        };

        let summary = format!(
            "Variant {} recommended. Latency: A={:.2}ms, B={:.2}ms (Δ{:.2}ms). RMS: A={:.1}dBFS, B={:.1}dBFS.",
            recommended_variant, latency_a_ms, latency_b_ms, latency_delta_ms, metrics_a.rms_dbfs, metrics_b.rms_dbfs
        );

        Ok(AbTestComparison {
            scenario_name: scenario.name.clone(),
            variant_a: VariantResult {
                engine_id: engine_a.id().to_string(),
                voice_id: scenario.variant_a.voice_id.clone(),
                latency_ms: latency_a_ms,
                audio_duration_seconds: metrics_a.duration_seconds,
                realtime_factor: rtf_a,
                metrics: metrics_a,
            },
            variant_b: VariantResult {
                engine_id: engine_b.id().to_string(),
                voice_id: scenario.variant_b.voice_id.clone(),
                latency_ms: latency_b_ms,
                audio_duration_seconds: metrics_b.duration_seconds,
                realtime_factor: rtf_b,
                metrics: metrics_b,
            },
            latency_delta_ms,
            speedup_ratio,
            rms_delta_db,
            faster_variant,
            recommended_variant,
            summary,
        })
    }

    pub async fn run_batch(&self, scenarios: &[AbTestScenario]) -> Result<Vec<AbTestComparison>> {
        let mut results = Vec::with_capacity(scenarios.len());
        for scenario in scenarios {
            let res = self.run_comparison(scenario).await?;
            results.push(res);
        }
        Ok(results)
    }
}
