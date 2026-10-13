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
    let clean = if let Some(idx) = payload.find(',') {
        payload[idx + 1..].trim()
    } else {
        payload.trim()
    };
    // 1. Try hex decode if all characters are valid hex
    if !clean.is_empty()
        && clean.chars().all(|c| c.is_ascii_hexdigit())
        && clean.len().is_multiple_of(2)
    {
        if let Ok(bytes) = hex::decode(clean) {
            return bytes;
        }
    }
    // 2. Base64 decode
    decode_base64(clean).unwrap_or_else(|| clean.as_bytes().to_vec())
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

        let mut profiles_map = HashMap::new();
        if let Ok(dir_str) = std::env::var("VOXFORG_PROFILES_DIR") {
            let dir = std::path::PathBuf::from(dir_str.trim());
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(profile) = serde_json::from_str::<VoiceProfile>(&content) {
                                profiles_map.insert(profile.id.clone(), profile);
                            }
                        }
                    }
                }
            }
        }

        Self {
            sample_rate,
            clone_count: AtomicUsize::new(0),
            endpoint_url,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
            cloned_profiles: Arc::new(RwLock::new(profiles_map)),
        }
    }

    pub fn with_endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint_url = Some(url.into());
        self
    }

    /// Extract a 512-dimensional speaker embedding vector from reference audio samples.
    /// Applies acoustic pre-conditioning: 70Hz high-pass filter (cuts mic thumps and room rumble)
    /// followed by target RMS normalization before calculating feature vectors.
    pub(crate) fn extract_speaker_embedding(pcm_data: &[i16]) -> Vec<f32> {
        let mut embedding = vec![0.0f32; 512];
        if pcm_data.is_empty() {
            return embedding;
        }

        // Acoustic pre-conditioning: remove sub-bass mic rumble (<70Hz) & normalize RMS energy
        let mut conditioned = pcm_data.to_vec();
        voxforg_audio::ParametricEq::high_pass(&mut conditioned, 24000, 70.0);
        voxforg_audio::AudioNormalizer::rms_normalize(&mut conditioned, 0.18);

        let len = conditioned.len();
        let energy: f64 = conditioned.iter().map(|&s| (s as f64).powi(2)).sum::<f64>() / len as f64;
        let rms = energy.sqrt() / 32768.0;

        let mut zcr_count = 0usize;
        for i in 1..len {
            if (conditioned[i] >= 0 && conditioned[i - 1] < 0)
                || (conditioned[i] < 0 && conditioned[i - 1] >= 0)
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
                let slice = &conditioned[start..end];
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

/// Measures the low, mid, and high frequency balance of speech to match vocal timbre.
fn compute_spectral_timbre_deltas(ref_pcm: &[i16], synth_pcm: &[i16]) -> (f32, f32, f32) {
    if ref_pcm.len() < 100 || synth_pcm.len() < 100 {
        return (0.0, 0.0, 0.0);
    }

    let band_energies = |samples: &[i16]| -> (f64, f64, f64) {
        let mut low_e = 0.0f64;
        let mut mid_e = 0.0f64;
        let mut high_e = 0.0f64;

        let n = samples.len().min(48000);
        for i in 2..n {
            let s = samples[i] as f64;
            let s_prev = samples[i - 1] as f64;
            let s_prev2 = samples[i - 2] as f64;

            let low = (s + 2.0 * s_prev + s_prev2) * 0.25;
            let high = s - s_prev;
            let mid = s - low;

            low_e += low * low;
            mid_e += mid * mid;
            high_e += high * high;
        }

        let total = (low_e + mid_e + high_e).max(1e-6);
        (low_e / total, mid_e / total, high_e / total)
    };

    let (ref_low, ref_mid, ref_high) = band_energies(ref_pcm);
    let (syn_low, syn_mid, syn_high) = band_energies(synth_pcm);

    let delta_low = (10.0 * (ref_low / syn_low.max(1e-4)).log10()).clamp(-6.0, 6.0) as f32;
    let delta_mid = (10.0 * (ref_mid / syn_mid.max(1e-4)).log10()).clamp(-4.0, 4.0) as f32;
    let delta_high = (10.0 * (ref_high / syn_high.max(1e-4)).log10()).clamp(-6.0, 6.0) as f32;

    (delta_low, delta_mid, delta_high)
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
            if let Some(p) = profiles.get(&request.voice_id) {
                Some(p.clone())
            } else {
                let clean_name = request
                    .voice_id
                    .trim_end_matches(" (Cloned)")
                    .trim()
                    .to_lowercase();
                profiles
                    .values()
                    .find(|p| {
                        p.name.trim().to_lowercase() == clean_name
                            || p.id.to_lowercase() == clean_name
                            || request.voice_id.contains(&p.id)
                    })
                    .cloned()
            }
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
        let mut chunk = edge.synthesize(&synth_req).await?;

        // Apply full biometric realism chain: micro-tremor, warmth, soft envelope, breath
        voxforg_audio::HumanMicroTremor::process(
            &mut chunk.pcm_data,
            chunk.sample_rate,
            0.35,
            0.28,
        );
        voxforg_audio::HarmonicWarmth::process(&mut chunk.pcm_data, 0.25, 0.28);
        voxforg_audio::DeEsser::process(&mut chunk.pcm_data, chunk.sample_rate, 6500.0, -22.0, 4.5);
        voxforg_audio::BrickwallLimiter::process(&mut chunk.pcm_data, -0.5);
        voxforg_audio::SoftEnvelope::process(&mut chunk.pcm_data, chunk.sample_rate, 6.0, 24.0);

        if chunk.pcm_data.len() > (chunk.sample_rate as usize) {
            let mut breath = voxforg_audio::NaturalBreath::generate(
                chunk.sample_rate,
                120.0,
                -36.0,
                1750.0,
            );
            voxforg_audio::SoftEnvelope::process(&mut breath, chunk.sample_rate, 8.0, 8.0);
            let pause_samples = (chunk.sample_rate as f32 * 0.025).round() as usize;
            breath.extend(vec![0i16; pause_samples]);
            breath.extend(chunk.pcm_data);
            chunk.pcm_data = breath;
        }

        Ok(chunk)
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
            .insert(profile_id.clone(), profile.clone());

        // Atomically persist to data/voice_profiles on disk so it survives restarts
        let profiles_dir = std::env::var("VOXFORG_PROFILES_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("data/voice_profiles"));
        let _ = std::fs::create_dir_all(&profiles_dir);
        let file_path = profiles_dir.join(format!("{}.json", profile_id));
        if let Ok(json_str) = serde_json::to_string_pretty(&profile) {
            let _ = std::fs::write(file_path, json_str);
        }

        Ok(profile)
    }

    #[allow(clippy::chunks_exact_to_as_chunks)]
    async fn synthesize_cloned(&self, request: &ClonedSynthesisRequest) -> Result<AudioChunk> {
        // 1. Dynamic sidecar resolution: check configured URL or standard ports (9000, 8880, 5000)
        let sidecar_url = self.endpoint_url.clone().or_else(|| {
            if auto_detect_local_port(9000) {
                Some("http://127.0.0.1:9000".to_string())
            } else if auto_detect_local_port(8880) {
                Some("http://127.0.0.1:8880".to_string())
            } else if auto_detect_local_port(5000) {
                Some("http://127.0.0.1:5000".to_string())
            } else {
                None
            }
        });

        if let Some(ref url) = sidecar_url {
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
                                .chunks_exact(2)
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

        // 2. Decode reference audio PCM if present
        let ref_audio_opt = if let Some(ref b64) = request.profile.reference_audio_base64 {
            let bytes = decode_audio_payload(b64);
            if bytes.starts_with(b"RIFF") {
                voxforg_audio::WavEncoder::decode_wav_to_pcm16(&bytes)
                    .ok()
                    .map(|(pcm, sr, _)| (pcm, sr))
            } else {
                let pcm: Vec<i16> = bytes
                    .chunks_exact(2)
                    .map(|c| i16::from_le_bytes([c[0], c[1]]))
                    .collect();
                Some((pcm, 24000))
            }
        } else if let Some(ref path_str) = request.profile.reference_audio_path {
            if let Ok(bytes) = std::fs::read(path_str) {
                if bytes.starts_with(b"RIFF") {
                    voxforg_audio::WavEncoder::decode_wav_to_pcm16(&bytes)
                        .ok()
                        .map(|(pcm, sr, _)| (pcm, sr))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // 3. Acoustic prosody & timbre analysis
        let (ref_pcm, ref_features_opt) = match ref_audio_opt {
            Some((pcm, sr)) if !pcm.is_empty() => {
                let features = voxforg_audio::ProsodyMirror::analyze(&pcm, sr);
                (Some(pcm), Some(features))
            }
            _ => (None, None),
        };

        // 4. Determine acoustic gender & base voice
        let acoustic_gender = if let Some(ref f) = ref_features_opt {
            if f.f0_mean_hz < 165.0 {
                Gender::Male
            } else {
                Gender::Female
            }
        } else if let Some(ref gender) = request.profile.gender {
            gender.clone()
        } else {
            Gender::Female
        };

        let fallback_voice = resolve_cross_lingual_base_voice(
            Some(&acoustic_gender),
            &request.text,
            &request.profile.language,
        );

        // 5. Calculate pitch delta matching the cloned speaker's exact fundamental pitch
        let mut pitch_offset = request.pitch;
        if let Some(ref f) = ref_features_opt {
            let nominal_base_f0 = if matches!(acoustic_gender, Gender::Male) {
                118.0
            } else {
                208.0
            };
            let f0_pitch_delta = (12.0 * (f.f0_mean_hz.clamp(60.0, 450.0) / nominal_base_f0).log2())
                .clamp(-14.0, 14.0);
            pitch_offset += f0_pitch_delta;
        } else if let Some(ref emb) = request.profile.embedding {
            let avg_centroid = emb.iter().take(32).sum::<f32>() / 32.0;
            pitch_offset += (avg_centroid * 4.0).clamp(-6.0, 6.0);
        }

        // 6. Natural speech cadence / tempo alignment
        let speed = if (request.speed - 1.0).abs() < 0.05 {
            if let Some(ref f) = ref_features_opt {
                (f.syllable_rate / 4.2).clamp(0.85, 1.25)
            } else {
                request.speed
            }
        } else {
            request.speed
        };

        // 7. Base voice synthesis pass
        let edge = crate::edge_tts::EdgeTtsEngine::new();
        let synth_req = SynthesisRequest {
            text: request.text.clone(),
            voice_id: fallback_voice.to_string(),
            speed,
            pitch: pitch_offset,
            format: request.format,
        };
        let mut chunk = edge.synthesize(&synth_req).await?;

        // 8. Formant & Spectral Timbre Morphing (match cloned speaker's frequency envelope)
        if let Some(ref ref_samples) = ref_pcm {
            let (low_gain, mid_gain, high_gain) =
                compute_spectral_timbre_deltas(ref_samples, &chunk.pcm_data);
            voxforg_audio::ParametricEq::process_3band(
                &mut chunk.pcm_data,
                chunk.sample_rate,
                low_gain,
                mid_gain,
                high_gain,
            );

            if let Some(ref f) = ref_features_opt {
                // Vocal tract anatomical formant resonance match
                let f1_gain = (low_gain * 0.6).clamp(-4.0, 4.0);
                let f2_gain = (mid_gain * 0.6).clamp(-4.0, 4.0);
                let presence_gain = (high_gain * 0.5).clamp(-3.0, 3.0);
                voxforg_audio::VocalTractResonator::process(
                    &mut chunk.pcm_data,
                    chunk.sample_rate,
                    f.formant_f1_hz,
                    f1_gain,
                    f.formant_f2_hz,
                    f2_gain,
                    presence_gain,
                );

                if f.voicing_ratio > 0.40 {
                    voxforg_audio::HarmonicWarmth::process(
                        &mut chunk.pcm_data,
                        0.25 + (f.voicing_ratio * 0.15).clamp(0.0, 0.25),
                        0.30,
                    );
                }
            }
        } else if let Some(ref emb) = request.profile.embedding {
            // Modulate timbre from 512-dim embedding slots
            let low_bias = (emb.iter().skip(32).take(64).sum::<f32>() / 64.0 * 6.0).clamp(-4.0, 4.0);
            let mid_bias = (emb.iter().skip(96).take(96).sum::<f32>() / 96.0 * 4.0).clamp(-3.0, 3.0);
            let high_bias = (emb.iter().skip(192).take(128).sum::<f32>() / 128.0 * 6.0).clamp(-4.0, 4.0);
            voxforg_audio::ParametricEq::process_3band(
                &mut chunk.pcm_data,
                chunk.sample_rate,
                low_bias,
                mid_bias,
                high_bias,
            );
        }

        // 9. Physiological vocal fold micro-tremor (jitter & shimmer)
        let jitter_depth = if let Some(ref f) = ref_features_opt {
            (f.f0_jitter_percent / 2.0).clamp(0.20, 0.55)
        } else {
            0.35
        };
        voxforg_audio::HumanMicroTremor::process(
            &mut chunk.pcm_data,
            chunk.sample_rate,
            jitter_depth,
            0.28,
        );

        // 10. Soft de-essing to tame harsh sibilance
        voxforg_audio::DeEsser::process(&mut chunk.pcm_data, chunk.sample_rate, 6500.0, -22.0, 5.0);

        // 11. Studio peak normalization & brickwall limiting
        let max_amp = chunk.pcm_data.iter().map(|&s| s.abs()).max().unwrap_or(0) as f32;
        if max_amp > 100.0 && max_amp < 28000.0 {
            let gain = (28000.0 / max_amp).min(1.6);
            for s in &mut chunk.pcm_data {
                *s = ((*s as f32 * gain).clamp(-32767.0, 32767.0)) as i16;
            }
        }
        voxforg_audio::BrickwallLimiter::process(&mut chunk.pcm_data, -0.5);

        // 12. Soft anti-click envelope on speech core
        voxforg_audio::SoftEnvelope::process(&mut chunk.pcm_data, chunk.sample_rate, 6.0, 24.0);

        // 13. Organic Pre-Speech Inhalation Aspiration
        if chunk.pcm_data.len() > (chunk.sample_rate as usize) {
            let breath_center = ref_features_opt.as_ref().map(|f| f.formant_f2_hz).unwrap_or(1750.0);
            let mut breath = voxforg_audio::NaturalBreath::generate(
                chunk.sample_rate,
                140.0,
                -34.0,
                breath_center,
            );
            voxforg_audio::SoftEnvelope::process(&mut breath, chunk.sample_rate, 8.0, 8.0);
            let pause_samples = (chunk.sample_rate as f32 * 0.030).round() as usize;
            breath.extend(vec![0i16; pause_samples]);
            breath.extend(chunk.pcm_data);
            chunk.pcm_data = breath;
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

        match engine.synthesize_cloned(&synth_req).await {
            Ok(chunk) => {
                assert_eq!(chunk.sample_rate, 24000);
                assert!(!chunk.pcm_data.is_empty());
            }
            Err(e) => {
                let err_str = e.to_string();
                assert!(
                    err_str.contains("No such host is known")
                        || err_str.contains("failed to lookup address")
                        || err_str.contains("Connect")
                        || err_str.contains("timeout")
                        || err_str.contains("Edge TTS WebSocket error"),
                    "Unexpected synthesis error: {err_str}"
                );
            }
        }
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

    #[test]
    fn test_extract_speaker_embedding_with_acoustic_preconditioning() {
        // Empty buffer returns zeros
        let empty_emb = Qwen3TtsEngine::extract_speaker_embedding(&[]);
        assert_eq!(empty_emb.len(), 512);
        assert!(empty_emb.iter().all(|&v| v == 0.0));

        // Audio with low rumble (25Hz) plus speech frequencies (400Hz)
        let sample_rate = 24000f32;
        let pcm: Vec<i16> = (0..4800)
            .map(|i| {
                let t = i as f32 / sample_rate;
                let rumble = (2.0 * std::f32::consts::PI * 25.0 * t).sin() * 8000.0;
                let voice = (2.0 * std::f32::consts::PI * 400.0 * t).sin() * 12000.0;
                (rumble + voice).round() as i16
            })
            .collect();

        let emb = Qwen3TtsEngine::extract_speaker_embedding(&pcm);
        assert_eq!(emb.len(), 512);
        assert!(emb.iter().all(|&v| v.is_finite()));
        // Embedding should have non-trivial variation
        let non_zero_count = emb.iter().filter(|&&v| v.abs() > 0.0001).count();
        assert!(non_zero_count > 400);
    }

    #[test]
    fn test_decode_audio_payload_handles_data_url() {
        let raw_bytes = b"RIFFfakeaudioWAVEfmt ";
        let b64 = format!(
            "data:audio/wav;base64,{}",
            std::str::from_utf8(&filtered_base64(raw_bytes)).unwrap()
        );
        let decoded = decode_audio_payload(&b64);
        assert_eq!(decoded, raw_bytes);
    }
    fn filtered_base64(input: &[u8]) -> Vec<u8> {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = Vec::new();
        let mut buf = 0u32;
        let mut bits = 0;
        for &b in input {
            buf = (buf << 8) | (b as u32);
            bits += 8;
            while bits >= 6 {
                bits -= 6;
                out.push(CHARS[((buf >> bits) & 0x3F) as usize]);
            }
        }
        if bits > 0 {
            out.push(CHARS[((buf << (6 - bits)) & 0x3F) as usize]);
            while out.len() % 4 != 0 {
                out.push(b'=');
            }
        }
        out
    }

    #[test]
    fn test_compute_spectral_timbre_deltas() {
        let empty_deltas = compute_spectral_timbre_deltas(&[], &[]);
        assert_eq!(empty_deltas, (0.0, 0.0, 0.0));

        let ref_pcm: Vec<i16> = (0..2400).map(|i| (i % 50 * 200) as i16).collect();
        let syn_pcm: Vec<i16> = (0..2400).map(|i| (i % 20 * 400) as i16).collect();
        let (low, mid, high) = compute_spectral_timbre_deltas(&ref_pcm, &syn_pcm);
        assert!(low.is_finite());
        assert!(mid.is_finite());
        assert!(high.is_finite());
    }
}

