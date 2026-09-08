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

    #[test]
    fn test_invalid_edge_references() {
        let invalid_pipeline = PipelineDefinition {
            id: Uuid::new_v4(),
            name: "Invalid Edge".to_string(),
            description: None,
            nodes: vec![
                PipelineNode { id: "a".to_string(), name: "A".to_string(), node_type: NodeType::TextInput, params: serde_json::json!({}), position: None },
            ],
            edges: vec![
                PipelineEdge { id: "e1".to_string(), from_node: "a".to_string(), to_node: "non_existent".to_string(), from_port: None, to_port: None },
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = GraphValidator::topological_sort(&invalid_pipeline);
        assert!(result.is_err());
    }

    #[test]
    fn test_topological_sort_diamond_graph() {
        let diamond_pipeline = PipelineDefinition {
            id: Uuid::new_v4(),
            name: "Diamond".to_string(),
            description: None,
            nodes: vec![
                PipelineNode { id: "in".to_string(), name: "Input".to_string(), node_type: NodeType::TextInput, params: serde_json::json!({}), position: None },
                PipelineNode { id: "b1".to_string(), name: "Branch 1".to_string(), node_type: NodeType::VoiceAssigner, params: serde_json::json!({}), position: None },
                PipelineNode { id: "b2".to_string(), name: "Branch 2".to_string(), node_type: NodeType::VoiceAssigner, params: serde_json::json!({}), position: None },
                PipelineNode { id: "out".to_string(), name: "Output".to_string(), node_type: NodeType::OutputSink, params: serde_json::json!({}), position: None },
            ],
            edges: vec![
                PipelineEdge { id: "e1".to_string(), from_node: "in".to_string(), to_node: "b1".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e2".to_string(), from_node: "in".to_string(), to_node: "b2".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e3".to_string(), from_node: "b1".to_string(), to_node: "out".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e4".to_string(), from_node: "b2".to_string(), to_node: "out".to_string(), from_port: None, to_port: None },
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let order = GraphValidator::topological_sort(&diamond_pipeline).expect("Topological sort must succeed");
        assert_eq!(order.len(), 4);
        assert_eq!(order[0], "in");
        assert_eq!(order[3], "out");
    }

    #[test]
    fn test_excessive_nodes_rejected() {
        let mut nodes = Vec::new();
        for i in 0..501 {
            nodes.push(PipelineNode {
                id: format!("node_{}", i),
                name: format!("Node {}", i),
                node_type: NodeType::TextInput,
                params: serde_json::json!({}),
                position: None,
            });
        }
        let pipeline = PipelineDefinition {
            id: Uuid::new_v4(),
            name: "Oversized".to_string(),
            description: None,
            nodes,
            edges: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = GraphValidator::topological_sort(&pipeline);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pipeline_dsp_filtering() {
        let registry = Arc::new(EngineRegistry::new());
        let mock_engine = Arc::new(MockTtsEngine::new(24000));
        registry.register(mock_engine).await;

        let executor = PipelineExecutor::new(registry);
        let pipeline = PipelineDefinition {
            id: Uuid::new_v4(),
            name: "DSP Pipeline Test".to_string(),
            description: None,
            nodes: vec![
                PipelineNode {
                    id: "in".to_string(),
                    name: "Input".to_string(),
                    node_type: NodeType::TextInput,
                    params: serde_json::json!({ "text": "DSP speech processing test." }),
                    position: None,
                },
                PipelineNode {
                    id: "parse".to_string(),
                    name: "Parser".to_string(),
                    node_type: NodeType::SpeakerParser,
                    params: serde_json::json!({}),
                    position: None,
                },
                PipelineNode {
                    id: "assign".to_string(),
                    name: "Assign".to_string(),
                    node_type: NodeType::VoiceAssigner,
                    params: serde_json::json!({ "default_voice": "mock-en-female" }),
                    position: None,
                },
                PipelineNode {
                    id: "synth".to_string(),
                    name: "Synth".to_string(),
                    node_type: NodeType::Synthesizer,
                    params: serde_json::json!({}),
                    position: None,
                },
                PipelineNode {
                    id: "dsp".to_string(),
                    name: "DSP Mastering".to_string(),
                    node_type: NodeType::AudioFilter,
                    params: serde_json::json!({
                        "eq": { "low_gain_db": 2.0, "mid_gain_db": -1.0, "high_gain_db": 1.5 },
                        "compressor": { "threshold_dbfs": -16.0, "ratio": 3.0, "attack_ms": 10.0, "release_ms": 80.0, "makeup_gain_db": 1.0 },
                        "limiter": { "ceiling_dbfs": -1.0 },
                        "silence_trim": { "threshold_dbfs": -40.0, "padding_ms": 20 }
                    }),
                    position: None,
                },
                PipelineNode {
                    id: "out".to_string(),
                    name: "Output".to_string(),
                    node_type: NodeType::OutputSink,
                    params: serde_json::json!({}),
                    position: None,
                },
            ],
            edges: vec![
                PipelineEdge { id: "e1".to_string(), from_node: "in".to_string(), to_node: "parse".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e2".to_string(), from_node: "parse".to_string(), to_node: "assign".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e3".to_string(), from_node: "assign".to_string(), to_node: "synth".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e4".to_string(), from_node: "synth".to_string(), to_node: "dsp".to_string(), from_port: None, to_port: None },
                PipelineEdge { id: "e5".to_string(), from_node: "dsp".to_string(), to_node: "out".to_string(), from_port: None, to_port: None },
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let wav = executor.execute(&pipeline, None).await.expect("DSP pipeline execution must succeed");
        assert!(!wav.is_empty());
        assert_eq!(&wav[0..4], b"RIFF");
    }
}
