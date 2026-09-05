use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use voxforg_core::error::Result;
use voxforg_core::models::{AudioChunk, AudioContainerFormat, Voice};

fn default_speed() -> f32 {
    1.0
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
    async fn voices(&self) -> Result<Vec<Voice>>;
    async fn synthesize(&self, request: &SynthesisRequest) -> Result<AudioChunk>;
    async fn synthesize_stream(
        &self,
        request: &SynthesisRequest,
    ) -> Result<mpsc::Receiver<Result<AudioChunk>>>;
    async fn health_check(&self) -> Result<bool>;
}
