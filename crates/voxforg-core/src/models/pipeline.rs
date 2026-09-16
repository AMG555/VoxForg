use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    TextInput,
    CharacterVoice,
    SpeakerParser,
    VoiceAssigner,
    Synthesizer,
    AudioFilter,
    AudioMerge,
    OutputSink,
    // Production DAG workflow nodes
    AsrTranscriber,
    Diarization,
    DocumentChunker,
    AudioTimeStretch,
    AudioMux,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub params: serde_json::Value,
    #[serde(default)]
    pub position: Option<NodePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineEdge {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
    #[serde(default)]
    pub from_port: Option<String>,
    #[serde(default)]
    pub to_port: Option<String>,
}

fn default_flexible_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<serde_json::Value>::deserialize(deserializer)?;
    match opt {
        Some(serde_json::Value::String(s)) => {
            if let Ok(u) = Uuid::parse_str(&s) {
                Ok(u)
            } else {
                Ok(Uuid::new_v4())
            }
        }
        _ => Ok(Uuid::new_v4()),
    }
}

fn default_now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    #[serde(default = "Uuid::new_v4", deserialize_with = "default_flexible_uuid")]
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub nodes: Vec<PipelineNode>,
    pub edges: Vec<PipelineEdge>,
    #[serde(default = "default_now")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default = "default_now")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
