use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::{
    AudioChunk, AudioContainerFormat, CloneVoiceRequest, ClonedSynthesisRequest, Voice,
    VoiceProfile,
};

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
    /// True when the engine supports zero-shot voice cloning.
    #[serde(default)]
    pub supports_cloning: bool,
    /// True when the engine supports streaming output chunks.
    #[serde(default = "default_true")]
    pub supports_streaming: bool,
}

fn default_true() -> bool {
    true
}

impl Default for EngineCapabilities {
    fn default() -> Self {
        Self {
            cost_per_1k_chars: 0.0,
            avg_latency_ms: 500,
            quality_score: 0.7,
            languages: vec!["en-US".to_string()],
            is_local: true,
            supports_cloning: false,
            supports_streaming: true,
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

    /// Whether this engine supports reference-audio zero-shot voice cloning.
    fn supports_cloning(&self) -> bool {
        false
    }

    /// Extract an acoustic embedding and register a new cloned voice profile.
    async fn clone_voice(&self, _request: &CloneVoiceRequest) -> Result<VoiceProfile> {
        Err(VoxForgError::Engine(format!(
            "Engine '{}' does not support voice cloning",
            self.id()
        )))
    }

    /// Synthesize speech conditioned on a cloned voice profile.
    async fn synthesize_cloned(&self, _request: &ClonedSynthesisRequest) -> Result<AudioChunk> {
        Err(VoxForgError::Engine(format!(
            "Engine '{}' does not support cloned voice synthesis",
            self.id()
        )))
    }
}
