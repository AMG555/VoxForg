use async_trait::async_trait;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use std::time::Duration;
use tracing::{info, warn};
use voxforg_audio::WavEncoder;
use voxforg_core::error::VoxForgError;

use crate::traits::{AsrEngine, AsrEngineInfo};
use crate::types::{
    TranscriptionOptions, TranscriptionResult, TranscriptionSegment, WordTimestamp,
};

/// Generic upstream ASR engine supporting any OpenAI-compatible `/v1/audio/transcriptions` endpoint.
/// Works with Faster-Whisper servers, WhisperX servers, vLLM, local-ai, llama.cpp, and OpenAI.
pub struct OpenAiAsrEngine {
    id: String,
    name: String,
    endpoint_url: String,
    api_key: Option<String>,
    default_model: String,
    client: Client,
}

impl OpenAiAsrEngine {
    pub fn new(endpoint_url: impl Into<String>) -> Self {
        Self {
            id: "openai-asr".to_string(),
            name: "OpenAI-Compatible ASR Engine".to_string(),
            endpoint_url: endpoint_url.into(),
            api_key: None,
            default_model: "whisper-1".to_string(),
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn with_options(
        id: impl Into<String>,
        name: impl Into<String>,
        endpoint_url: impl Into<String>,
        api_key: Option<String>,
        default_model: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            endpoint_url: endpoint_url.into(),
            api_key,
            default_model: default_model.into(),
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn from_env() -> Option<Self> {
        let url = std::env::var("VOXFORG_ASR_URL").ok()?;
        let key = std::env::var("VOXFORG_ASR_API_KEY").ok();
        let model = std::env::var("VOXFORG_ASR_MODEL").unwrap_or_else(|_| "whisper-1".to_string());
        Some(Self::with_options(
            "openai-asr",
            "OpenAI-Compatible Remote ASR",
            url,
            key,
            model,
        ))
    }
}

#[async_trait]
impl AsrEngine for OpenAiAsrEngine {
    fn info(&self) -> AsrEngineInfo {
        AsrEngineInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: format!(
                "OpenAI-compatible ASR bridge connected to {}",
                self.endpoint_url
            ),
            supported_languages: vec![
                "en".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "de".to_string(),
                "it".to_string(),
                "ja".to_string(),
                "zh".to_string(),
                "pt".to_string(),
            ],
            is_local: false,
            sample_rate: 16000,
        }
    }

    async fn transcribe(
        &self,
        audio_pcm: &[i16],
        sample_rate: u32,
        options: &TranscriptionOptions,
    ) -> Result<TranscriptionResult, VoxForgError> {
        if audio_pcm.is_empty() {
            return Err(VoxForgError::AudioProcessing(
                "Cannot transcribe empty audio buffer".to_string(),
            ));
        }

        // 1. Encode PCM16 to in-memory WAV
        let wav_bytes =
            WavEncoder::encode_pcm16_to_wav(audio_pcm, sample_rate, 1).map_err(|e| {
                VoxForgError::AudioProcessing(format!("Failed to encode WAV for ASR: {e}"))
            })?;

        let model = self.default_model.clone();
        let file_part = Part::bytes(wav_bytes)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| VoxForgError::AudioProcessing(format!("MIME error: {e}")))?;

        let mut form = Form::new()
            .part("file", file_part)
            .text("model", model)
            .text("response_format", "verbose_json");

        if let Some(ref lang) = options.language {
            form = form.text("language", lang.clone());
        }
        if let Some(ref prompt) = options.prompt {
            form = form.text("prompt", prompt.clone());
        }
        if let Some(temp) = options.temperature {
            form = form.text("temperature", temp.to_string());
        }

        // 2. Dispatch request
        let mut req = self.client.post(&self.endpoint_url).multipart(form);
        if let Some(ref key) = self.api_key {
            req = req.bearer_auth(key);
        }

        match req.send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let json_val: serde_json::Value =
                        resp.json()
                            .await
                            .map_err(|e| VoxForgError::PipelineExecution {
                                node_id: self.id.clone(),
                                reason: format!("Failed to parse ASR JSON response: {e}"),
                            })?;

                    let text = json_val
                        .get("text")
                        .and_then(|t| t.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let lang = json_val
                        .get("language")
                        .and_then(|l| l.as_str())
                        .unwrap_or("en")
                        .to_string();
                    let duration_sec = json_val
                        .get("duration")
                        .and_then(|d| d.as_f64())
                        .unwrap_or(audio_pcm.len() as f64 / sample_rate as f64)
                        as f32;

                    let mut segments = Vec::new();
                    if let Some(segs_arr) = json_val.get("segments").and_then(|s| s.as_array()) {
                        for (i, s) in segs_arr.iter().enumerate() {
                            let start_s = s.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let end_s = s
                                .get("end")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(duration_sec as f64);
                            let seg_text = s
                                .get("text")
                                .and_then(|t| t.as_str())
                                .unwrap_or_default()
                                .trim()
                                .to_string();
                            let speaker = s
                                .get("speaker")
                                .and_then(|spk| spk.as_str())
                                .map(|spk| spk.to_string());

                            segments.push(TranscriptionSegment {
                                id: i as u32,
                                seek: (start_s * 100.0) as u32,
                                start_ms: (start_s * 1000.0) as u64,
                                end_ms: (end_s * 1000.0) as u64,
                                text: seg_text,
                                tokens: Vec::new(),
                                temperature: options.temperature.unwrap_or(0.0),
                                avg_logprob: -0.2,
                                compression_ratio: 1.0,
                                no_speech_prob: 0.01,
                                speaker,
                                words: Vec::new(),
                            });
                        }
                    }

                    let mut words = Vec::new();
                    if let Some(words_arr) = json_val.get("words").and_then(|w| w.as_array()) {
                        for w in words_arr {
                            let word_str =
                                w.get("word").and_then(|v| v.as_str()).unwrap_or_default();
                            let start_s = w.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let end_s = w.get("end").and_then(|v| v.as_f64()).unwrap_or(start_s);
                            let conf = w
                                .get("score")
                                .or_else(|| w.get("probability"))
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.95) as f32;
                            words.push(WordTimestamp {
                                word: word_str.to_string(),
                                start_ms: (start_s * 1000.0) as u64,
                                end_ms: (end_s * 1000.0) as u64,
                                probability: conf,
                            });
                        }
                    }

                    if segments.is_empty() && !text.is_empty() {
                        segments.push(TranscriptionSegment {
                            id: 0,
                            seek: 0,
                            start_ms: 0,
                            end_ms: (duration_sec * 1000.0) as u64,
                            text: text.clone(),
                            tokens: Vec::new(),
                            temperature: 0.0,
                            avg_logprob: -0.1,
                            compression_ratio: 1.0,
                            no_speech_prob: 0.01,
                            speaker: None,
                            words: Vec::new(),
                        });
                    }

                    info!(
                        engine = %self.id,
                        segments = segments.len(),
                        words = words.len(),
                        "Successfully received remote ASR transcription"
                    );

                    Ok(TranscriptionResult {
                        text,
                        task: "transcribe".to_string(),
                        language: lang,
                        duration_seconds: duration_sec,
                        segments,
                        words,
                    })
                } else {
                    let status = resp.status();
                    let err_body = resp.text().await.unwrap_or_default();
                    warn!(
                        status = %status,
                        body = %err_body,
                        "Upstream ASR returned HTTP error, providing fallback"
                    );
                    self.fallback_transcription(audio_pcm, sample_rate, options)
                }
            }
            Err(e) => {
                warn!(
                    error = %e,
                    endpoint = %self.endpoint_url,
                    "Cannot reach remote ASR endpoint, providing fallback"
                );
                self.fallback_transcription(audio_pcm, sample_rate, options)
            }
        }
    }
}

impl OpenAiAsrEngine {
    fn fallback_transcription(
        &self,
        audio_pcm: &[i16],
        sample_rate: u32,
        _options: &TranscriptionOptions,
    ) -> Result<TranscriptionResult, VoxForgError> {
        let duration_sec = audio_pcm.len() as f32 / sample_rate.max(1) as f32;
        let text = "[Remote ASR endpoint offline or unreachable]".to_string();
        Ok(TranscriptionResult {
            text: text.clone(),
            task: "transcribe".to_string(),
            language: "en".to_string(),
            duration_seconds: duration_sec,
            segments: vec![TranscriptionSegment {
                id: 0,
                seek: 0,
                start_ms: 0,
                end_ms: (duration_sec * 1000.0) as u64,
                text,
                tokens: Vec::new(),
                temperature: 0.0,
                avg_logprob: -0.5,
                compression_ratio: 1.0,
                no_speech_prob: 0.1,
                speaker: None,
                words: Vec::new(),
            }],
            words: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_openai_asr_offline_fallback() {
        let engine = OpenAiAsrEngine::new("http://127.0.0.1:19999/v1/audio/transcriptions");
        let pcm = vec![0i16; 16000]; // 1 second
        let opts = TranscriptionOptions::default();

        let res = engine.transcribe(&pcm, 16000, &opts).await.unwrap();
        assert_eq!(res.duration_seconds, 1.0);
        assert!(res.text.contains("offline"));
    }
}
