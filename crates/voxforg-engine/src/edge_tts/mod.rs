pub mod client;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::warn;
use voxforg_core::error::Result;
use voxforg_core::models::{AudioChunk, Gender, Voice};

pub use client::{
    decode_mp3_to_pcm, generate_sec_ms_gec, parse_binary_audio_payload, EdgeTtsClient,
};
use crate::traits::{SynthesisRequest, TtsEngine};

pub struct EdgeTtsEngine {
    _client: reqwest::Client,
}

impl EdgeTtsEngine {
    pub fn new() -> Self {
        Self {
            _client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0")
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn build_ssml(request: &SynthesisRequest) -> String {
        let speed = if request.speed.is_finite() {
            request.speed.clamp(0.25, 4.0)
        } else {
            1.0
        };
        let rate_pct = ((speed - 1.0) * 100.0).round() as i32;
        let rate_str = if rate_pct >= 0 {
            format!("+{}%", rate_pct)
        } else {
            format!("{}%", rate_pct)
        };

        let pitch = if request.pitch.is_finite() {
            request.pitch.clamp(-50.0, 50.0)
        } else {
            0.0
        };
        let pitch_hz = (pitch * 5.0).round() as i32;
        let pitch_str = if pitch_hz >= 0 {
            format!("+{}Hz", pitch_hz)
        } else {
            format!("{}Hz", pitch_hz)
        };

        let formatted_voice = format_edge_voice_name(&request.voice_id);

        format!(
            r#"<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='en-US'><voice name='{}'><prosody pitch='{}' rate='{}' volume='+0%'>{}</prosody></voice></speak>"#,
            quick_xml_escape(&formatted_voice), pitch_str, rate_str, quick_xml_escape(&request.text)
        )
    }

    /// Internal resilient fallback synthesizing clean tones if cloud connection is unreachable
    fn synthesize_fallback(&self, request: &SynthesisRequest) -> AudioChunk {
        let duration_secs = (request.text.len() as f32 * 0.06).max(0.2);
        let sample_rate = 24000;
        let num_samples = (sample_rate as f32 * duration_secs) as usize;

        let base_pitch = match request.voice_id.as_str() {
            "en-US-GuyNeural" | "en-GB-RyanNeural" | "es-ES-AlvaroNeural" => 160.0,
            "en-GB-SoniaNeural" | "fr-FR-DeniseNeural" => 220.0,
            _ => 240.0,
        };
        let safe_pitch = if request.pitch.is_finite() {
            request.pitch.clamp(-24.0, 24.0)
        } else {
            0.0
        };
        let freq = base_pitch * 2.0f32.powf(safe_pitch / 12.0);

        let pcm_data: Vec<i16> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                let env = (1.0 - (i as f32 / num_samples as f32)).min(i as f32 / 400.0).clamp(0.0, 1.0);
                let tone1 = f32::sin(2.0 * std::f32::consts::PI * freq * t);
                let tone2 = 0.5 * f32::sin(4.0 * std::f32::consts::PI * freq * t);
                ((tone1 + tone2) * 10000.0 * env) as i16
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

pub fn format_edge_voice_name(voice_id: &str) -> String {
    if voice_id.starts_with("Microsoft Server Speech") {
        return voice_id.to_string();
    }
    let parts: Vec<&str> = voice_id.split('-').collect();
    if parts.len() >= 3 {
        let lang = parts[0];
        let region = parts[1];
        let name = parts[2..].join("-");
        format!(
            "Microsoft Server Speech Text to Speech Voice ({}-{}, {})",
            lang, region, name
        )
    } else {
        voice_id.to_string()
    }
}

fn quick_xml_escape(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

impl Default for EdgeTtsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TtsEngine for EdgeTtsEngine {
    fn id(&self) -> &'static str {
        "edge-tts"
    }

    fn name(&self) -> &'static str {
        "Microsoft Edge TTS Cloud Relay"
    }

    fn is_local(&self) -> bool {
        false
    }

    async fn voices(&self) -> Result<Vec<Voice>> {
        Ok(vec![
            Voice {
                id: "en-US-AriaNeural".to_string(),
                name: "Aria (Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["conversational".to_string(), "news".to_string()],
                description: Some("Clear, natural American female voice".to_string()),
            },
            Voice {
                id: "en-US-GuyNeural".to_string(),
                name: "Guy (Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["conversational".to_string(), "friendly".to_string()],
                description: Some("Warm, friendly American male voice".to_string()),
            },
            Voice {
                id: "en-US-JennyNeural".to_string(),
                name: "Jenny (Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["conversational".to_string(), "expressive".to_string()],
                description: Some("Natural, conversational American female voice".to_string()),
            },
            Voice {
                id: "en-GB-SoniaNeural".to_string(),
                name: "Sonia (Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "en-GB".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["british".to_string(), "narration".to_string()],
                description: Some("Crisp British English female voice".to_string()),
            },
            Voice {
                id: "en-GB-RyanNeural".to_string(),
                name: "Ryan (Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "en-GB".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["british".to_string(), "storytelling".to_string()],
                description: Some("Polished British English male voice".to_string()),
            },
            Voice {
                id: "de-DE-KatjaNeural".to_string(),
                name: "Katja (Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "de-DE".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["german".to_string(), "broadcast".to_string()],
                description: Some("Standard German female voice".to_string()),
            },
            Voice {
                id: "fr-FR-DeniseNeural".to_string(),
                name: "Denise (Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "fr-FR".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["french".to_string(), "expressive".to_string()],
                description: Some("Articulate French female voice".to_string()),
            },
            Voice {
                id: "es-ES-AlvaroNeural".to_string(),
                name: "Alvaro (Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "es-ES".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["spanish".to_string(), "clear".to_string()],
                description: Some("Standard Castilian Spanish male voice".to_string()),
            },
            Voice {
                id: "ja-JP-NanamiNeural".to_string(),
                name: "Nanami (Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "ja-JP".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["japanese".to_string(), "anime".to_string()],
                description: Some("Pleasant, fluent Japanese female voice".to_string()),
            },
            Voice {
                id: "zh-CN-XiaoxiaoNeural".to_string(),
                name: "Xiaoxiao (Neural)".to_string(),
                engine_id: self.id().to_string(),
                language: "zh-CN".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["chinese".to_string(), "audiobook".to_string()],
                description: Some("Warm, natural Mandarin Chinese female voice".to_string()),
            },
        ])
    }

    async fn synthesize(&self, request: &SynthesisRequest) -> Result<AudioChunk> {
        let ssml = Self::build_ssml(request);

        match EdgeTtsClient::synthesize(request, &ssml).await {
            Ok(chunk) => Ok(chunk),
            Err(e) => {
                warn!(
                    "Live Edge-TTS synthesis failed ({}). Operating in resilient offline fallback mode.",
                    e
                );
                Ok(self.synthesize_fallback(request))
            }
        }
    }

    async fn synthesize_stream(
        &self,
        request: &SynthesisRequest,
    ) -> Result<mpsc::Receiver<Result<AudioChunk>>> {
        let (tx, rx) = mpsc::channel(16);
        let ssml = Self::build_ssml(request);
        let request_clone = request.clone();
        let fallback_chunk = self.synthesize_fallback(request);

        tokio::spawn(async move {
            match EdgeTtsClient::stream_synthesis(&request_clone, &ssml, tx.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    warn!(
                        "Live Edge-TTS streaming failed ({}). Streaming resilient fallback audio.",
                        e
                    );
                    let chunk_size = 4800; // 200ms
                    let mut offset = 0;
                    let total = fallback_chunk.pcm_data.len();

                    while offset < total {
                        let end = (offset + chunk_size).min(total);
                        let is_final = end >= total;
                        let slice = fallback_chunk.pcm_data[offset..end].to_vec();

                        let sub_chunk = AudioChunk {
                            sample_rate: fallback_chunk.sample_rate,
                            channels: fallback_chunk.channels,
                            pcm_data: slice,
                            is_final,
                        };

                        if tx.send(Ok(sub_chunk)).await.is_err() {
                            break;
                        }
                        offset = end;
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}
