use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AudioContainerFormat {
    #[default]
    Wav,
    Mp3,
    Opus,
    Pcm,
    Flac,
    Aac,
}

impl AudioContainerFormat {
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Wav => "audio/wav",
            Self::Mp3 => "audio/mpeg",
            Self::Opus => "audio/opus",
            Self::Pcm => "audio/pcm",
            Self::Flac => "audio/flac",
            Self::Aac => "audio/aac",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioChunk {
    pub sample_rate: u32,
    pub channels: u16,
    pub pcm_data: Vec<i16>,
    pub is_final: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub duration_seconds: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: AudioContainerFormat,
    pub byte_size: usize,
}
