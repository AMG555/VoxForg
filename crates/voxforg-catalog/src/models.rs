use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Functional domain classification of a model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    Tts,
    Asr,
    Vad,
    Diarizer,
}

/// Serialization weight format container.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelFormat {
    Onnx,
    PyTorch,
    Safetensors,
    Ggml,
}

/// Installation and runtime state of a catalog model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatus {
    Available,
    Downloading,
    Installed,
    Error,
}

/// Metadata, storage, and requirements descriptor for a catalog weight package.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub model_type: ModelType,
    pub format: ModelFormat,
    pub size_bytes: u64,
    pub sha256: String,
    pub download_url: String,
    pub min_ram_mb: u64,
    pub requires_gpu: bool,
    pub supported_languages: Vec<String>,
    pub status: ModelStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
}
