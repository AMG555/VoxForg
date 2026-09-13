use crate::protocol::McpTool;

/// Returns the complete registry of MCP tool definitions provided by VoxForg.
pub fn get_mcp_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "synthesize_speech".to_string(),
            description: "Generate speech audio using local neural TTS engines (Edge-TTS, Kokoro, Piper, Qwen3-TTS) or cloned voice profiles.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text content to speak"
                    },
                    "voice": {
                        "type": "string",
                        "description": "Voice identifier, voice profile ID, or voice name (e.g. en-US-AriaNeural, mock-en-female, Alex Narrator)"
                    },
                    "speed": {
                        "type": "number",
                        "description": "Speaking rate multiplier (0.5 to 2.0). Default is 1.0"
                    },
                    "pitch": {
                        "type": "number",
                        "description": "Pitch shift in semitones (-12.0 to 12.0). Default is 0.0"
                    },
                    "response_format": {
                        "type": "string",
                        "enum": ["wav", "pcm"],
                        "description": "Audio container format. Default is wav"
                    }
                },
                "required": ["text", "voice"]
            }),
        },
        McpTool {
            name: "clone_voice".to_string(),
            description: "Extract high-dimensional speaker embeddings from reference audio to create a persistent cloned voice profile.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Display name for the new cloned voice profile"
                    },
                    "reference_audio_base64": {
                        "type": "string",
                        "description": "Hex or Base64-encoded reference audio clip (WAV or 16-bit PCM)"
                    },
                    "engine_id": {
                        "type": "string",
                        "description": "Target cloning engine (e.g. qwen3-tts, mock-tts). Default: qwen3-tts"
                    },
                    "language": {
                        "type": "string",
                        "description": "Primary ISO language code (e.g. en-US). Default: en-US"
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional human-readable description of voice characteristics"
                    }
                },
                "required": ["name", "reference_audio_base64"]
            }),
        },
        McpTool {
            name: "transcribe_audio".to_string(),
            description: "Transcribe audio speech into text or timed subtitles using neural Whisper ASR engines.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "audio_base64": {
                        "type": "string",
                        "description": "Base64 or hex-encoded 16-bit PCM or WAV container"
                    },
                    "model": {
                        "type": "string",
                        "description": "Target ASR model ID (e.g. whisper-base, mock-asr)"
                    },
                    "language": {
                        "type": "string",
                        "description": "Optional ISO language code hint (e.g. en, es)"
                    },
                    "response_format": {
                        "type": "string",
                        "enum": ["json", "verbose_json", "srt", "vtt", "text"],
                        "description": "Transcription output serialization format. Default: json"
                    }
                },
                "required": ["audio_base64"]
            }),
        },
        McpTool {
            name: "execute_pipeline".to_string(),
            description: "Execute a pre-built production speech DAG pipeline (e.g. video_dubbing, audiobook) or custom script.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "template": {
                        "type": "string",
                        "enum": ["video_dubbing", "audiobook"],
                        "description": "Pre-built template to run"
                    },
                    "title": {
                        "type": "string",
                        "description": "Title / name of the pipeline job"
                    },
                    "voice": {
                        "type": "string",
                        "description": "Primary voice identifier to use for the pipeline"
                    },
                    "input_text": {
                        "type": "string",
                        "description": "Input script, manuscript, or dialogue text"
                    }
                },
                "required": ["template"]
            }),
        },
        McpTool {
            name: "list_voices".to_string(),
            description: "List all available voices across registered TTS engines, portable voice identities, and persistent cloned profiles.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "description": "Optional language filter (e.g. en, es, fr)"
                    }
                }
            }),
        },
        McpTool {
            name: "list_models".to_string(),
            description: "Browse the local open-weights model catalogue (Piper, Kokoro, Whisper, Qwen3, Silero).".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "type": {
                        "type": "string",
                        "enum": ["tts", "asr", "vad", "diarizer"],
                        "description": "Filter by model domain"
                    },
                    "installed_only": {
                        "type": "boolean",
                        "description": "Filter to only locally installed weight packages"
                    }
                }
            }),
        },
        McpTool {
            name: "benchmark_engine".to_string(),
            description: "Trigger latency, throughput, and RTF benchmarking on an engine using the standard test sentence suite.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "engine_id": {
                        "type": "string",
                        "description": "Engine identifier to benchmark (e.g. edge-tts, mock-tts, qwen3-tts)"
                    }
                }
            }),
        },
        McpTool {
            name: "get_cluster_status".to_string(),
            description: "Inspect local hardware accelerator profile (CPU features, RAM, GPU) and distributed worker nodes.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}
