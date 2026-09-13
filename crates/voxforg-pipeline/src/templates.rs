use chrono::Utc;
use uuid::Uuid;
use voxforg_core::models::{NodeType, PipelineDefinition, PipelineEdge, PipelineNode};

/// Pre-built production workflow pipeline templates.
pub struct PipelineTemplates;

impl PipelineTemplates {
    /// Constructs a production Video Dubbing DAG pipeline:
    /// Audio/Text Input -> ASR Transcriber -> Diarization -> Voice Assigner ->
    /// Synthesizer -> Time Stretch Aligner -> Audio Muxer -> Output Sink.
    pub fn video_dubbing(name: &str, default_voice: &str) -> PipelineDefinition {
        let nodes = vec![
            PipelineNode {
                id: "node_input".to_string(),
                name: "Audio / Script Ingestion".to_string(),
                node_type: NodeType::TextInput,
                params: serde_json::json!({
                    "text": "Alice: Welcome to the future of speech.\nBob: All signals are operational."
                }),
                position: None,
            },
            PipelineNode {
                id: "node_asr".to_string(),
                name: "ASR Transcriber".to_string(),
                node_type: NodeType::AsrTranscriber,
                params: serde_json::json!({
                    "model": "whisper-base",
                    "word_timestamps": true
                }),
                position: None,
            },
            PipelineNode {
                id: "node_diarize".to_string(),
                name: "Speaker Diarization".to_string(),
                node_type: NodeType::Diarization,
                params: serde_json::json!({
                    "num_speakers": 2,
                    "default_speaker": "Narrator"
                }),
                position: None,
            },
            PipelineNode {
                id: "node_voice_assign".to_string(),
                name: "Voice Profile Assigner".to_string(),
                node_type: NodeType::VoiceAssigner,
                params: serde_json::json!({
                    "default_voice": default_voice,
                    "speaker_map": {
                        "Alice": default_voice,
                        "SPEAKER_00": default_voice
                    }
                }),
                position: None,
            },
            PipelineNode {
                id: "node_synth".to_string(),
                name: "Multi-Voice Synthesizer".to_string(),
                node_type: NodeType::Synthesizer,
                params: serde_json::json!({}),
                position: None,
            },
            PipelineNode {
                id: "node_timestretch".to_string(),
                name: "Audio Time Stretch Aligner".to_string(),
                node_type: NodeType::AudioTimeStretch,
                params: serde_json::json!({
                    "match_subtitles": true,
                    "max_stretch_factor": 1.4
                }),
                position: None,
            },
            PipelineNode {
                id: "node_mux".to_string(),
                name: "Audio Track Muxer".to_string(),
                node_type: NodeType::AudioMux,
                params: serde_json::json!({
                    "voice_volume": 1.0,
                    "background_volume": 0.15,
                    "ducking": true
                }),
                position: None,
            },
            PipelineNode {
                id: "node_sink".to_string(),
                name: "Master Audio Output".to_string(),
                node_type: NodeType::OutputSink,
                params: serde_json::json!({}),
                position: None,
            },
        ];

        let edges = vec![
            PipelineEdge {
                id: "e1".to_string(),
                from_node: "node_input".to_string(),
                to_node: "node_asr".to_string(),
                from_port: None,
                to_port: None,
            },
            PipelineEdge {
                id: "e2".to_string(),
                from_node: "node_asr".to_string(),
                to_node: "node_diarize".to_string(),
                from_port: None,
                to_port: None,
            },
            PipelineEdge {
                id: "e3".to_string(),
                from_node: "node_diarize".to_string(),
                to_node: "node_voice_assign".to_string(),
                from_port: None,
                to_port: None,
            },
            PipelineEdge {
                id: "e4".to_string(),
                from_node: "node_voice_assign".to_string(),
                to_node: "node_synth".to_string(),
                from_port: None,
                to_port: None,
            },
            PipelineEdge {
                id: "e5".to_string(),
                from_node: "node_synth".to_string(),
                to_node: "node_timestretch".to_string(),
                from_port: None,
                to_port: None,
            },
            PipelineEdge {
                id: "e6".to_string(),
                from_node: "node_timestretch".to_string(),
                to_node: "node_mux".to_string(),
                from_port: None,
                to_port: None,
            },
            PipelineEdge {
                id: "e7".to_string(),
                from_node: "node_mux".to_string(),
                to_node: "node_sink".to_string(),
                from_port: None,
                to_port: None,
            },
        ];

        PipelineDefinition {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: Some("Automated speech recognition, speaker diarization, voice matching, time alignment, and audio mix".to_string()),
            nodes,
            edges,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Constructs a production Audiobook Mastering DAG pipeline:
    /// Text Document -> Document Chunker (Chapters & Dialogue) -> Voice Assigner ->
    /// Synthesizer -> Audio Filter (EBU R128 mastering) -> Audio Merge -> Output Sink.
    pub fn audiobook(title: &str, narrator_voice: &str) -> PipelineDefinition {
        let nodes = vec![
            PipelineNode {
                id: "node_text".to_string(),
                name: "Manuscript Input".to_string(),
                node_type: NodeType::TextInput,
                params: serde_json::json!({
                    "text": "Chapter 1: The Departure.\nThe morning air was crisp and clear.\n\"Are we ready to sail?\" asked Elena.\n\"Every sail is set,\" replied the Captain."
                }),
                position: None,
            },
            PipelineNode {
                id: "node_chunker".to_string(),
                name: "Document & Chapter Chunker".to_string(),
                node_type: NodeType::DocumentChunker,
                params: serde_json::json!({
                    "chapter_regex": r"(?i)Chapter\s+\d+",
                    "dialogue_quotes": true,
                    "default_speaker": "Narrator"
                }),
                position: None,
            },
            PipelineNode {
                id: "node_voice".to_string(),
                name: "Character Voice Assigner".to_string(),
                node_type: NodeType::VoiceAssigner,
                params: serde_json::json!({
                    "default_voice": narrator_voice,
                    "speaker_map": {
                        "Narrator": narrator_voice,
                        "Elena": narrator_voice,
                        "Captain": narrator_voice
                    }
                }),
                position: None,
            },
            PipelineNode {
                id: "node_synth".to_string(),
                name: "Chapter Synthesizer".to_string(),
                node_type: NodeType::Synthesizer,
                params: serde_json::json!({}),
                position: None,
            },
            PipelineNode {
                id: "node_filter".to_string(),
                name: "EBU R128 Mastering Filter".to_string(),
                node_type: NodeType::AudioFilter,
                params: serde_json::json!({
                    "normalize_peak": 0.95,
                    "trim_silence": true,
                    "limiter": {
                        "ceiling_dbfs": -1.0
                    }
                }),
                position: None,
            },
            PipelineNode {
                id: "node_merge".to_string(),
                name: "Chapter Sticher".to_string(),
                node_type: NodeType::AudioMerge,
                params: serde_json::json!({
                    "pause_ms": 300
                }),
                position: None,
            },
            PipelineNode {
                id: "node_sink".to_string(),
                name: "Master Audio Sink".to_string(),
                node_type: NodeType::OutputSink,
                params: serde_json::json!({}),
                position: None,
            },
        ];

        let edges = vec![
            PipelineEdge {
                id: "e1".to_string(),
                from_node: "node_text".to_string(),
                to_node: "node_chunker".to_string(),
                from_port: None,
                to_port: None,
            },
            PipelineEdge {
                id: "e2".to_string(),
                from_node: "node_chunker".to_string(),
                to_node: "node_voice".to_string(),
                from_port: None,
                to_port: None,
            },
            PipelineEdge {
                id: "e3".to_string(),
                from_node: "node_voice".to_string(),
                to_node: "node_synth".to_string(),
                from_port: None,
                to_port: None,
            },
            PipelineEdge {
                id: "e4".to_string(),
                from_node: "node_synth".to_string(),
                to_node: "node_filter".to_string(),
                from_port: None,
                to_port: None,
            },
            PipelineEdge {
                id: "e5".to_string(),
                from_node: "node_filter".to_string(),
                to_node: "node_merge".to_string(),
                from_port: None,
                to_port: None,
            },
            PipelineEdge {
                id: "e6".to_string(),
                from_node: "node_merge".to_string(),
                to_node: "node_sink".to_string(),
                from_port: None,
                to_port: None,
            },
        ];

        PipelineDefinition {
            id: Uuid::new_v4(),
            name: format!("Audiobook Pipeline: {title}"),
            description: Some(
                "Chapter chunking, multi-character voice assignment, and EBU R128 audio mastering"
                    .to_string(),
            ),
            nodes,
            edges,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
