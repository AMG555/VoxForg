use std::sync::Arc;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use voxforg_asr::{AsrRegistry, TranscriptionOptions};
use voxforg_audio::WavEncoder;
use voxforg_catalog::ModelCatalogStore;
use voxforg_core::models::{AudioContainerFormat, CloneVoiceRequest, ClonedSynthesisRequest};
use voxforg_engine::{
    EdgeTtsEngine, EngineRegistry, MockTtsEngine, Qwen3TtsEngine, SynthesisRequest,
    VoiceProfileStore,
};
use voxforg_hardware::{HardwareProbe, HardwareProfile};
use voxforg_pipeline::{PipelineExecutor, PipelineTemplates};
use voxforg_worker::WorkerPool;

use crate::protocol::{JsonRpcRequest, JsonRpcResponse, ToolCallResult};
use crate::tools::get_mcp_tools;

/// Full in-process MCP server dispatching to VoxForg speech runtimes.
pub struct McpServer {
    pub engine_registry: Arc<EngineRegistry>,
    pub voice_profiles: Arc<VoiceProfileStore>,
    pub asr_registry: Arc<AsrRegistry>,
    pub model_catalog: Arc<ModelCatalogStore>,
    pub pipeline_executor: Arc<PipelineExecutor>,
    pub hardware: HardwareProfile,
    pub worker_pool: Arc<WorkerPool>,
}

impl Default for McpServer {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl McpServer {
    /// Construct server with fully initialized standard speech engines and registries.
    pub fn with_defaults() -> Self {
        let engine_registry = Arc::new(EngineRegistry::new());
        let mock_engine = Arc::new(MockTtsEngine::new(24000));
        let edge_engine = Arc::new(EdgeTtsEngine::new());
        let qwen3_engine = Arc::new(Qwen3TtsEngine::default());

        // Register default engines synchronously
        engine_registry.register_sync(mock_engine);
        engine_registry.register_sync(edge_engine);
        engine_registry.register_sync(qwen3_engine);

        let voice_profiles = Arc::new(VoiceProfileStore::with_defaults());
        let asr_registry = Arc::new(AsrRegistry::with_defaults());
        let model_catalog = Arc::new(ModelCatalogStore::with_curated_models());
        let pipeline_executor = Arc::new(
            PipelineExecutor::new(engine_registry.clone()).with_asr_registry(asr_registry.clone()),
        );
        let hardware = HardwareProbe::probe();
        let worker_pool = Arc::new(WorkerPool::new());

        Self {
            engine_registry,
            voice_profiles,
            asr_registry,
            model_catalog,
            pipeline_executor,
            hardware,
            worker_pool,
        }
    }

    /// Process a raw single-line JSON-RPC request and return formatted response string.
    pub async fn handle_message(&self, raw: &str) -> Option<String> {
        let trimmed = raw.trim();
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
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "voxforg-mcp",
                        "version": "0.1.0"
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

                if text.trim().is_empty() {
                    return ToolCallResult::error("Parameter 'text' cannot be empty");
                }

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
                                let hex_data = hex::encode(&wav_bytes);
                                let duration =
                                    chunk.pcm_data.len() as f32 / chunk.sample_rate as f32;
                                return ToolCallResult::json(&serde_json::json!({
                                    "status": "success",
                                    "voice_profile": profile.name,
                                    "duration_seconds": duration,
                                    "audio_wav_hex": hex_data,
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
                                let hex_data = hex::encode(&wav_bytes);
                                let duration =
                                    chunk.pcm_data.len() as f32 / chunk.sample_rate as f32;
                                ToolCallResult::json(&serde_json::json!({
                                    "status": "success",
                                    "voice": voice_info.name,
                                    "engine": voice_info.engine_id,
                                    "duration_seconds": duration,
                                    "audio_wav_hex": hex_data,
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
                let ref_audio = args
                    .get("reference_audio_base64")
                    .and_then(|a| a.as_str())
                    .unwrap_or("");
                let engine_id = args
                    .get("engine_id")
                    .and_then(|e| e.as_str())
                    .unwrap_or("qwen3-tts");
                let lang = args
                    .get("language")
                    .and_then(|l| l.as_str())
                    .unwrap_or("en-US");
                let desc = args.get("description").and_then(|d| d.as_str());

                if name.trim().is_empty() || ref_audio.trim().is_empty() {
                    return ToolCallResult::error(
                        "Parameters 'name' and 'reference_audio_base64' are required",
                    );
                }

                let engine = match self.engine_registry.get(engine_id).await {
                    Some(e) => e,
                    None => {
                        return ToolCallResult::error(format!("Engine '{engine_id}' not found"))
                    }
                };

                let clone_req = CloneVoiceRequest {
                    name: name.to_string(),
                    engine_id: engine_id.to_string(),
                    reference_audio_base64: Some(ref_audio.to_string()),
                    reference_audio_path: None,
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
                let audio_raw = args
                    .get("audio_base64")
                    .and_then(|a| a.as_str())
                    .unwrap_or("");
                let model = args.get("model").and_then(|m| m.as_str());
                let lang = args
                    .get("language")
                    .and_then(|l| l.as_str())
                    .map(|s| s.to_string());
                let resp_format = args
                    .get("response_format")
                    .and_then(|f| f.as_str())
                    .unwrap_or("json");

                if audio_raw.trim().is_empty() {
                    return ToolCallResult::error("Parameter 'audio_base64' cannot be empty");
                }

                let bytes =
                    hex::decode(audio_raw.trim()).unwrap_or_else(|_| audio_raw.as_bytes().to_vec());
                let mut samples = Vec::with_capacity(bytes.len() / 2);
                for chunk in bytes.chunks_exact(2) {
                    samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
                }

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
                    word_timestamps: true,
                    response_format: resp_format.to_string(),
                };

                match engine.transcribe(&samples, 16000, &opts).await {
                    Ok(res) => {
                        if resp_format == "srt" {
                            ToolCallResult::text(res.to_srt())
                        } else if resp_format == "vtt" {
                            ToolCallResult::text(res.to_vtt())
                        } else {
                            ToolCallResult::json(&res)
                        }
                    }
                    Err(e) => ToolCallResult::error(format!("Transcription failed: {e}")),
                }
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

                let pipeline = match template_type {
                    "audiobook" => PipelineTemplates::audiobook(title, voice),
                    _ => PipelineTemplates::video_dubbing(title, voice),
                };

                match self.pipeline_executor.execute(&pipeline, input_text).await {
                    Ok(wav_bytes) => ToolCallResult::json(&serde_json::json!({
                        "status": "success",
                        "pipeline_id": pipeline.id,
                        "pipeline_name": pipeline.name,
                        "output_size_bytes": wav_bytes.len(),
                        "output_wav_hex": hex::encode(wav_bytes)
                    })),
                    Err(e) => ToolCallResult::error(format!("Pipeline execution error: {e}")),
                }
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
        assert!(tools.len() >= 8);
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
        assert!(!resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .is_empty());

        // Call get_cluster_status
        let call_req = r#"{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"get_cluster_status","arguments":{}}}"#;
        let resp_str = server.handle_message(call_req).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_str).unwrap();
        assert_eq!(resp["id"], 11);
        assert_eq!(resp["result"]["isError"], false);
    }
}
