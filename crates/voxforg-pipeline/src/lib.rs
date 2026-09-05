pub mod context;
pub mod executor;
pub mod graph;

pub use context::{ExecutionContext, ScriptSegment};
pub use executor::PipelineExecutor;
pub use graph::GraphValidator;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uuid::Uuid;
    use voxforg_audio::WavEncoder;
    use voxforg_core::models::{NodeType, PipelineDefinition, PipelineEdge, PipelineNode};
    use voxforg_engine::{EngineRegistry, MockTtsEngine};

    #[tokio::test]
    async fn test_full_pipeline_execution() {
        let registry = Arc::new(EngineRegistry::new());
        let mock_engine = Arc::new(MockTtsEngine::new(24000));
        registry.register(mock_engine).await;

        let pipeline = PipelineDefinition {
            id: Uuid::new_v4(),
            name: "Dialog Test".to_string(),
            description: None,
            nodes: vec![
                PipelineNode {
                    id: "n1".to_string(),
                    name: "Script Input".to_string(),
                    node_type: NodeType::TextInput,
                    params: serde_json::json!({
                        "text": "Alice: Greetings commander.\nBob: Systems ready."
                    }),
                    position: None,
                },
                PipelineNode {
                    id: "n2".to_string(),
                    name: "Speaker Parser".to_string(),
                    node_type: NodeType::SpeakerParser,
                    params: serde_json::json!({}),
                    position: None,
                },
                PipelineNode {
                    id: "n3".to_string(),
                    name: "Voice Assigner".to_string(),
                    node_type: NodeType::VoiceAssigner,
                    params: serde_json::json!({
                        "default_voice": "mock-en-female",
                        "speaker_map": {
                            "Alice": "mock-en-female",
                            "Bob": "mock-en-male"
                        }
                    }),
                    position: None,
                },
                PipelineNode {
                    id: "n4".to_string(),
                    name: "Synthesizer".to_string(),
                    node_type: NodeType::Synthesizer,
                    params: serde_json::json!({}),
                    position: None,
                },
                PipelineNode {
                    id: "n5".to_string(),
                    name: "Audio Filter".to_string(),
                    node_type: NodeType::AudioFilter,
                    params: serde_json::json!({ "normalize_peak": 0.95 }),
                    position: None,
                },
                PipelineNode {
                    id: "n6".to_string(),
                    name: "Audio Merger".to_string(),
                    node_type: NodeType::AudioMerge,
                    params: serde_json::json!({ "pause_ms": 150 }),
                    position: None,
                },
                PipelineNode {
                    id: "n7".to_string(),
                    name: "Output".to_string(),
                    node_type: NodeType::OutputSink,
                    params: serde_json::json!({}),
                    position: None,
                },
            ],
            edges: vec![
                PipelineEdge { id: "e1".to_string(), from_node: "n1".to_string(), to_node: "n2".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e2".to_string(), from_node: "n2".to_string(), to_node: "n3".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e3".to_string(), from_node: "n3".to_string(), to_node: "n4".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e4".to_string(), from_node: "n4".to_string(), to_node: "n5".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e5".to_string(), from_node: "n5".to_string(), to_node: "n6".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e6".to_string(), from_node: "n6".to_string(), to_node: "n7".to_string(), from_port: None, to_port: None },
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let executor = PipelineExecutor::new(registry);
        let wav_output = executor
            .execute(&pipeline, None)
            .await
            .expect("Pipeline execution must succeed");

        assert!(!wav_output.is_empty());
        let (pcm, rate, channels) = WavEncoder::decode_wav_to_pcm16(&wav_output).unwrap();
        assert_eq!(rate, 24000);
        assert_eq!(channels, 1);
        assert!(!pcm.is_empty());
    }

    #[test]
    fn test_cycle_detection() {
        let cyclic_pipeline = PipelineDefinition {
            id: Uuid::new_v4(),
            name: "Cyclic".to_string(),
            description: None,
            nodes: vec![
                PipelineNode { id: "a".to_string(), name: "A".to_string(), node_type: NodeType::TextInput, params: serde_json::json!({}), position: None },
                PipelineNode { id: "b".to_string(), name: "B".to_string(), node_type: NodeType::Synthesizer, params: serde_json::json!({}), position: None },
            ],
            edges: vec![
                PipelineEdge { id: "e1".to_string(), from_node: "a".to_string(), to_node: "b".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e2".to_string(), from_node: "b".to_string(), to_node: "a".to_string(), from_port: None, to_port: None },
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = GraphValidator::topological_sort(&cyclic_pipeline);
        assert!(result.is_err());
    }
}
