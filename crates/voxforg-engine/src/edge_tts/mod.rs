pub mod client;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::warn;
use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::{AudioChunk, Gender, Voice};

use crate::traits::{EngineCapabilities, SynthesisRequest, TtsEngine};
pub use client::{
    decode_mp3_to_pcm, generate_sec_ms_gec, parse_binary_audio_payload, EdgeTtsClient,
};

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
        let xml_lang = extract_locale_from_voice_id(&request.voice_id);

        format!(
            r#"<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='{}'><voice name='{}'><prosody pitch='{}' rate='{}' volume='+0%'>{}</prosody></voice></speak>"#,
            quick_xml_escape(&xml_lang),
            quick_xml_escape(&formatted_voice),
            pitch_str,
            rate_str,
            quick_xml_escape(&request.text)
        )
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

pub fn extract_locale_from_voice_id(voice_id: &str) -> String {
    if voice_id.contains('(') && voice_id.contains(',') {
        if let Some(start) = voice_id.find('(') {
            if let Some(comma) = voice_id.find(',') {
                if comma > start {
                    return voice_id[start + 1..comma].trim().to_string();
                }
            }
        }
    }
    let parts: Vec<&str> = voice_id.split('-').collect();
    if parts.len() >= 2 && parts[0].len() >= 2 && parts[1].len() >= 2 {
        format!("{}-{}", parts[0], parts[1])
    } else {
        "en-US".to_string()
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

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            cost_per_1k_chars: 0.0, // free tier via browser relay
            avg_latency_ms: 300,
            quality_score: 0.85,
            languages: vec![
                "en-US".to_string(),
                "en-GB".to_string(),
                "ml-IN".to_string(),
                "hi-IN".to_string(),
                "ta-IN".to_string(),
                "te-IN".to_string(),
                "kn-IN".to_string(),
                "bn-IN".to_string(),
                "mr-IN".to_string(),
                "gu-IN".to_string(),
                "ur-IN".to_string(),
                "pa-IN".to_string(),
                "de-DE".to_string(),
                "fr-FR".to_string(),
                "es-ES".to_string(),
                "zh-CN".to_string(),
                "ja-JP".to_string(),
            ],
            is_local: false,
            supports_cloning: false,
            supports_streaming: true,
        }
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
            // --- Indian Languages Neural Voices ---
            Voice {
                id: "ml-IN-SobhanaNeural".to_string(),
                name: "Sobhana (Neural - മലയാളം)".to_string(),
                engine_id: self.id().to_string(),
                language: "ml-IN".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec![
                    "malayalam".to_string(),
                    "kerala".to_string(),
                    "expressive".to_string(),
                ],
                description: Some(
                    "Warm, expressive Malayalam female voice with natural native cadence"
                        .to_string(),
                ),
            },
            Voice {
                id: "ml-IN-MidhunNeural".to_string(),
                name: "Midhun (Neural - മലയാളം)".to_string(),
                engine_id: self.id().to_string(),
                language: "ml-IN".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec![
                    "malayalam".to_string(),
                    "kerala".to_string(),
                    "broadcast".to_string(),
                ],
                description: Some(
                    "Crisp, authoritative Malayalam male voice for news and narration".to_string(),
                ),
            },
            Voice {
                id: "hi-IN-SwaraNeural".to_string(),
                name: "Swara (Neural - हिन्दी)".to_string(),
                engine_id: self.id().to_string(),
                language: "hi-IN".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["hindi".to_string(), "conversational".to_string()],
                description: Some(
                    "Natural Hindi female voice with authentic conversational rhythm".to_string(),
                ),
            },
            Voice {
                id: "hi-IN-MadhurNeural".to_string(),
                name: "Madhur (Neural - हिन्दी)".to_string(),
                engine_id: self.id().to_string(),
                language: "hi-IN".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["hindi".to_string(), "storytelling".to_string()],
                description: Some("Deep, engaging Hindi male voice".to_string()),
            },
            Voice {
                id: "ta-IN-PallaviNeural".to_string(),
                name: "Pallavi (Neural - தமிழ்)".to_string(),
                engine_id: self.id().to_string(),
                language: "ta-IN".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["tamil".to_string(), "expressive".to_string()],
                description: Some("Fluent Tamil female voice with crisp articulation".to_string()),
            },
            Voice {
                id: "ta-IN-ValluvarNeural".to_string(),
                name: "Valluvar (Neural - தமிழ்)".to_string(),
                engine_id: self.id().to_string(),
                language: "ta-IN".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["tamil".to_string(), "narration".to_string()],
                description: Some(
                    "Resonant Tamil male voice for stories and broadcast".to_string(),
                ),
            },
            Voice {
                id: "te-IN-ShrutiNeural".to_string(),
                name: "Shruti (Neural - తెలుగు)".to_string(),
                engine_id: self.id().to_string(),
                language: "te-IN".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["telugu".to_string(), "melodic".to_string()],
                description: Some("Melodic Telugu female voice with natural cadence".to_string()),
            },
            Voice {
                id: "te-IN-MohanNeural".to_string(),
                name: "Mohan (Neural - తెలుగు)".to_string(),
                engine_id: self.id().to_string(),
                language: "te-IN".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["telugu".to_string(), "professional".to_string()],
                description: Some("Clear, articulate Telugu male voice".to_string()),
            },
            Voice {
                id: "kn-IN-SapnaNeural".to_string(),
                name: "Sapna (Neural - ಕನ್ನಡ)".to_string(),
                engine_id: self.id().to_string(),
                language: "kn-IN".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["kannada".to_string(), "conversational".to_string()],
                description: Some("Engaging Kannada female voice with smooth flow".to_string()),
            },
            Voice {
                id: "kn-IN-GaganNeural".to_string(),
                name: "Gagan (Neural - ಕನ್ನಡ)".to_string(),
                engine_id: self.id().to_string(),
                language: "kn-IN".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["kannada".to_string(), "broadcast".to_string()],
                description: Some("Warm Kannada male voice".to_string()),
            },
            Voice {
                id: "bn-IN-TanishaaNeural".to_string(),
                name: "Tanishaa (Neural - বাংলা)".to_string(),
                engine_id: self.id().to_string(),
                language: "bn-IN".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["bengali".to_string(), "expressive".to_string()],
                description: Some("Sweet, expressive Bengali female voice".to_string()),
            },
            Voice {
                id: "bn-IN-BashkarNeural".to_string(),
                name: "Bashkar (Neural - বাংলা)".to_string(),
                engine_id: self.id().to_string(),
                language: "bn-IN".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["bengali".to_string(), "narration".to_string()],
                description: Some("Articulate Bengali male voice".to_string()),
            },
            Voice {
                id: "mr-IN-AarohiNeural".to_string(),
                name: "Aarohi (Neural - मराठी)".to_string(),
                engine_id: self.id().to_string(),
                language: "mr-IN".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["marathi".to_string(), "conversational".to_string()],
                description: Some("Polished Marathi female voice".to_string()),
            },
            Voice {
                id: "mr-IN-ManoharNeural".to_string(),
                name: "Manohar (Neural - मराठी)".to_string(),
                engine_id: self.id().to_string(),
                language: "mr-IN".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["marathi".to_string(), "storytelling".to_string()],
                description: Some("Deep Marathi male voice".to_string()),
            },
            Voice {
                id: "gu-IN-DhwaniNeural".to_string(),
                name: "Dhwani (Neural - ગુજરાતી)".to_string(),
                engine_id: self.id().to_string(),
                language: "gu-IN".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["gujarati".to_string(), "expressive".to_string()],
                description: Some("Clear Gujarati female voice".to_string()),
            },
            Voice {
                id: "gu-IN-NiranjanNeural".to_string(),
                name: "Niranjan (Neural - ગુજરાતી)".to_string(),
                engine_id: self.id().to_string(),
                language: "gu-IN".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["gujarati".to_string(), "friendly".to_string()],
                description: Some("Warm Gujarati male voice".to_string()),
            },
            Voice {
                id: "ur-IN-GulNeural".to_string(),
                name: "Gul (Neural - اردو)".to_string(),
                engine_id: self.id().to_string(),
                language: "ur-IN".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["urdu".to_string(), "poetic".to_string()],
                description: Some("Eloquent Urdu female voice".to_string()),
            },
            Voice {
                id: "ur-IN-SalmanNeural".to_string(),
                name: "Salman (Neural - اردو)".to_string(),
                engine_id: self.id().to_string(),
                language: "ur-IN".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["urdu".to_string(), "narration".to_string()],
                description: Some("Poetic and resonant Urdu male voice".to_string()),
            },
            Voice {
                id: "pa-IN-VaaniNeural".to_string(),
                name: "Vaani (Neural - ਪੰਜਾਬੀ)".to_string(),
                engine_id: self.id().to_string(),
                language: "pa-IN".to_string(),
                gender: Gender::Female,
                sample_rate_hz: 24000,
                tags: vec!["punjabi".to_string(), "energetic".to_string()],
                description: Some("Lively Punjabi female voice".to_string()),
            },
            Voice {
                id: "pa-IN-OjasNeural".to_string(),
                name: "Ojas (Neural - ਪੰਜਾਬੀ)".to_string(),
                engine_id: self.id().to_string(),
                language: "pa-IN".to_string(),
                gender: Gender::Male,
                sample_rate_hz: 24000,
                tags: vec!["punjabi".to_string(), "bold".to_string()],
                description: Some("Bold, energetic Punjabi male voice".to_string()),
            },
            // --- Global Languages ---
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
        EdgeTtsClient::synthesize(request, &ssml)
            .await
            .map_err(|e| VoxForgError::Engine(format!("Edge-TTS neural synthesis failed: {}", e)))
    }

    async fn synthesize_stream(
        &self,
        request: &SynthesisRequest,
    ) -> Result<mpsc::Receiver<Result<AudioChunk>>> {
        let (tx, rx) = mpsc::channel(16);
        let ssml = Self::build_ssml(request);
        let request_clone = request.clone();

        tokio::spawn(async move {
            if let Err(e) = EdgeTtsClient::stream_synthesis(&request_clone, &ssml, tx.clone()).await
            {
                warn!("Live Edge-TTS streaming error: {}", e);
                let _ = tx
                    .send(Err(VoxForgError::Engine(format!(
                        "Edge-TTS neural streaming failed: {}",
                        e
                    ))))
                    .await;
            }
        });

        Ok(rx)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}
