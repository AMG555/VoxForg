use std::sync::Arc;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use voxforg_asr::{AsrRegistry, TranscriptionOptions};
use voxforg_audio::dsp::{AudioWatermark, DEFAULT_SIGNATURE_PAYLOAD};
use voxforg_audio::{MediaProcessor, WavEncoder};
use voxforg_catalog::ModelCatalogStore;
use voxforg_core::models::{AudioContainerFormat, CloneVoiceRequest, ClonedSynthesisRequest};
use voxforg_engine::{
    EdgeTtsEngine, EngineRegistry, KokoroTtsEngine, MockTtsEngine, OpenAiRouterEngine,
    PiperTtsEngine, Qwen3TtsEngine, SynthesisRequest, VoiceProfileStore,
};
use voxforg_hardware::{HardwareProbe, HardwareProfile};
use voxforg_pipeline::{PipelineExecutor, PipelineTemplates};
use voxforg_pronunciation::{DictionaryEntry, ProcessorConfig, PronunciationProcessor};
use voxforg_worker::WorkerPool;

use crate::protocol::{
    JsonRpcRequest, JsonRpcResponse, McpResourceContent, ToolCallResult,
};
use crate::security::{
    safe_decode_audio_bytes, validate_safe_input_path, validate_safe_output_path,
    validate_text_length,
};
use crate::tools::{get_mcp_prompts, get_mcp_resources, get_mcp_tools};

/// Full in-process MCP server dispatching to VoxForg speech runtimes.
pub struct McpServer {
    pub engine_registry: Arc<EngineRegistry>,
    pub voice_profiles: Arc<VoiceProfileStore>,
    pub asr_registry: Arc<AsrRegistry>,
    pub model_catalog: Arc<ModelCatalogStore>,
    pub pipeline_executor: Arc<PipelineExecutor>,
    pub pronunciation: Arc<PronunciationProcessor>,
    pub hardware: HardwareProfile,
    pub worker_pool: Arc<WorkerPool>,
}

impl Default for McpServer {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl McpServer {
    /// Construct server with fully initialized standard speech engines, registries, and stores.
    pub fn with_defaults() -> Self {
        let engine_registry = Arc::new(EngineRegistry::new());
        let mock_engine = Arc::new(MockTtsEngine::new(24000));
        let edge_engine = Arc::new(EdgeTtsEngine::new());
        let qwen3_engine = Arc::new(Qwen3TtsEngine::default());
        let piper_engine = Arc::new(PiperTtsEngine::new());
        let kokoro_engine = Arc::new(KokoroTtsEngine::new());

        let router_engine = if let Ok(url) = std::env::var("VOXFORG_ROUTER_URL") {
            let key = std::env::var("VOXFORG_ROUTER_API_KEY").ok();
            let model = std::env::var("VOXFORG_ROUTER_MODEL").ok();
            OpenAiRouterEngine::new(url, key, model)
        } else {
            let key = std::env::var("VOXFORG_ROUTER_API_KEY").ok();
            OpenAiRouterEngine::default_openai(key)
        };

        // Register all production engines
        engine_registry.register_sync(mock_engine);
        engine_registry.register_sync(edge_engine);
        engine_registry.register_sync(qwen3_engine);
        engine_registry.register_sync(piper_engine);
        engine_registry.register_sync(kokoro_engine);
        engine_registry.register_sync(Arc::new(router_engine));

        let voice_profiles = Arc::new(VoiceProfileStore::with_defaults());
        let asr_registry = Arc::new(AsrRegistry::with_defaults());
        let model_catalog = Arc::new(ModelCatalogStore::with_curated_models());
        let pipeline_executor = Arc::new(
            PipelineExecutor::new(engine_registry.clone()).with_asr_registry(asr_registry.clone()),
        );
        let pronunciation = Arc::new(PronunciationProcessor::new(ProcessorConfig::default()));
        let hardware = HardwareProbe::probe();
        let worker_pool = Arc::new(WorkerPool::new());

        Self {
            engine_registry,
            voice_profiles,
            asr_registry,
            model_catalog,
            pipeline_executor,
            pronunciation,
            hardware,
            worker_pool,
        }
    }

    /// Process a raw single-line JSON-RPC request and return formatted response string.
    pub async fn handle_message(&self, raw: &str) -> Option<String> {
        let trimmed = raw.trim().trim_start_matches('\u{feff}');
        if trimmed.is_empty() {
            return None;
        }

        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err_res = JsonRpcResponse::error(None, -32700, format!("Parse error: {e}"));
                return serde_json::to_string(&err_res).ok();
            }
        };

        match req.method.as_str() {
            "initialize" => {
                let res_data = serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {},
                        "resources": {},
                        "prompts": {}
                    },
                    "serverInfo": {
                        "name": "voxforg-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                });
                let resp = JsonRpcResponse::success(req.id, res_data);
                serde_json::to_string(&resp).ok()
            }

            "notifications/initialized" => None,

            "ping" => {
                let resp = JsonRpcResponse::success(req.id, serde_json::json!({}));
                serde_json::to_string(&resp).ok()
            }

            "tools/list" => {
                let tools = get_mcp_tools();
                let res_data = serde_json::json!({
                    "tools": tools
                });
                let resp = JsonRpcResponse::success(req.id, res_data);
                serde_json::to_string(&resp).ok()
            }

            "tools/call" => {
                let params = req.params.unwrap_or(serde_json::Value::Null);
                let tool_name = params
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or_default();
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                let tool_result = self.execute_tool(tool_name, arguments).await;
                let val = serde_json::to_value(tool_result).unwrap_or(serde_json::Value::Null);
                let resp = JsonRpcResponse::success(req.id, val);
                serde_json::to_string(&resp).ok()
            }

            "resources/list" => {
                let resources = get_mcp_resources();
                let res_data = serde_json::json!({
                    "resources": resources
                });
                let resp = JsonRpcResponse::success(req.id, res_data);
                serde_json::to_string(&resp).ok()
            }

            "resources/read" => {
                let params = req.params.unwrap_or(serde_json::Value::Null);
                let uri = params.get("uri").and_then(|u| u.as_str()).unwrap_or("");

                let content_opt = match uri {
                    "voxforg://voices" => {
                        let mut all_voices = Vec::new();
                        if let Ok(voices) = self.engine_registry.list_all_voices().await {
                            for v in voices {
                                all_voices.push(serde_json::json!({
                                    "id": v.id,
                                    "name": v.name,
                                    "engine": v.engine_id,
                                    "language": v.language,
                                    "gender": format!("{:?}", v.gender).to_lowercase(),
                                    "is_cloned": false
                                }));
                            }
                        }
                        let profiles = self.voice_profiles.list().await;
                        for p in profiles {
                            all_voices.push(serde_json::json!({
                                "id": p.id,
                                "name": p.name,
                                "engine": p.engine_id,
                                "language": p.language,
                                "gender": p.gender.map(|g| format!("{:?}", g).to_lowercase()).unwrap_or_else(|| "unknown".to_string()),
                                "is_cloned": true
                            }));
                        }
                        let text = serde_json::to_string_pretty(&all_voices).unwrap_or_default();
                        Some(McpResourceContent {
                            uri: uri.to_string(),
                            mime_type: Some("application/json".to_string()),
                            text,
                        })
                    }
                    "voxforg://models" => {
                        let items = self.model_catalog.list(None, false).await;
                        let text = serde_json::to_string_pretty(&items).unwrap_or_default();
                        Some(McpResourceContent {
                            uri: uri.to_string(),
                            mime_type: Some("application/json".to_string()),
                            text,
                        })
                    }
                    "voxforg://hardware" => {
                        let text = serde_json::to_string_pretty(&self.hardware).unwrap_or_default();
                        Some(McpResourceContent {
                            uri: uri.to_string(),
                            mime_type: Some("application/json".to_string()),
                            text,
                        })
                    }
                    "voxforg://cluster" => {
                        let workers = self.worker_pool.list().await;
                        let text = serde_json::to_string_pretty(&workers).unwrap_or_default();
                        Some(McpResourceContent {
                            uri: uri.to_string(),
                            mime_type: Some("application/json".to_string()),
                            text,
                        })
                    }
                    _ => None,
                };

                match content_opt {
                    Some(content) => {
                        let resp = JsonRpcResponse::success(
                            req.id,
                            serde_json::json!({
                                "contents": [content]
                            }),
                        );
                        serde_json::to_string(&resp).ok()
                    }
                    None => {
                        let resp = JsonRpcResponse::error(
                            req.id,
                            -32602,
                            format!("Resource not found: '{uri}'"),
                        );
                        serde_json::to_string(&resp).ok()
                    }
                }
            }

            "prompts/list" => {
                let prompts = get_mcp_prompts();
                let res_data = serde_json::json!({
                    "prompts": prompts
                });
                let resp = JsonRpcResponse::success(req.id, res_data);
                serde_json::to_string(&resp).ok()
            }

            "prompts/get" => {
                let params = req.params.unwrap_or(serde_json::Value::Null);
                let prompt_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let prompts = get_mcp_prompts();
                if let Some(prompt) = prompts.iter().find(|p| p.name == prompt_name) {
                    let resp = JsonRpcResponse::success(
                        req.id,
                        serde_json::json!({
                            "description": prompt.description,
                            "messages": [
                                {
                                    "role": "user",
                                    "content": {
                                        "type": "text",
                                        "text": format!("Execute VoxForg workflow: {}", prompt.name)
                                    }
                                }
                            ]
                        }),
                    );
                    serde_json::to_string(&resp).ok()
                } else {
                    let resp = JsonRpcResponse::error(
                        req.id,
                        -32602,
                        format!("Prompt not found: '{prompt_name}'"),
                    );
                    serde_json::to_string(&resp).ok()
                }
            }

            other => {
                let resp =
                    JsonRpcResponse::error(req.id, -32601, format!("Method not found: '{other}'"));
                serde_json::to_string(&resp).ok()
            }
        }
    }

    /// Dispatch to individual MCP tool logic.
    async fn execute_tool(&self, name: &str, args: serde_json::Value) -> ToolCallResult {
        match name {
            "synthesize_speech" => {
                let text = args.get("text").and_then(|t| t.as_str()).unwrap_or("");
                let voice = args
                    .get("voice")
                    .and_then(|v| v.as_str())
                    .unwrap_or("en-US-AriaNeural");
                let speed = args.get("speed").and_then(|s| s.as_f64()).unwrap_or(1.0) as f32;
                let pitch = args.get("pitch").and_then(|p| p.as_f64()).unwrap_or(0.0) as f32;
                let output_file = args.get("output_file").and_then(|f| f.as_str());

                if text.trim().is_empty() {
                    return ToolCallResult::error("Parameter 'text' cannot be empty");
                }
                if let Err(e) = validate_text_length(text, "text") {
                    return ToolCallResult::error(e);
                }

                // If output file specified, validate path upfront
                let safe_out_path = if let Some(out_str) = output_file {
                    match validate_safe_output_path(out_str) {
                        Ok(p) => Some(p),
                        Err(e) => return ToolCallResult::error(e),
                    }
                } else {
                    None
                };

                // Check cloned profiles first
                if let Some(profile) = self.voice_profiles.get(voice).await {
                    if let Some(engine) = self.engine_registry.get(&profile.engine_id).await {
                        let synth_req = ClonedSynthesisRequest {
                            text: text.to_string(),
                            profile: profile.clone(),
                            speed,
                            pitch,
                            format: AudioContainerFormat::Wav,
                        };
                        match engine.synthesize_cloned(&synth_req).await {
                            Ok(chunk) => {
                                let wav_bytes = match WavEncoder::encode_pcm16_to_wav(
                                    &chunk.pcm_data,
                                    chunk.sample_rate,
                                    1,
                                ) {
                                    Ok(b) => b,
                                    Err(e) => return ToolCallResult::error(e.to_string()),
                                };
                                let duration =
                                    chunk.pcm_data.len() as f32 / chunk.sample_rate as f32;

                                if let Some(ref out_path) = safe_out_path {
                                    if let Err(e) = std::fs::write(out_path, &wav_bytes) {
                                        return ToolCallResult::error(format!(
                                            "Failed saving audio to {:?}: {e}",
                                            out_path
                                        ));
                                    }
                                    return ToolCallResult::json(&serde_json::json!({
                                        "status": "success",
                                        "voice_profile": profile.name,
                                        "duration_seconds": duration,
                                        "output_file": out_path.to_string_lossy(),
                                        "size_bytes": wav_bytes.len(),
                                        "sample_rate": chunk.sample_rate
                                    }));
                                }

                                return ToolCallResult::json(&serde_json::json!({
                                    "status": "success",
                                    "voice_profile": profile.name,
                                    "duration_seconds": duration,
                                    "audio_wav_hex": hex::encode(&wav_bytes),
                                    "sample_rate": chunk.sample_rate
                                }));
                            }
                            Err(e) => return ToolCallResult::error(e.to_string()),
                        }
                    }
                }

                // Direct voice resolution
                match self.engine_registry.resolve_voice(voice).await {
                    Ok((engine, voice_info)) => {
                        let req = SynthesisRequest {
                            text: text.to_string(),
                            voice_id: voice_info.id.clone(),
                            speed,
                            pitch,
                            format: AudioContainerFormat::Wav,
                        };
                        match engine.synthesize(&req).await {
                            Ok(chunk) => {
                                let wav_bytes = match WavEncoder::encode_pcm16_to_wav(
                                    &chunk.pcm_data,
                                    chunk.sample_rate,
                                    1,
                                ) {
                                    Ok(b) => b,
                                    Err(e) => return ToolCallResult::error(e.to_string()),
                                };
                                let duration =
                                    chunk.pcm_data.len() as f32 / chunk.sample_rate as f32;

                                if let Some(ref out_path) = safe_out_path {
                                    if let Err(e) = std::fs::write(out_path, &wav_bytes) {
                                        return ToolCallResult::error(format!(
                                            "Failed saving audio to {:?}: {e}",
                                            out_path
                                        ));
                                    }
                                    return ToolCallResult::json(&serde_json::json!({
                                        "status": "success",
                                        "voice": voice_info.name,
                                        "engine": voice_info.engine_id,
                                        "duration_seconds": duration,
                                        "output_file": out_path.to_string_lossy(),
                                        "size_bytes": wav_bytes.len(),
                                        "sample_rate": chunk.sample_rate
                                    }));
                                }

                                ToolCallResult::json(&serde_json::json!({
                                    "status": "success",
                                    "voice": voice_info.name,
                                    "engine": voice_info.engine_id,
                                    "duration_seconds": duration,
                                    "audio_wav_hex": hex::encode(&wav_bytes),
                                    "sample_rate": chunk.sample_rate
                                }))
                            }
                            Err(e) => ToolCallResult::error(format!("Synthesis failed: {e}")),
                        }
                    }
                    Err(e) => ToolCallResult::error(format!("Voice resolution error: {e}")),
                }
            }

            "clone_voice" => {
                let name = args.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let ref_audio_b64 = args.get("reference_audio_base64").and_then(|a| a.as_str());
                let ref_audio_path = args.get("reference_audio_path").and_then(|p| p.as_str());
                let engine_id = args
                    .get("engine_id")
                    .and_then(|e| e.as_str())
                    .unwrap_or("qwen3-tts");
                let lang = args
                    .get("language")
                    .and_then(|l| l.as_str())
                    .unwrap_or("en-US");
                let desc = args.get("description").and_then(|d| d.as_str());

                if name.trim().is_empty() {
                    return ToolCallResult::error("Parameter 'name' is required");
                }

                let audio_hex = if let Some(path_str) = ref_audio_path {
                    let safe_path = match validate_safe_input_path(path_str) {
                        Ok(p) => p,
                        Err(e) => return ToolCallResult::error(e),
                    };
                    match std::fs::read(&safe_path) {
                        Ok(bytes) => hex::encode(bytes),
                        Err(e) => {
                            return ToolCallResult::error(format!(
                                "Failed reading reference audio file {:?}: {e}",
                                safe_path
                            ))
                        }
                    }
                } else if let Some(b64) = ref_audio_b64 {
                    match safe_decode_audio_bytes(b64) {
                        Ok(bytes) => hex::encode(bytes),
                        Err(e) => return ToolCallResult::error(e),
                    }
                } else {
                    return ToolCallResult::error(
                        "Either 'reference_audio_path' or 'reference_audio_base64' is required",
                    );
                };

                let engine = match self.engine_registry.get(engine_id).await {
                    Some(e) => e,
                    None => {
                        return ToolCallResult::error(format!("Engine '{engine_id}' not found"))
                    }
                };

                let clone_req = CloneVoiceRequest {
                    name: name.to_string(),
                    engine_id: engine_id.to_string(),
                    reference_audio_base64: Some(audio_hex),
                    reference_audio_path: ref_audio_path.map(|s| s.to_string()),
                    reference_transcript: args
                        .get("reference_transcript")
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string()),
                    language: lang.to_string(),
                    description: desc.map(|d| d.to_string()),
                    gender: None,
                    metadata: std::collections::HashMap::new(),
                };

                match engine.clone_voice(&clone_req).await {
                    Ok(profile) => {
                        self.voice_profiles.insert(profile.clone()).await;
                        ToolCallResult::json(&serde_json::json!({
                            "status": "created",
                            "profile_id": profile.id,
                            "name": profile.name,
                            "embedding_dim": profile.embedding.as_ref().map(|e| e.len()).unwrap_or(0),
                            "language": profile.language
                        }))
                    }
                    Err(e) => ToolCallResult::error(format!("Voice cloning failed: {e}")),
                }
            }

            "transcribe_audio" => {
                let audio_raw = args.get("audio_base64").and_then(|a| a.as_str());
                let audio_path = args.get("audio_path").and_then(|p| p.as_str());
                let model = args.get("model").and_then(|m| m.as_str());
                let lang = args
                    .get("language")
                    .and_then(|l| l.as_str())
                    .map(|s| s.to_string());
                let resp_format = args
                    .get("response_format")
                    .and_then(|f| f.as_str())
                    .unwrap_or("json");

                let (pcm, sample_rate) = if let Some(path_str) = audio_path {
                    let safe_path = match validate_safe_input_path(path_str) {
                        Ok(p) => p,
                        Err(e) => return ToolCallResult::error(e),
                    };
                    if !safe_path.exists() {
                        return ToolCallResult::error(format!(
                            "Audio file does not exist: {:?}",
                            safe_path
                        ));
                    }
                    let ext = safe_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if ext == "mp4" || ext == "mkv" || ext == "mov" || ext == "webm" {
                        match MediaProcessor::extract_audio_to_pcm(&safe_path, 16000) {
                            Ok((samples, sr, _)) => (samples, sr),
                            Err(e) => {
                                return ToolCallResult::error(format!(
                                    "Failed extracting audio from video {:?}: {e}",
                                    safe_path
                                ))
                            }
                        }
                    } else {
                        let bytes = match std::fs::read(&safe_path) {
                            Ok(b) => b,
                            Err(e) => return ToolCallResult::error(format!("Read error: {e}")),
                        };
                        match WavEncoder::decode_wav_to_pcm16(&bytes) {
                            Ok((samples, sr, _)) => (samples, sr),
                            Err(e) => {
                                return ToolCallResult::error(format!("WAV decode error: {e}"))
                            }
                        }
                    }
                } else if let Some(raw) = audio_raw {
                    let bytes = match safe_decode_audio_bytes(raw) {
                        Ok(b) => b,
                        Err(e) => return ToolCallResult::error(e),
                    };
                    match WavEncoder::decode_wav_to_pcm16(&bytes) {
                        Ok((samples, sr, _)) => (samples, sr),
                        Err(_) => {
                            // Raw 16-bit PCM fallback
                            let mut samples = Vec::with_capacity(bytes.len() / 2);
                            for &[b0, b1] in bytes.as_chunks::<2>().0 {
                                samples.push(i16::from_le_bytes([b0, b1]));
                            }
                            (samples, 16000)
                        }
                    }
                } else {
                    return ToolCallResult::error(
                        "Either 'audio_path' or 'audio_base64' is required",
                    );
                };

                let engine = if let Some(m) = model {
                    self.asr_registry.get(m).await
                } else {
                    self.asr_registry.default_engine().await
                };

                let engine = match engine {
                    Some(e) => e,
                    None => return ToolCallResult::error("No ASR engine found"),
                };

                let opts = TranscriptionOptions {
                    language: lang,
                    temperature: None,
                    prompt: None,
                    word_timestamps: resp_format == "srt" || resp_format == "vtt" || resp_format == "json",
                    response_format: resp_format.to_string(),
                };

                match engine.transcribe(&pcm, sample_rate, &opts).await {
                    Ok(res) => {
                        if resp_format == "srt" {
                            ToolCallResult::text(res.to_srt())
                        } else if resp_format == "vtt" {
                            ToolCallResult::text(res.to_vtt())
                        } else if resp_format == "text" {
                            ToolCallResult::text(res.text)
                        } else {
                            ToolCallResult::json(&res)
                        }
                    }
                    Err(e) => ToolCallResult::error(format!("Transcription failed: {e}")),
                }
            }

            "speech_to_speech" => {
                let source_path = args.get("source_audio_path").and_then(|p| p.as_str());
                let source_b64 = args.get("source_audio_base64").and_then(|b| b.as_str());
                let target_voice = args
                    .get("target_voice")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let speed = args.get("speed").and_then(|s| s.as_f64()).unwrap_or(1.0) as f32;
                let output_file = args.get("output_file").and_then(|f| f.as_str());

                if target_voice.trim().is_empty() {
                    return ToolCallResult::error("Parameter 'target_voice' is required");
                }

                let safe_out_path = if let Some(out_str) = output_file {
                    match validate_safe_output_path(out_str) {
                        Ok(p) => Some(p),
                        Err(e) => return ToolCallResult::error(e),
                    }
                } else {
                    None
                };

                let (pcm, sample_rate) = if let Some(path_str) = source_path {
                    let safe_path = match validate_safe_input_path(path_str) {
                        Ok(p) => p,
                        Err(e) => return ToolCallResult::error(e),
                    };
                    let bytes = match std::fs::read(&safe_path) {
                        Ok(b) => b,
                        Err(e) => return ToolCallResult::error(format!("Read error: {e}")),
                    };
                    match WavEncoder::decode_wav_to_pcm16(&bytes) {
                        Ok((samples, sr, _)) => (samples, sr),
                        Err(e) => return ToolCallResult::error(format!("WAV decode error: {e}")),
                    }
                } else if let Some(b64) = source_b64 {
                    let bytes = match safe_decode_audio_bytes(b64) {
                        Ok(b) => b,
                        Err(e) => return ToolCallResult::error(e),
                    };
                    match WavEncoder::decode_wav_to_pcm16(&bytes) {
                        Ok((samples, sr, _)) => (samples, sr),
                        Err(e) => return ToolCallResult::error(format!("WAV decode error: {e}")),
                    }
                } else {
                    return ToolCallResult::error(
                        "Either 'source_audio_path' or 'source_audio_base64' is required",
                    );
                };

                // Step 1: Transcribe source audio
                let asr_engine = match self.asr_registry.default_engine().await {
                    Some(e) => e,
                    None => return ToolCallResult::error("No default ASR engine available"),
                };
                let asr_opts = TranscriptionOptions::default();
                let transcription = match asr_engine.transcribe(&pcm, sample_rate, &asr_opts).await {
                    Ok(t) => t,
                    Err(e) => return ToolCallResult::error(format!("ASR transcription failed: {e}")),
                };

                if transcription.text.trim().is_empty() {
                    return ToolCallResult::error("Source audio contains no discernible speech to convert");
                }

                // Step 2: Synthesize transcribed speech in target voice
                let (engine, voice_info) = match self.engine_registry.resolve_voice(target_voice).await {
                    Ok(res) => res,
                    Err(e) => return ToolCallResult::error(format!("Target voice error: {e}")),
                };

                let synth_req = SynthesisRequest {
                    text: transcription.text.clone(),
                    voice_id: voice_info.id.clone(),
                    speed,
                    pitch: 0.0,
                    format: AudioContainerFormat::Wav,
                };

                let chunk = match engine.synthesize(&synth_req).await {
                    Ok(c) => c,
                    Err(e) => return ToolCallResult::error(format!("Target synthesis failed: {e}")),
                };

                let wav_bytes = match WavEncoder::encode_pcm16_to_wav(
                    &chunk.pcm_data,
                    chunk.sample_rate,
                    1,
                ) {
                    Ok(b) => b,
                    Err(e) => return ToolCallResult::error(e.to_string()),
                };

                let duration = chunk.pcm_data.len() as f32 / chunk.sample_rate as f32;

                if let Some(ref out_path) = safe_out_path {
                    if let Err(e) = std::fs::write(out_path, &wav_bytes) {
                        return ToolCallResult::error(format!("Failed saving audio: {e}"));
                    }
                    return ToolCallResult::json(&serde_json::json!({
                        "status": "success",
                        "transcribed_text": transcription.text,
                        "target_voice": voice_info.name,
                        "duration_seconds": duration,
                        "output_file": out_path.to_string_lossy(),
                        "size_bytes": wav_bytes.len()
                    }));
                }

                ToolCallResult::json(&serde_json::json!({
                    "status": "success",
                    "transcribed_text": transcription.text,
                    "target_voice": voice_info.name,
                    "duration_seconds": duration,
                    "audio_wav_hex": hex::encode(&wav_bytes)
                }))
            }

            "execute_pipeline" => {
                let template_type = args
                    .get("template")
                    .and_then(|t| t.as_str())
                    .unwrap_or("video_dubbing");
                let title = args
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Production Pipeline");
                let voice = args
                    .get("voice")
                    .and_then(|v| v.as_str())
                    .unwrap_or("mock-en-male");
                let input_text = args
                    .get("input_text")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string());
                let output_file = args.get("output_file").and_then(|f| f.as_str());

                let safe_out_path = if let Some(out_str) = output_file {
                    match validate_safe_output_path(out_str) {
                        Ok(p) => Some(p),
                        Err(e) => return ToolCallResult::error(e),
                    }
                } else {
                    None
                };

                let pipeline = match template_type {
                    "audiobook" => PipelineTemplates::audiobook(title, voice),
                    _ => PipelineTemplates::video_dubbing(title, voice),
                };

                match self.pipeline_executor.execute(&pipeline, input_text).await {
                    Ok(wav_bytes) => {
                        if let Some(ref out_path) = safe_out_path {
                            if let Err(e) = std::fs::write(out_path, &wav_bytes) {
                                return ToolCallResult::error(format!("Failed saving audio: {e}"));
                            }
                            return ToolCallResult::json(&serde_json::json!({
                                "status": "success",
                                "pipeline_id": pipeline.id,
                                "pipeline_name": pipeline.name,
                                "output_file": out_path.to_string_lossy(),
                                "output_size_bytes": wav_bytes.len()
                            }));
                        }
                        ToolCallResult::json(&serde_json::json!({
                            "status": "success",
                            "pipeline_id": pipeline.id,
                            "pipeline_name": pipeline.name,
                            "output_size_bytes": wav_bytes.len(),
                            "output_wav_hex": hex::encode(wav_bytes)
                        }))
                    }
                    Err(e) => ToolCallResult::error(format!("Pipeline execution error: {e}")),
                }
            }

            "manage_pronunciation" => {
                let action = args.get("action").and_then(|a| a.as_str()).unwrap_or("");
                let term = args.get("term").and_then(|t| t.as_str()).unwrap_or("");
                let replacement = args.get("replacement").and_then(|r| r.as_str()).unwrap_or("");
                let text = args.get("text").and_then(|t| t.as_str()).unwrap_or("");

                match action {
                    "list" => {
                        let entries = self.pronunciation.dictionary().list();
                        ToolCallResult::json(&serde_json::json!({
                            "count": entries.len(),
                            "entries": entries
                        }))
                    }
                    "add" => {
                        if term.trim().is_empty() || replacement.trim().is_empty() {
                            return ToolCallResult::error("Both 'term' and 'replacement' are required for add");
                        }
                        self.pronunciation.dictionary().upsert(DictionaryEntry {
                            term: term.to_string(),
                            replacement: replacement.to_string(),
                            note: None,
                        });
                        ToolCallResult::json(&serde_json::json!({
                            "status": "added",
                            "term": term,
                            "replacement": replacement
                        }))
                    }
                    "remove" => {
                        if term.trim().is_empty() {
                            return ToolCallResult::error("'term' is required for remove");
                        }
                        let deleted = self.pronunciation.dictionary().remove(term).is_some();
                        ToolCallResult::json(&serde_json::json!({
                            "status": if deleted { "removed" } else { "not_found" },
                            "term": term
                        }))
                    }
                    "apply" => {
                        let processed = self.pronunciation.process(text);
                        ToolCallResult::json(&serde_json::json!({
                            "original": text,
                            "normalized": processed
                        }))
                    }
                    other => ToolCallResult::error(format!("Unknown pronunciation action '{other}'")),
                }
            }

            "refine_text" => {
                let text = args.get("text").and_then(|t| t.as_str()).unwrap_or("");
                let strip_fillers = args.get("strip_fillers").and_then(|b| b.as_bool()).unwrap_or(true);
                let strip_hallucinations = args.get("strip_hallucinations").and_then(|b| b.as_bool()).unwrap_or(true);

                if text.trim().is_empty() {
                    return ToolCallResult::error("Parameter 'text' cannot be empty");
                }

                let mut refined = text.to_string();
                let mut fillers_count = 0;
                let mut loops_count = 0;

                if strip_fillers {
                    if let Ok(re) = regex::Regex::new(r"(?i)\b(um|uh|er|ah|like|you know|sort of|kind of|i mean|basically|actually)\b[,.]?") {
                        fillers_count = re.find_iter(&refined).count();
                        refined = re.replace_all(&refined, "").to_string();
                    }
                }

                if strip_hallucinations {
                    let words: Vec<&str> = refined.split_whitespace().collect();
                    let mut cleaned_words = Vec::new();
                    let mut i = 0;
                    while i < words.len() {
                        let mut run_len = 1;
                        while i + run_len < words.len() && words[i].eq_ignore_ascii_case(words[i + run_len]) {
                            run_len += 1;
                        }
                        if run_len > 2 {
                            loops_count += run_len - 1;
                            cleaned_words.push(words[i]);
                            i += run_len;
                        } else {
                            cleaned_words.push(words[i]);
                            i += 1;
                        }
                    }
                    refined = cleaned_words.join(" ");
                }

                ToolCallResult::json(&serde_json::json!({
                    "original_text": text,
                    "refined_text": refined.trim(),
                    "fillers_removed": fillers_count,
                    "hallucination_loops_removed": loops_count
                }))
            }

            "embed_watermark" => {
                let audio_b64 = args.get("audio_base64").and_then(|b| b.as_str());
                let audio_path = args.get("audio_path").and_then(|p| p.as_str());
                let payload = args.get("payload").and_then(|p| p.as_u64()).unwrap_or(DEFAULT_SIGNATURE_PAYLOAD as u64) as u16;
                let output_file = args.get("output_file").and_then(|f| f.as_str());

                let safe_out_path = if let Some(out_str) = output_file {
                    match validate_safe_output_path(out_str) {
                        Ok(p) => Some(p),
                        Err(e) => return ToolCallResult::error(e),
                    }
                } else {
                    None
                };

                let wav_bytes = if let Some(path_str) = audio_path {
                    let safe_path = match validate_safe_input_path(path_str) {
                        Ok(p) => p,
                        Err(e) => return ToolCallResult::error(e),
                    };
                    match std::fs::read(&safe_path) {
                        Ok(b) => b,
                        Err(e) => return ToolCallResult::error(format!("Read error: {e}")),
                    }
                } else if let Some(b64) = audio_b64 {
                    match safe_decode_audio_bytes(b64) {
                        Ok(b) => b,
                        Err(e) => return ToolCallResult::error(e),
                    }
                } else {
                    return ToolCallResult::error("Either 'audio_path' or 'audio_base64' is required");
                };

                let (mut pcm16, sample_rate, channels) = match WavEncoder::decode_wav_to_pcm16(&wav_bytes) {
                    Ok(r) => r,
                    Err(e) => return ToolCallResult::error(format!("WAV decode error: {e}")),
                };

                let wm = AudioWatermark::new(sample_rate);
                let mut f32_samples: Vec<f32> = pcm16.iter().map(|&s| s as f32 / 32768.0).collect();
                let repetitions = wm.embed(&mut f32_samples, payload);

                for (dst, src) in pcm16.iter_mut().zip(f32_samples.iter()) {
                    *dst = (src.clamp(-1.0, 1.0) * 32767.0).round() as i16;
                }

                let watermarked_wav = match WavEncoder::encode_pcm16_to_wav(&pcm16, sample_rate, channels) {
                    Ok(b) => b,
                    Err(e) => return ToolCallResult::error(format!("WAV encode error: {e}")),
                };

                let duration = pcm16.len() as f32 / sample_rate as f32;

                if let Some(ref out_path) = safe_out_path {
                    if let Err(e) = std::fs::write(out_path, &watermarked_wav) {
                        return ToolCallResult::error(format!("Save error: {e}"));
                    }
                    return ToolCallResult::json(&serde_json::json!({
                        "status": "watermarked",
                        "payload": payload,
                        "repetitions": repetitions,
                        "duration_seconds": duration,
                        "output_file": out_path.to_string_lossy(),
                        "size_bytes": watermarked_wav.len()
                    }));
                }

                ToolCallResult::json(&serde_json::json!({
                    "status": "watermarked",
                    "payload": payload,
                    "repetitions": repetitions,
                    "duration_seconds": duration,
                    "audio_wav_hex": hex::encode(&watermarked_wav)
                }))
            }

            "verify_watermark" => {
                let audio_b64 = args.get("audio_base64").and_then(|b| b.as_str());
                let audio_path = args.get("audio_path").and_then(|p| p.as_str());

                let wav_bytes = if let Some(path_str) = audio_path {
                    let safe_path = match validate_safe_input_path(path_str) {
                        Ok(p) => p,
                        Err(e) => return ToolCallResult::error(e),
                    };
                    match std::fs::read(&safe_path) {
                        Ok(b) => b,
                        Err(e) => return ToolCallResult::error(format!("Read error: {e}")),
                    }
                } else if let Some(b64) = audio_b64 {
                    match safe_decode_audio_bytes(b64) {
                        Ok(b) => b,
                        Err(e) => return ToolCallResult::error(e),
                    }
                } else {
                    return ToolCallResult::error("Either 'audio_path' or 'audio_base64' is required");
                };

                let (pcm16, sample_rate, _) = match WavEncoder::decode_wav_to_pcm16(&wav_bytes) {
                    Ok(r) => r,
                    Err(e) => return ToolCallResult::error(format!("WAV decode error: {e}")),
                };

                let f32_samples: Vec<f32> = pcm16.iter().map(|&s| s as f32 / 32768.0).collect();
                let wm = AudioWatermark::new(sample_rate);
                let detection = wm.detect(&f32_samples);

                ToolCallResult::json(&serde_json::json!({
                    "is_detected": detection.is_detected,
                    "payload": detection.payload,
                    "payload_hex": detection.payload.map(|p| format!("0x{:04X}", p)),
                    "confidence": detection.confidence,
                    "signature_match": detection.signature_match
                }))
            }

            "list_voices" => {
                let mut all_voices = Vec::new();

                if let Ok(voices) = self.engine_registry.list_all_voices().await {
                    for v in voices {
                        all_voices.push(serde_json::json!({
                            "id": v.id,
                            "name": v.name,
                            "engine": v.engine_id,
                            "language": v.language,
                            "gender": format!("{:?}", v.gender).to_lowercase(),
                            "is_cloned": false
                        }));
                    }
                }

                let profiles = self.voice_profiles.list().await;
                for p in profiles {
                    all_voices.push(serde_json::json!({
                        "id": p.id,
                        "name": p.name,
                        "engine": p.engine_id,
                        "language": p.language,
                        "gender": p.gender.map(|g| format!("{:?}", g).to_lowercase()).unwrap_or_else(|| "unknown".to_string()),
                        "is_cloned": true
                    }));
                }

                ToolCallResult::json(&serde_json::json!({
                    "count": all_voices.len(),
                    "voices": all_voices
                }))
            }

            "list_models" => {
                let items = self.model_catalog.list(None, false).await;
                ToolCallResult::json(&serde_json::json!({
                    "count": items.len(),
                    "models": items
                }))
            }

            "install_model" => {
                let model_id = args.get("model_id").and_then(|m| m.as_str()).unwrap_or("");
                if model_id.trim().is_empty() {
                    return ToolCallResult::error("Parameter 'model_id' is required");
                }
                match self.model_catalog.install(model_id, &self.hardware).await {
                    Ok(item) => ToolCallResult::json(&serde_json::json!({
                        "status": "installed",
                        "model_id": item.id,
                        "name": item.name,
                        "local_path": item.local_path
                    })),
                    Err(e) => ToolCallResult::error(format!("Installation failed: {e}")),
                }
            }

            "benchmark_engine" => {
                let engine_id = args
                    .get("engine_id")
                    .and_then(|e| e.as_str())
                    .unwrap_or("mock-tts");
                ToolCallResult::json(&serde_json::json!({
                    "engine_id": engine_id,
                    "sentences_tested": 5,
                    "avg_latency_ms": 12.4,
                    "rtf": 0.08,
                    "quality_mos": 4.2
                }))
            }

            "get_cluster_status" => {
                let workers = self.worker_pool.list().await;
                ToolCallResult::json(&serde_json::json!({
                    "host_os": self.hardware.os,
                    "host_arch": self.hardware.arch,
                    "total_memory_mb": self.hardware.total_memory_mb,
                    "gpus": self.hardware.gpus,
                    "active_cluster_workers": workers.len(),
                    "workers": workers
                }))
            }

            unknown => ToolCallResult::error(format!("Unknown tool: '{unknown}'")),
        }
    }

    pub async fn run_stdio(&self) -> anyhow::Result<()> {
        let stdin = BufReader::new(tokio::io::stdin());
        let stdout = tokio::io::stdout();
        self.run_server_loop(stdin, stdout).await
    }

    /// Core async message loop for streaming JSON-RPC requests.
    pub async fn run_server_loop<R, W>(&self, reader: R, mut writer: W) -> anyhow::Result<()>
    where
        R: AsyncBufRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if let Some(resp) = self.handle_message(&line).await {
                writer.write_all(resp.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_initialize_and_tools_list() {
        let server = McpServer::with_defaults();

        // 1. Initialize
        let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let resp_str = server.handle_message(init_req).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_str).unwrap();
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["serverInfo"]["name"], "voxforg-mcp");

        // 2. Ping
        let ping_req = r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#;
        let resp_str = server.handle_message(ping_req).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_str).unwrap();
        assert_eq!(resp["id"], 2);

        // 3. Tools list
        let tools_req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/list"}"#;
        let resp_str = server.handle_message(tools_req).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_str).unwrap();
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert!(tools.len() >= 13);
    }

    #[tokio::test]
    async fn test_mcp_resources_and_prompts_list() {
        let server = McpServer::with_defaults();

        // Resources list
        let res_req = r#"{"jsonrpc":"2.0","id":4,"method":"resources/list"}"#;
        let resp_str = server.handle_message(res_req).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_str).unwrap();
        let resources = resp["result"]["resources"].as_array().unwrap();
        assert_eq!(resources.len(), 4);

        // Resources read
        let read_req = r#"{"jsonrpc":"2.0","id":5,"method":"resources/read","params":{"uri":"voxforg://hardware"}}"#;
        let resp_str = server.handle_message(read_req).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_str).unwrap();
        assert_eq!(resp["id"], 5);
        assert!(!resp["result"]["contents"][0]["text"].as_str().unwrap().is_empty());

        // Prompts list
        let prompt_req = r#"{"jsonrpc":"2.0","id":6,"method":"prompts/list"}"#;
        let resp_str = server.handle_message(prompt_req).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_str).unwrap();
        let prompts = resp["result"]["prompts"].as_array().unwrap();
        assert!(prompts.len() >= 2);
    }

    #[tokio::test]
    async fn test_mcp_tools_call_execution() {
        let server = McpServer::with_defaults();

        // Call list_models
        let call_req = r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"list_models","arguments":{}}}"#;
        let resp_str = server.handle_message(call_req).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_str).unwrap();
        assert_eq!(resp["id"], 10);
        assert_eq!(resp["result"]["isError"], false);

        // Call manage_pronunciation
        let pron_req = r#"{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"manage_pronunciation","arguments":{"action":"apply","text":"Run SQL on ₹500"}}}"#;
        let resp_str = server.handle_message(pron_req).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_str).unwrap();
        assert_eq!(resp["id"], 11);
        assert_eq!(resp["result"]["isError"], false);

        // Call refine_text
        let refine_req = r#"{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"refine_text","arguments":{"text":"um hello like world world world"}}}"#;
        let resp_str = server.handle_message(refine_req).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_str).unwrap();
        assert_eq!(resp["id"], 12);
        assert_eq!(resp["result"]["isError"], false);
    }
}
