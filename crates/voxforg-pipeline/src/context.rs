use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use voxforg_core::models::AudioContainerFormat;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptSegment {
    pub speaker: String,
    pub text: String,
    pub voice_id: Option<String>,
    pub speed: Option<f32>,
    pub pitch: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    pub raw_text: Option<String>,
    pub segments: Vec<ScriptSegment>,
    pub audio_segments: Vec<Vec<i16>>,
    pub master_audio_pcm: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u16,
    pub output_format: AudioContainerFormat,
    pub node_outputs: HashMap<String, serde_json::Value>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            raw_text: None,
            segments: Vec::new(),
            audio_segments: Vec::new(),
            master_audio_pcm: Vec::new(),
            sample_rate: 24000,
            channels: 1,
            output_format: AudioContainerFormat::Wav,
            node_outputs: HashMap::new(),
        }
    }
}
