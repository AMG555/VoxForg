use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use voxforg_core::error::Result;
use voxforg_core::models::{AudioChunk, AudioContainerFormat, Voice};

fn default_speed() -> f32 {
    1.0
}

/// Advertised capabilities of a TTS engine.
/// Used by `VoiceRouter` to score engines against a `SynthesisPolicy`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCapabilities {
    /// Estimated cost in USD per 1,000 characters (0.0 = free/local).
    pub cost_per_1k_chars: f32,
    /// Typical synthesis latency in milliseconds (P50).
    pub avg_latency_ms: u64,
    /// Subjective quality score: 0.0 (poor) – 1.0 (excellent).
    pub quality_score: f32,
    /// BCP-47 language codes supported, e.g. ["en-US", "hi-IN"].
    pub languages: Vec<String>,
    /// True when the engine runs locally (no upstream network needed).
    pub is_local: bool,
}

impl Default for EngineCapabilities {
    fn default() -> Self {
        Self {
            cost_per_1k_chars: 0.0,
            avg_latency_ms: 500,
            quality_score: 0.7,
            languages: vec!["en-US".to_string()],
            is_local: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisRequest {
    #[serde(default)]
    pub text: String,
    pub voice_id: String,
    #[serde(default = "default_speed")]
    pub speed: f32,
    #[serde(default)]
    pub pitch: f32,
    #[serde(default)]
    pub format: AudioContainerFormat,
}

impl Default for SynthesisRequest {
    fn default() -> Self {
        Self {
            text: String::new(),
            voice_id: String::new(),
            speed: 1.0,
            pitch: 0.0,
            format: AudioContainerFormat::Wav,
        }
    }
}

#[async_trait]
pub trait TtsEngine: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn is_local(&self) -> bool;
    /// Return engine capability metadata for routing decisions.
    fn capabilities(&self) -> EngineCapabilities;
    async fn voices(&self) -> Result<Vec<Voice>>;
    async fn synthesize(&self, request: &SynthesisRequest) -> Result<AudioChunk>;
    async fn synthesize_stream(
        &self,
        request: &SynthesisRequest,
    ) -> Result<mpsc::Receiver<Result<AudioChunk>>>;
    async fn health_check(&self) -> Result<bool>;
}
