use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use voxforg_core::error::VoxForgError;

use crate::types::{TranscriptionOptions, TranscriptionResult};

/// Metadata description for an ASR engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AsrEngineInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub supported_languages: Vec<String>,
    pub is_local: bool,
    pub sample_rate: u32,
}

/// Abstract contract for automated speech recognition engines.
#[async_trait]
pub trait AsrEngine: Send + Sync {
    /// Return engine metadata and capabilities.
    fn info(&self) -> AsrEngineInfo;

    /// Primary unique identifier for the engine (e.g. "whisper-base", "mock-asr").
    fn id(&self) -> String {
        self.info().id
    }

    /// Human-readable display name.
    fn name(&self) -> String {
        self.info().name
    }

    /// Transcribe an audio PCM sample slice into text and timed segments.
    async fn transcribe(
        &self,
        audio_pcm: &[i16],
        sample_rate: u32,
        options: &TranscriptionOptions,
    ) -> Result<TranscriptionResult, VoxForgError>;
}
