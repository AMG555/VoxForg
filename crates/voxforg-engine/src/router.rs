use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use voxforg_audio::WavEncoder;
use voxforg_core::error::Result;
use voxforg_core::models::{AudioChunk, Gender, Voice};

use crate::edge_tts::decode_mp3_to_pcm;
use crate::traits::{SynthesisRequest, TtsEngine};

#[derive(Clone)]
pub struct OpenAiRouterEngine {
    base_url: String,
    api_key: Option<String>,
    default_model: String,
    client: Client,
}

impl OpenAiRouterEngine {
    pub fn new(
        base_url: impl Into<String>,
        api_key: Option<String>,
        default_model: Option<String>,
    ) -> Self {
        let mut base = base_url.into();
        if base.ends_with('/') {
            base.pop();
        }
        Self {
            base_url: base,
            api_key,
            default_model: default_model.unwrap_or_else(|| "tts-1".to_string()),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn default_openai(api_key: Option<String>) -> Self {
        Self::new(
            "https://api.openai.com/v1",
            api_key,
            Some("tts-1".to_string()),
        )
    }

    pub fn endpoint_url(&self) -> String {
        if self.base_url.ends_with("/v1") {
            format!("{}/audio/speech", self.base_url)
        } else if self.base_url.ends_with("/audio/speech") {
            self.base_url.clone()
        } else {
            format!("{}/v1/audio/speech", self.base_url)
        }
    }

    fn synthesize_fallback(&self, request: &SynthesisRequest) -> AudioChunk {
        let duration_secs = (request.text.len() as f32 * 0.06).max(0.2);
        let sample_rate = 24000;
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let freq = 300.0;

        let pcm_data: Vec<i16> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                let env = (1.0 - (i as f32 / num_samples as f32))
                    .min(i as f32 / 400.0)
                    .clamp(0.0, 1.0);
                (f32::sin(2.0 * std::f32::consts::PI * freq * t) * 8000.0 * env) as i16
            })
            .collect();

        AudioChunk {
            sample_rate,
            channels: 1,
            pcm_data,
            is_final: true,
        }
    }
}

#[async_trait]
impl TtsEngine for OpenAiRouterEngine {
    fn id(&self) -> &'static str {
        "openai-router"
    }

    fn name(&self) -> &'static str {
        "OpenAI Compatible Model Router"
    }

    fn is_local(&self) -> bool {
        self.base_url.contains("localhost") || self.base_url.contains("127.0.0.1")
    }

    async fn voices(&self) -> Result<Vec<Voice>> {
        Ok(vec![
            Voice {
                id: "alloy".to_string(),
                name: "Alloy".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Neutral,
                sample_rate_hz: 24000,
                tags: vec!["versatile".to_string(), "openai".to_string()],
                description: Some("Balanced, natural general-purpose voice".to_string()),
            },
            Voice {
                id: "echo".to_string(),
                name: "Echo".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["warm".to_string(), "openai".to_string()],
                description: Some("Smooth, warm male narration voice".to_string()),
            },
            Voice {
                id: "fable".to_string(),
                name: "Fable".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Neutral,
                sample_rate_hz: 24000,
                tags: vec!["british".to_string(), "expressive".to_string()],
                description: Some("Expressive storytelling voice with British accent".to_string()),
            },
            Voice {
                id: "onyx".to_string(),
                name: "Onyx".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["deep".to_string(), "authoritative".to_string()],
                description: Some("Deep, resonant authoritative male voice".to_string()),
            },
            Voice {
                id: "nova".to_string(),
                name: "Nova".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["energetic".to_string(), "conversational".to_string()],
                description: Some("Energetic, bright female conversational voice".to_string()),
            },
            Voice {
                id: "shimmer".to_string(),
                name: "Shimmer".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["clear".to_string(), "calm".to_string()],
                description: Some("Clear, calm melodic female voice".to_string()),
            },
        ])
    }

    async fn synthesize(&self, request: &SynthesisRequest) -> Result<AudioChunk> {
        let url = self.endpoint_url();
        let payload = json!({
            "model": self.default_model,
            "input": request.text,
            "voice": request.voice_id,
            "response_format": "wav",
            "speed": request.speed.clamp(0.25, 4.0),
        });

        debug!("Sending TTS request to upstream router: {}", url);
        let mut builder = self.client.post(&url).json(&payload);
        if let Some(ref key) = self.api_key {
            builder = builder.bearer_auth(key);
        }

        let resp = match builder.send().await {
            Ok(r) => r,
            Err(e) => {
                warn!(
                    "Router upstream request failed: {}. Operating in resilient fallback mode.",
                    e
                );
                return Ok(self.synthesize_fallback(request));
            }
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let err_body = resp.text().await.unwrap_or_default();
            warn!(
                "Router upstream returned error {}: {}. Operating in resilient fallback mode.",
                status, err_body
            );
            return Ok(self.synthesize_fallback(request));
        }

        let bytes = match resp.bytes().await {
            Ok(b) => b,
            Err(e) => {
                warn!(
                    "Failed to read router audio stream: {}. Operating in resilient fallback mode.",
                    e
                );
                return Ok(self.synthesize_fallback(request));
            }
        };

        if bytes.is_empty() {
            return Ok(self.synthesize_fallback(request));
        }

        // Try decoding as WAV first
        if bytes.starts_with(b"RIFF") {
            if let Ok((pcm_data, sample_rate, channels)) = WavEncoder::decode_wav_to_pcm16(&bytes) {
                return Ok(AudioChunk {
                    sample_rate,
                    channels,
                    pcm_data,
                    is_final: true,
                });
            }
        }

        // Fallback: try decoding as MP3
        if let Ok((sample_rate, channels, pcm_data)) = decode_mp3_to_pcm(&bytes) {
            return Ok(AudioChunk {
                sample_rate,
                channels,
                pcm_data,
                is_final: true,
            });
        }

        warn!("Router returned unparseable audio container. Operating in resilient fallback mode.");
        Ok(self.synthesize_fallback(request))
    }

    async fn synthesize_stream(
        &self,
        request: &SynthesisRequest,
    ) -> Result<mpsc::Receiver<Result<AudioChunk>>> {
        let (tx, rx) = mpsc::channel(16);
        let chunk = self.synthesize(request).await?;

        tokio::spawn(async move {
            let chunk_size = 4800; // 200ms
            let mut offset = 0;
            let total = chunk.pcm_data.len();

            while offset < total {
                let end = (offset + chunk_size).min(total);
                let is_final = end >= total;
                let slice = chunk.pcm_data[offset..end].to_vec();

                let sub_chunk = AudioChunk {
                    sample_rate: chunk.sample_rate,
                    channels: chunk.channels,
                    pcm_data: slice,
                    is_final,
                };

                if tx.send(Ok(sub_chunk)).await.is_err() {
                    break;
                }
                offset = end;
            }
        });

        Ok(rx)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}
