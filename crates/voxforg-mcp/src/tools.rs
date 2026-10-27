use crate::protocol::{McpPrompt, McpPromptArgument, McpResource, McpTool};

/// Returns the complete registry of MCP tool definitions provided by VoxForg.
pub fn get_mcp_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "synthesize_speech".to_string(),
            description: "Generate speech audio using local neural TTS engines (Edge-TTS, Kokoro, Piper, Qwen3-TTS) or cloned voice profiles. Supports directly writing the output audio file to disk.".to_string(),
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
                        "description": "Speaking rate multiplier (0.25 to 4.0). Default is 1.0"
                    },
                    "pitch": {
                        "type": "number",
                        "description": "Pitch shift in semitones (-12.0 to 12.0). Default is 0.0"
                    },
                    "response_format": {
                        "type": "string",
                        "enum": ["wav", "pcm"],
                        "description": "Audio container format. Default is wav"
                    },
                    "output_file": {
                        "type": "string",
                        "description": "Optional local file path to save the generated audio (.wav or .pcm). Avoids returning large hex/base64 payload in model context."
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
                        "description": "Base64 or hex-encoded reference audio clip (WAV or 16-bit PCM)"
                    },
                    "reference_audio_path": {
                        "type": "string",
                        "description": "Local filesystem path to reference audio file (.wav, .mp3, etc.)"
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
                    },
                    "reference_transcript": {
                        "type": "string",
                        "description": "Optional exact text transcript of reference audio to improve cloning fidelity"
                    }
                },
                "required": ["name"]
            }),
        },
        McpTool {
            name: "transcribe_audio".to_string(),
            description: "Transcribe audio speech into text or timed subtitles using neural Whisper ASR engines. Accepts file path or base64 data.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "audio_base64": {
                        "type": "string",
                        "description": "Base64 or hex-encoded 16-bit PCM or WAV container"
                    },
                    "audio_path": {
                        "type": "string",
                        "description": "Local filesystem path to input audio or video file (.wav, .mp3, .mp4, .mkv, .webm)"
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
                }
            }),
        },
        McpTool {
            name: "speech_to_speech".to_string(),
            description: "Convert a source voice audio recording into a target voice persona while preserving timing and cadence.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "source_audio_base64": {
                        "type": "string",
                        "description": "Base64 or hex-encoded source speech audio"
                    },
                    "source_audio_path": {
                        "type": "string",
                        "description": "Local filesystem path to source speech audio file"
                    },
                    "target_voice": {
                        "type": "string",
                        "description": "Target voice identifier or profile to convert into"
                    },
                    "speed": {
                        "type": "number",
                        "description": "Speed factor (0.5 to 2.5). Default: 1.0"
                    },
                    "preserve_tempo": {
                        "type": "boolean",
                        "description": "Whether to preserve the exact duration of the source speech"
                    },
                    "output_file": {
                        "type": "string",
                        "description": "Optional path to save the converted audio directly to disk (.wav)"
                    }
                },
                "required": ["target_voice"]
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
                    },
                    "output_file": {
                        "type": "string",
                        "description": "Optional path to save the resulting audio (.wav)"
                    }
                },
                "required": ["template"]
            }),
        },
        McpTool {
            name: "manage_pronunciation".to_string(),
            description: "Manage custom phoneme and word pronunciation dictionary (e.g. SQL -> sequel, domain acronyms).".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list", "add", "remove", "apply"],
                        "description": "Pronunciation dictionary action"
                    },
                    "term": {
                        "type": "string",
                        "description": "Original term or word (for add or remove)"
                    },
                    "replacement": {
                        "type": "string",
                        "description": "Spoken replacement or phoneme string (for add)"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to normalize through pronunciation dictionary (for apply)"
                    }
                },
                "required": ["action"]
            }),
        },
        McpTool {
            name: "refine_text".to_string(),
            description: "Clean transcribed or draft speech text by removing conversational filler words ('um', 'uh') and Whisper hallucination loops.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to refine and clean"
                    },
                    "strip_fillers": {
                        "type": "boolean",
                        "description": "Strip verbal filler words (default: true)"
                    },
                    "strip_hallucinations": {
                        "type": "boolean",
                        "description": "Strip repetitive hallucination loops (default: true)"
                    }
                },
                "required": ["text"]
            }),
        },
        McpTool {
            name: "embed_watermark".to_string(),
            description: "Embed an invisible, spread-spectrum provenance watermark into a WAV audio file or base64 stream.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "audio_base64": {
                        "type": "string",
                        "description": "Base64 or hex-encoded WAV audio data"
                    },
                    "audio_path": {
                        "type": "string",
                        "description": "Local filesystem path to WAV audio file"
                    },
                    "payload": {
                        "type": "integer",
                        "description": "Optional 16-bit payload signature (0 - 65535). Default: 0x5658"
                    },
                    "output_file": {
                        "type": "string",
                        "description": "Optional path to save watermarked WAV audio file"
                    }
                }
            }),
        },
        McpTool {
            name: "verify_watermark".to_string(),
            description: "Detect and verify the presence of an invisible cryptographic provenance watermark in an audio recording.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "audio_base64": {
                        "type": "string",
                        "description": "Base64 or hex-encoded WAV audio data"
                    },
                    "audio_path": {
                        "type": "string",
                        "description": "Local filesystem path to WAV audio file"
                    }
                }
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
            name: "install_model".to_string(),
            description: "Download and install an open-weights model package into the local cache directory.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "model_id": {
                        "type": "string",
                        "description": "Model identifier to install (e.g. kokoro-v0_19, piper-en-lessac-medium)"
                    }
                },
                "required": ["model_id"]
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

/// Returns the complete registry of MCP resources available for context inspection.
pub fn get_mcp_resources() -> Vec<McpResource> {
    vec![
        McpResource {
            uri: "voxforg://voices".to_string(),
            name: "Available Voices & Profiles".to_string(),
            mime_type: Some("application/json".to_string()),
            description: Some("Catalogue of all registered TTS voices, language coverage, and cloned profiles".to_string()),
        },
        McpResource {
            uri: "voxforg://models".to_string(),
            name: "Model Weights Catalogue".to_string(),
            mime_type: Some("application/json".to_string()),
            description: Some("Open-weights packages (Piper, Kokoro, Whisper, Qwen3) and installation state".to_string()),
        },
        McpResource {
            uri: "voxforg://hardware".to_string(),
            name: "Hardware Acceleration Profile".to_string(),
            mime_type: Some("application/json".to_string()),
            description: Some("Host CPU SIMD extensions, physical RAM, GPU accelerators, and VRAM budget".to_string()),
        },
        McpResource {
            uri: "voxforg://cluster".to_string(),
            name: "Distributed Cluster Nodes".to_string(),
            mime_type: Some("application/json".to_string()),
            description: Some("Registered distributed worker nodes, status, and health metrics".to_string()),
        },
    ]
}

/// Returns pre-built prompt workflows for agent consumption.
pub fn get_mcp_prompts() -> Vec<McpPrompt> {
    vec![
        McpPrompt {
            name: "speech_production".to_string(),
            description: "Prepare and synthesize production-ready narration audio with dictionary normalization".to_string(),
            arguments: vec![
                McpPromptArgument {
                    name: "script".to_string(),
                    description: "Script text to speak".to_string(),
                    required: true,
                },
                McpPromptArgument {
                    name: "voice".to_string(),
                    description: "Voice identifier to use".to_string(),
                    required: false,
                },
            ],
        },
        McpPrompt {
            name: "audio_transcription_and_dubbing".to_string(),
            description: "Transcribe source media and dub into target language/voice using DAG pipeline".to_string(),
            arguments: vec![
                McpPromptArgument {
                    name: "media_path".to_string(),
                    description: "Path to video or audio file".to_string(),
                    required: true,
                },
                McpPromptArgument {
                    name: "target_voice".to_string(),
                    description: "Target voice identifier".to_string(),
                    required: true,
                },
            ],
        },
    ]
}
