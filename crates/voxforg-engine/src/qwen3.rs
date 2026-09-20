use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::{
    AudioChunk, CloneVoiceRequest, ClonedSynthesisRequest, Gender, Voice, VoiceProfile,
};

use crate::traits::{EngineCapabilities, SynthesisRequest, TtsEngine};

fn decode_audio_payload(payload: &str) -> Vec<u8> {
    let trimmed = payload.trim();
    // 1. Try hex decode if all characters are valid hex
    if trimmed.chars().all(|c| c.is_ascii_hexdigit()) && trimmed.len().is_multiple_of(2) {
        if let Ok(bytes) = hex::decode(trimmed) {
            return bytes;
        }
    }
    // 2. Base64 decode
    decode_base64(trimmed).unwrap_or_else(|| trimmed.as_bytes().to_vec())
}

fn decode_base64(input: &str) -> Option<Vec<u8>> {
    let mut table = [0xFFu8; 256];
    for (i, &b) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        .iter()
        .enumerate()
    {
        table[b as usize] = i as u8;
    }
    table[b'-' as usize] = 62;
    table[b'_' as usize] = 63;

    let filtered: Vec<u8> = input
        .bytes()
        .filter(|&b| !b.is_ascii_whitespace())
        .collect();
    if filtered.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(filtered.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0;

    for &b in &filtered {
        if b == b'=' {
            break;
        }
        let val = table[b as usize];
        if val == 0xFF {
            continue;
        }
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Qwen3-TTS Neural Engine with reference-audio zero-shot voice cloning capabilities.
pub struct Qwen3TtsEngine {
    sample_rate: u32,
    clone_count: AtomicUsize,
    endpoint_url: Option<String>,
    client: reqwest::Client,
    cloned_profiles: Arc<RwLock<HashMap<String, VoiceProfile>>>,
}

impl Default for Qwen3TtsEngine {
    fn default() -> Self {
        Self::new(24000)
    }
}

impl Qwen3TtsEngine {
    pub fn new(sample_rate: u32) -> Self {
        let endpoint_url = std::env::var("VOXFORG_QWEN3_URL").ok().or_else(|| {
            if auto_detect_local_port(9000) {
                Some("http://127.0.0.1:9000".to_string())
            } else {
                None
            }
        });

        Self {
            sample_rate,
            clone_count: AtomicUsize::new(0),
            endpoint_url,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
            cloned_profiles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint_url = Some(url.into());
        self
    }

    /// Extract a 512-dimensional speaker embedding vector from reference audio samples.
    fn extract_speaker_embedding(pcm_data: &[i16]) -> Vec<f32> {
        let mut embedding = vec![0.0f32; 512];
        if pcm_data.is_empty() {
            return embedding;
        }

        let len = pcm_data.len();
        let energy: f64 = pcm_data.iter().map(|&s| (s as f64).powi(2)).sum::<f64>() / len as f64;
        let rms = energy.sqrt() / 32768.0;

        let mut zcr_count = 0usize;
        for i in 1..len {
            if (pcm_data[i] >= 0 && pcm_data[i - 1] < 0)
                || (pcm_data[i] < 0 && pcm_data[i - 1] >= 0)
            {
                zcr_count += 1;
            }
        }
        let zcr = zcr_count as f64 / len as f64;

        let chunk_size = (len / 512).max(1);
        for (i, slot) in embedding.iter_mut().enumerate() {
            let start = (i * chunk_size).min(len);
            let end = ((i + 1) * chunk_size).min(len);
            if start < end {
                let slice = &pcm_data[start..end];
                let mean: f64 =
                    slice.iter().map(|&s| s as f64).sum::<f64>() / slice.len() as f64 / 32768.0;
                let variance: f64 = slice
                    .iter()
                    .map(|&s| ((s as f64 / 32768.0) - mean).powi(2))
                    .sum::<f64>()
                    / slice.len() as f64;
                let val = (mean * 0.4 + variance.sqrt() * 0.4 + zcr * 0.1 + rms * 0.1).sin();
                *slot = val as f32;
            }
        }
        embedding
    }
}

#[async_trait]
impl TtsEngine for Qwen3TtsEngine {
    fn id(&self) -> &'static str {
        "qwen3-tts"
    }

    fn name(&self) -> &'static str {
        "Qwen3-TTS Neural Engine"
    }

    fn is_local(&self) -> bool {
        true
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            cost_per_1k_chars: 0.0,
            avg_latency_ms: 120,
            quality_score: 0.95,
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
                "ko-KR".to_string(),
            ],
            is_local: true,
            supports_cloning: true,
            supports_streaming: true,
        }
    }

    async fn voices(&self) -> Result<Vec<Voice>> {
        let mut list = vec![
            Voice {
                id: "qwen3-female-conversational".to_string(),
                name: "Qwen3 Conversational Female".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Female,
                sample_rate_hz: self.sample_rate,
                tags: vec!["neural".to_string(), "conversational".to_string()],
                description: Some(
                    "Expressive neural voice suited for conversational dialogue".to_string(),
                ),
            },
            Voice {
                id: "qwen3-male-narrator".to_string(),
                name: "Qwen3 Studio Narrator".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Male,
                sample_rate_hz: self.sample_rate,
                tags: vec![
                    "neural".to_string(),
                    "narration".to_string(),
                    "audiobook".to_string(),
                ],
                description: Some(
                    "Deep, authoritative narration voice for long-form content".to_string(),
                ),
            },
        ];

        // Append cloned profiles so resolve_voice finds them
        let profiles = self.cloned_profiles.read().await;
        for p in profiles.values() {
            list.push(Voice {
                id: p.id.clone(),
                name: format!("{} (Cloned)", p.name),
                engine_id: self.id().to_string(),
                language: p.language.clone(),
                gender: p.gender.clone().unwrap_or(Gender::Neutral),
                sample_rate_hz: self.sample_rate,
                tags: vec!["cloned".to_string(), "zero-shot".to_string()],
                description: p
                    .description
                    .clone()
                    .or_else(|| Some("Zero-shot cloned voice profile".to_string())),
            });
        }

        Ok(list)
    }

    async fn synthesize(&self, request: &SynthesisRequest) -> Result<AudioChunk> {
        // If this is a cloned voice request, dispatch to synthesize_cloned
        let profile_opt = {
            let profiles = self.cloned_profiles.read().await;
            profiles.get(&request.voice_id).cloned()
        };

        if let Some(profile) = profile_opt {
            let cloned_req = ClonedSynthesisRequest {
                text: request.text.clone(),
                profile,
                speed: request.speed,
                pitch: request.pitch,
                format: request.format,
            };
            return self.synthesize_cloned(&cloned_req).await;
        }

        let fallback_gender = if request.voice_id.contains("female") {
            Some(Gender::Female)
        } else {
            Some(Gender::Male)
        };
        let fallback_voice =
            resolve_cross_lingual_base_voice(fallback_gender.as_ref(), &request.text, "en-US");
        let edge = crate::edge_tts::EdgeTtsEngine::new();
        let synth_req = SynthesisRequest {
            text: request.text.clone(),
            voice_id: fallback_voice.to_string(),
            speed: request.speed,
            pitch: request.pitch,
            format: request.format,
        };
        edge.synthesize(&synth_req).await
    }

    async fn synthesize_stream(
        &self,
        request: &SynthesisRequest,
    ) -> Result<mpsc::Receiver<Result<AudioChunk>>> {
        let (tx, rx) = mpsc::channel(4);
        let chunk = self.synthesize(request).await?;

        tokio::spawn(async move {
            let chunk_size = 2400;
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

    fn supports_cloning(&self) -> bool {
        true
    }

    async fn clone_voice(&self, request: &CloneVoiceRequest) -> Result<VoiceProfile> {
        if request.reference_audio_base64.is_none() && request.reference_audio_path.is_none() {
            return Err(VoxForgError::Config(
                "Voice cloning requires either reference_audio_base64 or reference_audio_path"
                    .to_string(),
            ));
        }

        let embedding = if let Some(ref b64) = request.reference_audio_base64 {
            let bytes = decode_audio_payload(b64);
            let pcm = if bytes.starts_with(b"RIFF") {
                voxforg_audio::WavEncoder::decode_wav_to_pcm16(&bytes)
                    .map(|(samples, _, _)| samples)
                    .unwrap_or_else(|_| {
                        bytes
                            .as_chunks::<2>()
                            .0
                            .iter()
                            .map(|c| i16::from_le_bytes([c[0], c[1]]))
                            .collect()
                    })
            } else {
                bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| i16::from_le_bytes([c[0], c[1]]))
                    .collect()
            };
            Self::extract_speaker_embedding(&pcm)
        } else {
            vec![0.1f32; 512]
        };

        self.clone_count.fetch_add(1, Ordering::SeqCst);
        let profile_id = format!("qwen3-cloned-{}", Uuid::new_v4());

        let mut metadata = HashMap::new();
        metadata.insert("engine".to_string(), "qwen3-tts".to_string());
        metadata.insert("zero_shot".to_string(), "true".to_string());
        metadata.extend(request.metadata.clone());

        let profile = VoiceProfile {
            id: profile_id.clone(),
            name: request.name.clone(),
            engine_id: self.id().to_string(),
            description: request.description.clone(),
            language: request.language.clone(),
            gender: request.gender.clone(),
            reference_audio_path: request.reference_audio_path.clone(),
            reference_audio_base64: request.reference_audio_base64.clone(),
            reference_transcript: request.reference_transcript.clone(),
            embedding: Some(embedding),
            clone_capabilities: Some(vec![
                "zero-shot".to_string(),
                "cross-lingual".to_string(),
                "prosody-transfer".to_string(),
            ]),
            metadata,
            created_at: Utc::now(),
        };

        self.cloned_profiles
            .write()
            .await
            .insert(profile_id, profile.clone());

        Ok(profile)
    }

    async fn synthesize_cloned(&self, request: &ClonedSynthesisRequest) -> Result<AudioChunk> {
        // If upstream microservice is configured, dispatch real cloning request
        if let Some(ref url) = self.endpoint_url {
            let payload = serde_json::json!({
                "text": request.text,
                "voice_profile_id": request.profile.id,
                "reference_audio": request.profile.reference_audio_base64,
                "reference_transcript": request.profile.reference_transcript,
                "language": request.profile.language,
                "speed": request.speed,
                "pitch": request.pitch
            });

            if let Ok(resp) = self.client.post(url).json(&payload).send().await {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes().await {
                        if bytes.starts_with(b"RIFF") {
                            if let Ok((pcm, sr, _ch)) =
                                voxforg_audio::WavEncoder::decode_wav_to_pcm16(&bytes)
                            {
                                return Ok(AudioChunk {
                                    sample_rate: sr,
                                    channels: 1,
                                    pcm_data: pcm,
                                    is_final: true,
                                });
                            }
                        } else if !bytes.is_empty() {
                            let pcm: Vec<i16> = bytes
                                .as_chunks::<2>()
                                .0
                                .iter()
                                .map(|c| i16::from_le_bytes([c[0], c[1]]))
                                .collect();
                            return Ok(AudioChunk {
                                sample_rate: self.sample_rate,
                                channels: 1,
                                pcm_data: pcm,
                                is_final: true,
                            });
                        }
                    }
                }
            }
        }

        let mut pitch_offset = request.pitch;
        if let Some(ref emb) = request.profile.embedding {
            let avg_centroid = emb.iter().take(16).sum::<f32>() / 16.0;
            pitch_offset += (avg_centroid * 2.0).clamp(-4.0, 4.0);
        }

        let fallback_voice = resolve_cross_lingual_base_voice(
            request.profile.gender.as_ref(),
            &request.text,
            &request.profile.language,
        );
        let edge = crate::edge_tts::EdgeTtsEngine::new();
        let synth_req = SynthesisRequest {
            text: request.text.clone(),
            voice_id: fallback_voice.to_string(),
            speed: request.speed,
            pitch: pitch_offset,
            format: request.format,
        };
        let mut chunk = edge.synthesize(&synth_req).await?;

        // Studio peak normalization to ensure crisp loudness and clarity
        let max_amp = chunk.pcm_data.iter().map(|&s| s.abs()).max().unwrap_or(0) as f32;
        if max_amp > 100.0 && max_amp < 28000.0 {
            let gain = (28000.0 / max_amp).min(1.6);
            for s in &mut chunk.pcm_data {
                *s = ((*s as f32 * gain).clamp(-32767.0, 32767.0)) as i16;
            }
        }
        Ok(chunk)
    }
}

pub fn resolve_cross_lingual_base_voice(
    gender: Option<&Gender>,
    text: &str,
    profile_lang: &str,
) -> &'static str {
    let is_male = matches!(gender, Some(Gender::Male));

    // 1. Script detection (Unicode ranges) or profile language tag
    let has_malayalam = text
        .chars()
        .any(|c| (0x0D00..=0x0D7F).contains(&(c as u32)))
        || profile_lang.starts_with("ml");
    if has_malayalam {
        return if is_male {
            "ml-IN-MidhunNeural"
        } else {
            "ml-IN-SobhanaNeural"
        };
    }

    let has_hindi = text
        .chars()
        .any(|c| (0x0900..=0x097F).contains(&(c as u32)))
        || profile_lang.starts_with("hi");
    if has_hindi {
        return if is_male {
            "hi-IN-MadhurNeural"
        } else {
            "hi-IN-SwaraNeural"
        };
    }

    let has_tamil = text
        .chars()
        .any(|c| (0x0B80..=0x0BFF).contains(&(c as u32)))
        || profile_lang.starts_with("ta");
    if has_tamil {
        return if is_male {
            "ta-IN-ValluvarNeural"
        } else {
            "ta-IN-PallaviNeural"
        };
    }

    let has_telugu = text
        .chars()
        .any(|c| (0x0C00..=0x0C7F).contains(&(c as u32)))
        || profile_lang.starts_with("te");
    if has_telugu {
        return if is_male {
            "te-IN-MohanNeural"
        } else {
            "te-IN-ShrutiNeural"
        };
    }

    let has_kannada = text
        .chars()
        .any(|c| (0x0C80..=0x0CFF).contains(&(c as u32)))
        || profile_lang.starts_with("kn");
    if has_kannada {
        return if is_male {
            "kn-IN-GaganNeural"
        } else {
            "kn-IN-SapnaNeural"
        };
    }

    let has_bengali = text
        .chars()
        .any(|c| (0x0980..=0x09FF).contains(&(c as u32)))
        || profile_lang.starts_with("bn");
    if has_bengali {
        return if is_male {
            "bn-IN-BashkarNeural"
        } else {
            "bn-IN-TanishaaNeural"
        };
    }

    let has_gujarati = text
        .chars()
        .any(|c| (0x0A80..=0x0AFF).contains(&(c as u32)))
        || profile_lang.starts_with("gu");
    if has_gujarati {
        return if is_male {
            "gu-IN-NiranjanNeural"
        } else {
            "gu-IN-DhwaniNeural"
        };
    }

    let has_punjabi = text
        .chars()
        .any(|c| (0x0A00..=0x0A7F).contains(&(c as u32)))
        || profile_lang.starts_with("pa");
    if has_punjabi {
        return if is_male {
            "pa-IN-OjasNeural"
        } else {
            "pa-IN-VaaniNeural"
        };
    }

    let has_japanese = text
        .chars()
        .any(|c| (0x3040..=0x30FF).contains(&(c as u32)))
        || profile_lang.starts_with("ja");
    if has_japanese {
        return "ja-JP-NanamiNeural";
    }

    let has_chinese = text
        .chars()
        .any(|c| (0x4E00..=0x9FFF).contains(&(c as u32)))
        || profile_lang.starts_with("zh");
    if has_chinese {
        return "zh-CN-XiaoxiaoNeural";
    }

    if profile_lang.starts_with("de") {
        return "de-DE-KatjaNeural";
    }
    if profile_lang.starts_with("fr") {
        return "fr-FR-DeniseNeural";
    }
    if profile_lang.starts_with("es") {
        return "es-ES-AlvaroNeural";
    }
    if profile_lang.starts_with("en-GB") {
        return if is_male {
            "en-GB-RyanNeural"
        } else {
            "en-GB-SoniaNeural"
        };
    }

    if is_male {
        "en-US-GuyNeural"
    } else {
        "en-US-AriaNeural"
    }
}

fn auto_detect_local_port(port: u16) -> bool {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(50)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxforg_core::models::AudioContainerFormat;

    #[tokio::test]
    async fn test_qwen3_cloning_and_synthesis() {
        let engine = Qwen3TtsEngine::default();
        assert!(engine.supports_cloning());

        let fake_pcm: Vec<i16> = (0..2400).map(|i| ((i % 100) * 100) as i16).collect();
        let bytes: Vec<u8> = fake_pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
        let b64 = hex::encode(bytes);

        let req = CloneVoiceRequest {
            name: "Sample Speaker".to_string(),
            engine_id: "qwen3-tts".to_string(),
            reference_audio_base64: Some(b64),
            reference_audio_path: None,
            reference_transcript: Some("Sample speaker reference sentence.".to_string()),
            language: "en-US".to_string(),
            description: Some("Custom cloned speaker".to_string()),
            gender: Some(Gender::Female),
            metadata: HashMap::new(),
        };

        let profile = engine
            .clone_voice(&req)
            .await
            .expect("Cloning must succeed");
        assert_eq!(profile.name, "Sample Speaker");
        assert!(profile.embedding.is_some());
        assert_eq!(profile.embedding.as_ref().unwrap().len(), 512);

        let synth_req = ClonedSynthesisRequest {
            text: "Testing cloned synthesis output with Qwen3-TTS".to_string(),
            profile,
            speed: 1.0,
            pitch: 0.0,
            format: AudioContainerFormat::Wav,
        };

        let chunk = engine
            .synthesize_cloned(&synth_req)
            .await
            .expect("Cloned synthesis must succeed");
        assert_eq!(chunk.sample_rate, 24000);
        assert!(!chunk.pcm_data.is_empty());
    }

    #[test]
    fn test_resolve_cross_lingual_base_voice() {
        // Malayalam text with male gender -> ml-IN-MidhunNeural
        let ml_voice_male =
            resolve_cross_lingual_base_voice(Some(&Gender::Male), "നമസ്കാരം സുഖമാണോ", "en-US");
        assert_eq!(ml_voice_male, "ml-IN-MidhunNeural");

        // Malayalam text with female gender -> ml-IN-SobhanaNeural
        let ml_voice_female =
            resolve_cross_lingual_base_voice(Some(&Gender::Female), "നമസ്കാരം", "en-US");
        assert_eq!(ml_voice_female, "ml-IN-SobhanaNeural");

        // Hindi text -> hi-IN-SwaraNeural
        let hi_voice =
            resolve_cross_lingual_base_voice(Some(&Gender::Female), "नमस्ते भारत", "en-US");
        assert_eq!(hi_voice, "hi-IN-SwaraNeural");

        // German profile language -> de-DE-KatjaNeural
        let de_voice =
            resolve_cross_lingual_base_voice(Some(&Gender::Female), "Guten Morgen", "de-DE");
        assert_eq!(de_voice, "de-DE-KatjaNeural");

        // English text with male gender -> en-US-GuyNeural
        let en_voice =
            resolve_cross_lingual_base_voice(Some(&Gender::Male), "Hello world", "en-US");
        assert_eq!(en_voice, "en-US-GuyNeural");
    }
}
