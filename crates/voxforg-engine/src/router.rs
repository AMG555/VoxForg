use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use voxforg_audio::WavEncoder;
use voxforg_core::error::Result;
use voxforg_core::models::{AudioChunk, Gender, Voice};

use crate::edge_tts::decode_mp3_to_pcm;
use crate::traits::{EngineCapabilities, SynthesisRequest, TtsEngine};

pub fn mask_secret(secret: &str) -> String {
    if secret.len() <= 8 {
        "***".to_string()
    } else {
        format!("{}...{}", &secret[..4], &secret[secret.len() - 4..])
    }
}

pub fn validate_router_url(
    url_str: &str,
    allow_private_ips: bool,
) -> std::result::Result<(), String> {
    let parsed = reqwest::Url::parse(url_str).map_err(|e| format!("Invalid URL: {e}"))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(format!(
            "Invalid URL scheme '{scheme}', only http and https allowed"
        ));
    }

    if let Some(host) = parsed.host_str() {
        let host_lower = host.to_lowercase();
        // Disallow cloud metadata endpoints unconditionally
        if host_lower == "169.254.169.254"
            || host_lower == "metadata.google.internal"
            || host_lower.contains("169.254.")
        {
            return Err(
                "Access to cloud metadata endpoints (169.254.x.x) is strictly forbidden"
                    .to_string(),
            );
        }

        // IP address checks
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            match ip {
                std::net::IpAddr::V4(ipv4) => {
                    if ipv4.is_link_local() {
                        return Err(
                            "Link-local addresses (169.254.0.0/16) are forbidden".to_string()
                        );
                    }
                    if !allow_private_ips && (ipv4.is_loopback() || ipv4.is_private()) {
                        return Err(format!(
                            "Private IP address '{ipv4}' is blocked by SSRF policy. Use --allow-private-ips to permit."
                        ));
                    }
                }
                std::net::IpAddr::V6(ipv6) => {
                    if !allow_private_ips && ipv6.is_loopback() {
                        return Err("Loopback IPv6 is blocked by SSRF policy. Use --allow-private-ips to permit.".to_string());
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Clone)]
pub struct OpenAiRouterEngine {
    base_url: String,
    api_key: Option<String>,
    default_model: String,
    allow_private_ips: bool,
    client: Client,
    consecutive_failures: Arc<AtomicU32>,
    circuit_open_until_ms: Arc<AtomicU64>,
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

        // Default allow_private_ips to true if local development URL is specified
        let is_local_dev =
            base.contains("localhost") || base.contains("127.0.0.1") || base.contains("[::1]");

        let client = Client::builder()
            .tcp_keepalive(Some(Duration::from_secs(15)))
            .pool_idle_timeout(Some(Duration::from_secs(90)))
            .pool_max_idle_per_host(32)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(25))
            .build()
            .unwrap_or_default();

        Self {
            base_url: base,
            api_key,
            default_model: default_model.unwrap_or_else(|| "tts-1".to_string()),
            allow_private_ips: is_local_dev,
            client,
            consecutive_failures: Arc::new(AtomicU32::new(0)),
            circuit_open_until_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn with_allow_private_ips(mut self, allow: bool) -> Self {
        self.allow_private_ips = allow;
        self
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

    fn current_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    fn is_circuit_open(&self) -> bool {
        let open_until = self.circuit_open_until_ms.load(Ordering::Relaxed);
        if open_until == 0 {
            return false;
        }
        let now = Self::current_timestamp_ms();
        if now < open_until {
            true
        } else {
            self.circuit_open_until_ms.store(0, Ordering::Relaxed);
            false
        }
    }

    fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.circuit_open_until_ms.store(0, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        let fails = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        if fails >= 5 {
            let cooldown = Self::current_timestamp_ms() + 30_000; // 30s circuit break
            self.circuit_open_until_ms
                .store(cooldown, Ordering::Relaxed);
            warn!(
                "Upstream router reached {} consecutive failures. Circuit breaker opened for 30s.",
                fails
            );
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

    fn capabilities(&self) -> EngineCapabilities {
        let local = self.is_local();
        EngineCapabilities {
            // Cloud OpenAI pricing as baseline; 0.0 for local inference
            cost_per_1k_chars: if local { 0.0 } else { 0.015 },
            avg_latency_ms: if local { 150 } else { 600 },
            quality_score: 0.90,
            languages: vec![
                "en-US".to_string(),
                "en-GB".to_string(),
                "de-DE".to_string(),
                "fr-FR".to_string(),
                "es-ES".to_string(),
                "zh-CN".to_string(),
                "ja-JP".to_string(),
                "hi-IN".to_string(),
            ],
            is_local: local,
        }
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
                tags: vec!["conversational".to_string(), "openai".to_string()],
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

        // 1. SSRF URL validation
        if let Err(err) = validate_router_url(&url, self.allow_private_ips) {
            warn!(
                "Router request blocked by SSRF policy: {}. Operating in fallback mode.",
                err
            );
            return Ok(self.synthesize_fallback(request));
        }

        // 2. Circuit Breaker Check
        if self.is_circuit_open() {
            debug!("Router circuit breaker is active. Fast-failing to fallback synthesis.");
            return Ok(self.synthesize_fallback(request));
        }

        let payload = json!({
            "model": self.default_model,
            "input": request.text,
            "voice": request.voice_id,
            "response_format": "wav",
            "speed": request.speed.clamp(0.25, 4.0),
        });

        if let Some(ref key) = self.api_key {
            debug!(
                "Sending TTS request to upstream router {} (auth: {})",
                url,
                mask_secret(key)
            );
        } else {
            debug!("Sending TTS request to upstream router {}", url);
        }

        // 3. Retry loop with exponential backoff and jitter
        let mut attempts = 0;
        let max_retries = 2;

        loop {
            attempts += 1;
            let mut builder = self.client.post(&url).json(&payload);
            if let Some(ref key) = self.api_key {
                builder = builder.bearer_auth(key);
            }

            match builder.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        let bytes = match resp.bytes().await {
                            Ok(b) => b,
                            Err(e) => {
                                self.record_failure();
                                warn!("Failed to read router audio stream: {e}. Operating in fallback mode.");
                                return Ok(self.synthesize_fallback(request));
                            }
                        };

                        if bytes.is_empty() {
                            self.record_failure();
                            return Ok(self.synthesize_fallback(request));
                        }

                        // Try decoding as WAV first
                        if bytes.starts_with(b"RIFF") {
                            if let Ok((pcm_data, sample_rate, channels)) =
                                WavEncoder::decode_wav_to_pcm16(&bytes)
                            {
                                self.record_success();
                                return Ok(AudioChunk {
                                    sample_rate,
                                    channels,
                                    pcm_data,
                                    is_final: true,
                                });
                            }
                        }

                        // Try decoding as MP3
                        if let Ok((sample_rate, channels, pcm_data)) = decode_mp3_to_pcm(&bytes) {
                            self.record_success();
                            return Ok(AudioChunk {
                                sample_rate,
                                channels,
                                pcm_data,
                                is_final: true,
                            });
                        }

                        self.record_failure();
                        warn!("Router returned unparseable audio container. Operating in fallback mode.");
                        return Ok(self.synthesize_fallback(request));
                    }

                    // Transient 5xx server errors warrant retry
                    if (status.as_u16() == 502 || status.as_u16() == 503 || status.as_u16() == 504)
                        && attempts <= max_retries
                    {
                        let backoff_ms = (attempts as u64) * 80 + 20;
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        continue;
                    }

                    let err_body = resp.text().await.unwrap_or_default();
                    self.record_failure();
                    warn!(
                        "Router upstream returned error {}: {}. Operating in resilient fallback mode.",
                        status, err_body
                    );
                    return Ok(self.synthesize_fallback(request));
                }
                Err(e) => {
                    if attempts <= max_retries {
                        let backoff_ms = (attempts as u64) * 80 + 20;
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        continue;
                    }

                    self.record_failure();
                    warn!(
                        "Router upstream request failed after {} attempts: {}. Operating in fallback mode.",
                        attempts, e
                    );
                    return Ok(self.synthesize_fallback(request));
                }
            }
        }
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
        Ok(!self.is_circuit_open())
    }
}

impl Default for OpenAiRouterEngine {
    fn default() -> Self {
        Self::new("https://api.openai.com/v1", None, None)
    }
}
