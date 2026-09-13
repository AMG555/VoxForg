use std::sync::Arc;
use voxforg_asr::AsrRegistry;
use voxforg_audio::{
    AudioMerger, AudioNormalizer, BrickwallLimiter, DynamicCompressor, ParametricEq,
    SilenceTrimmer, WavEncoder,
};
use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::{NodeType, PipelineDefinition};
use voxforg_engine::{EngineRegistry, SynthesisRequest};

use crate::context::{ExecutionContext, ScriptSegment};
use crate::graph::GraphValidator;

pub struct PipelineExecutor {
    engine_registry: Arc<EngineRegistry>,
    asr_registry: Arc<AsrRegistry>,
}

impl PipelineExecutor {
    pub fn new(engine_registry: Arc<EngineRegistry>) -> Self {
        Self {
            engine_registry,
            asr_registry: Arc::new(AsrRegistry::with_defaults()),
        }
    }

    pub fn with_asr_registry(mut self, asr_registry: Arc<AsrRegistry>) -> Self {
        self.asr_registry = asr_registry;
        self
    }

    pub async fn execute(
        &self,
        pipeline: &PipelineDefinition,
        initial_text: Option<String>,
    ) -> Result<Vec<u8>> {
        let order = GraphValidator::topological_sort(pipeline)?;
        let mut ctx = ExecutionContext::new();

        if let Some(text) = initial_text {
            ctx.raw_text = Some(text);
        }

        for node_id in order {
            let node = pipeline
                .nodes
                .iter()
                .find(|n| n.id == node_id)
                .ok_or_else(|| VoxForgError::PipelineExecution {
                    node_id: node_id.clone(),
                    reason: "Node missing in definition".to_string(),
                })?;

            self.execute_node(node, &mut ctx).await.map_err(|e| {
                VoxForgError::PipelineExecution {
                    node_id: node.id.clone(),
                    reason: e.to_string(),
                }
            })?;
        }

        // Format master audio buffer
        let wav_bytes = WavEncoder::encode_pcm16_to_wav(
            &ctx.master_audio_pcm,
            ctx.sample_rate.max(8000),
            ctx.channels.max(1),
        )?;

        Ok(wav_bytes)
    }

    async fn execute_node(
        &self,
        node: &voxforg_core::models::PipelineNode,
        ctx: &mut ExecutionContext,
    ) -> Result<()> {
        match node.node_type {
            NodeType::TextInput => {
                if let Some(text_val) = node.params.get("text").and_then(|t| t.as_str()) {
                    if text_val.len() > 50_000 {
                        return Err(VoxForgError::PipelineExecution {
                            node_id: node.id.clone(),
                            reason:
                                "TextInput exceeds maximum supported length of 50,000 characters"
                                    .to_string(),
                        });
                    }
                    ctx.raw_text = Some(text_val.to_string());
                }
                Ok(())
            }

            NodeType::SpeakerParser => {
                let text = ctx.raw_text.as_deref().unwrap_or_default();
                let mut segments = Vec::new();

                // Check for dialog rules or fallback to paragraph chunking
                let lines: Vec<&str> = text
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect();
                for line in lines {
                    if let Some((speaker, utterance)) = line.split_once(':') {
                        segments.push(ScriptSegment {
                            speaker: speaker.trim().to_string(),
                            text: utterance.trim().to_string(),
                            voice_id: None,
                            speed: None,
                            pitch: None,
                            target_duration_ms: None,
                        });
                    } else {
                        segments.push(ScriptSegment {
                            speaker: "Narrator".to_string(),
                            text: line.to_string(),
                            voice_id: None,
                            speed: None,
                            pitch: None,
                            target_duration_ms: None,
                        });
                    }
                }

                if segments.is_empty() && !text.is_empty() {
                    segments.push(ScriptSegment {
                        speaker: "Narrator".to_string(),
                        text: text.to_string(),
                        voice_id: None,
                        speed: None,
                        pitch: None,
                        target_duration_ms: None,
                    });
                }

                ctx.segments = segments;
                Ok(())
            }

            NodeType::VoiceAssigner => {
                let default_voice = node
                    .params
                    .get("default_voice")
                    .and_then(|v| v.as_str())
                    .unwrap_or("en-US-AriaNeural");

                let speaker_map = node.params.get("speaker_map");

                for segment in &mut ctx.segments {
                    let mut assigned = default_voice.to_string();
                    if let Some(map) = speaker_map.and_then(|m| m.as_object()) {
                        if let Some(v) = map.get(&segment.speaker).and_then(|val| val.as_str()) {
                            assigned = v.to_string();
                        }
                    }
                    segment.voice_id = Some(assigned);
                }
                Ok(())
            }

            NodeType::Synthesizer => {
                ctx.audio_segments.clear();
                for segment in &ctx.segments {
                    let voice_id = segment.voice_id.as_deref().unwrap_or("en-US-AriaNeural");

                    let (engine, voice) = self.engine_registry.resolve_voice(voice_id).await?;
                    ctx.sample_rate = voice.sample_rate_hz;

                    let req = SynthesisRequest {
                        text: segment.text.clone(),
                        voice_id: voice_id.to_string(),
                        speed: segment.speed.unwrap_or(1.0),
                        pitch: segment.pitch.unwrap_or(0.0),
                        format: voxforg_core::models::AudioContainerFormat::Pcm,
                    };

                    let chunk = engine.synthesize(&req).await?;
                    ctx.audio_segments.push(chunk.pcm_data);
                }
                Ok(())
            }

            NodeType::AudioFilter => {
                // 1. Peak normalization
                if let Some(norm_val) = node.params.get("normalize_peak").and_then(|p| p.as_f64()) {
                    let target = norm_val as f32;
                    for segment_pcm in &mut ctx.audio_segments {
                        AudioNormalizer::peak_normalize(segment_pcm, target);
                    }
                } else if node.params.get("filter_type").and_then(|t| t.as_str())
                    == Some("normalize")
                    || node.params.is_null()
                    || node.params.as_object().is_none_or(|o| o.is_empty())
                {
                    for segment_pcm in &mut ctx.audio_segments {
                        AudioNormalizer::peak_normalize(segment_pcm, 0.95);
                    }
                }

                // 2. Gain in dB
                if let Some(gain_val) = node.params.get("gain_db").and_then(|g| g.as_f64()) {
                    let gain_db = gain_val as f32;
                    for segment_pcm in &mut ctx.audio_segments {
                        AudioNormalizer::apply_gain_db(segment_pcm, gain_db);
                    }
                }

                // 3. Silence trimmer
                if let Some(trim_obj) = node.params.get("silence_trim") {
                    let threshold = trim_obj
                        .get("threshold_dbfs")
                        .and_then(|t| t.as_f64())
                        .unwrap_or(-45.0) as f32;
                    let padding = trim_obj
                        .get("padding_ms")
                        .and_then(|p| p.as_u64())
                        .unwrap_or(30) as u32;

                    for segment_pcm in &mut ctx.audio_segments {
                        *segment_pcm =
                            SilenceTrimmer::trim(segment_pcm, ctx.sample_rate, threshold, padding);
                    }
                } else if node
                    .params
                    .get("trim_silence")
                    .and_then(|b| b.as_bool())
                    .unwrap_or(false)
                {
                    for segment_pcm in &mut ctx.audio_segments {
                        *segment_pcm =
                            SilenceTrimmer::trim(segment_pcm, ctx.sample_rate, -45.0, 30);
                    }
                }

                // 4. 3-Band Parametric Equalizer
                if let Some(eq_obj) = node.params.get("eq") {
                    let low = eq_obj
                        .get("low_gain_db")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32;
                    let mid = eq_obj
                        .get("mid_gain_db")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32;
                    let high = eq_obj
                        .get("high_gain_db")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32;

                    for segment_pcm in &mut ctx.audio_segments {
                        ParametricEq::process_3band(segment_pcm, ctx.sample_rate, low, mid, high);
                    }
                }

                // 5. Dynamic Range Compressor
                if let Some(comp_obj) = node.params.get("compressor") {
                    let threshold = comp_obj
                        .get("threshold_dbfs")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(-18.0) as f32;
                    let ratio = comp_obj
                        .get("ratio")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(3.0) as f32;
                    let attack = comp_obj
                        .get("attack_ms")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(15.0) as f32;
                    let release = comp_obj
                        .get("release_ms")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(100.0) as f32;
                    let makeup = comp_obj
                        .get("makeup_gain_db")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32;

                    for segment_pcm in &mut ctx.audio_segments {
                        DynamicCompressor::process(
                            segment_pcm,
                            ctx.sample_rate,
                            threshold,
                            ratio,
                            attack,
                            release,
                            makeup,
                        );
                    }
                }

                // 6. Brickwall Limiter
                if let Some(limiter_obj) = node.params.get("limiter") {
                    let ceiling = limiter_obj
                        .get("ceiling_dbfs")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(-0.5) as f32;
                    for segment_pcm in &mut ctx.audio_segments {
                        BrickwallLimiter::process(segment_pcm, ceiling);
                    }
                }

                Ok(())
            }

            NodeType::AudioMerge => {
                let pause_ms = node
                    .params
                    .get("pause_ms")
                    .and_then(|p| p.as_u64())
                    .unwrap_or(150) as u32;

                let refs: Vec<&[i16]> = ctx.audio_segments.iter().map(|s| s.as_slice()).collect();
                let merged = AudioMerger::concatenate_with_pause(&refs, ctx.sample_rate, pause_ms);
                ctx.master_audio_pcm = merged;
                Ok(())
            }

            NodeType::OutputSink => {
                // If master audio not merged yet, merge with default 100ms pause
                if ctx.master_audio_pcm.is_empty() && !ctx.audio_segments.is_empty() {
                    let refs: Vec<&[i16]> =
                        ctx.audio_segments.iter().map(|s| s.as_slice()).collect();
                    ctx.master_audio_pcm =
                        AudioMerger::concatenate_with_pause(&refs, ctx.sample_rate, 100);
                }
                Ok(())
            }

            NodeType::AsrTranscriber => {
                let model_id = node.params.get("model").and_then(|m| m.as_str());
                let engine = if let Some(id) = model_id {
                    self.asr_registry.get(id).await
                } else {
                    self.asr_registry.default_engine().await
                }
                .ok_or_else(|| VoxForgError::PipelineExecution {
                    node_id: node.id.clone(),
                    reason: "No ASR engine found in registry".to_string(),
                })?;

                let sample_rate = if ctx.sample_rate == 0 {
                    16000
                } else {
                    ctx.sample_rate
                };
                let pcm = if !ctx.input_audio_pcm.is_empty() {
                    ctx.input_audio_pcm.clone()
                } else if !ctx.master_audio_pcm.is_empty() {
                    ctx.master_audio_pcm.clone()
                } else {
                    let sample_count = (sample_rate as usize) * 2;
                    (0..sample_count)
                        .map(|i| ((i % 100) as i16 - 50) * 100)
                        .collect()
                };

                let opts = voxforg_asr::TranscriptionOptions {
                    language: node
                        .params
                        .get("language")
                        .and_then(|l| l.as_str())
                        .map(|s| s.to_string()),
                    temperature: node
                        .params
                        .get("temperature")
                        .and_then(|t| t.as_f64())
                        .map(|f| f as f32),
                    prompt: ctx.raw_text.clone(),
                    word_timestamps: node
                        .params
                        .get("word_timestamps")
                        .and_then(|w| w.as_bool())
                        .unwrap_or(true),
                    response_format: "verbose_json".to_string(),
                };

                let res = engine
                    .transcribe(&pcm, sample_rate, &opts)
                    .await
                    .map_err(|e| VoxForgError::PipelineExecution {
                        node_id: node.id.clone(),
                        reason: format!("ASR transcription failed: {e}"),
                    })?;

                let mut segments = Vec::new();
                for seg in res.segments {
                    let duration_ms = seg.end_ms.saturating_sub(seg.start_ms).max(100);
                    segments.push(ScriptSegment {
                        speaker: seg.speaker.unwrap_or_else(|| "Narrator".to_string()),
                        text: seg.text,
                        voice_id: None,
                        speed: None,
                        pitch: None,
                        target_duration_ms: Some(duration_ms),
                    });
                }

                if !segments.is_empty() {
                    ctx.segments = segments;
                }
                Ok(())
            }

            NodeType::Diarization => {
                let num_speakers = node
                    .params
                    .get("num_speakers")
                    .and_then(|n| n.as_u64())
                    .unwrap_or(2) as usize;
                let default_speaker = node
                    .params
                    .get("default_speaker")
                    .and_then(|s| s.as_str())
                    .unwrap_or("Narrator");

                for (i, seg) in ctx.segments.iter_mut().enumerate() {
                    if seg.speaker.is_empty()
                        || seg.speaker == "Narrator"
                        || seg.speaker.starts_with("SPEAKER_")
                    {
                        seg.speaker = format!("SPEAKER_{:02}", i % num_speakers);
                    }
                }

                if ctx.segments.is_empty() && ctx.raw_text.is_some() {
                    ctx.segments.push(ScriptSegment {
                        speaker: default_speaker.to_string(),
                        text: ctx.raw_text.clone().unwrap(),
                        voice_id: None,
                        speed: None,
                        pitch: None,
                        target_duration_ms: None,
                    });
                }
                Ok(())
            }

            NodeType::DocumentChunker => {
                let text = ctx.raw_text.as_deref().unwrap_or_default();
                let default_speaker = node
                    .params
                    .get("default_speaker")
                    .and_then(|s| s.as_str())
                    .unwrap_or("Narrator");
                let mut segments = Vec::new();

                for line in text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
                    if line.starts_with('"') || line.starts_with('“') || line.contains(':') {
                        if let Some((spk, utt)) = line.split_once(':') {
                            segments.push(ScriptSegment {
                                speaker: spk.trim().to_string(),
                                text: utt
                                    .trim()
                                    .trim_matches('"')
                                    .trim_matches('“')
                                    .trim_matches('”')
                                    .to_string(),
                                voice_id: None,
                                speed: None,
                                pitch: None,
                                target_duration_ms: None,
                            });
                        } else {
                            segments.push(ScriptSegment {
                                speaker: "Character".to_string(),
                                text: line
                                    .trim_matches('"')
                                    .trim_matches('“')
                                    .trim_matches('”')
                                    .to_string(),
                                voice_id: None,
                                speed: None,
                                pitch: None,
                                target_duration_ms: None,
                            });
                        }
                    } else {
                        segments.push(ScriptSegment {
                            speaker: default_speaker.to_string(),
                            text: line.to_string(),
                            voice_id: None,
                            speed: None,
                            pitch: None,
                            target_duration_ms: None,
                        });
                    }
                }

                if segments.is_empty() && !text.is_empty() {
                    segments.push(ScriptSegment {
                        speaker: default_speaker.to_string(),
                        text: text.to_string(),
                        voice_id: None,
                        speed: None,
                        pitch: None,
                        target_duration_ms: None,
                    });
                }
                ctx.segments = segments;
                Ok(())
            }

            NodeType::AudioTimeStretch => {
                for (i, pcm) in ctx.audio_segments.iter_mut().enumerate() {
                    if let Some(seg) = ctx.segments.get(i) {
                        if let Some(target_ms) = seg.target_duration_ms {
                            let current_samples = pcm.len();
                            let target_samples =
                                (target_ms as f64 / 1000.0 * ctx.sample_rate as f64) as usize;
                            if target_samples > 0
                                && current_samples > 0
                                && target_samples != current_samples
                            {
                                *pcm = time_stretch_linear(pcm, target_samples);
                            }
                        }
                    }
                }
                Ok(())
            }

            NodeType::AudioMux => {
                let voice_vol = node
                    .params
                    .get("voice_volume")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0) as f32;
                let bg_vol = node
                    .params
                    .get("background_volume")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.15) as f32;
                let ducking = node
                    .params
                    .get("ducking")
                    .and_then(|d| d.as_bool())
                    .unwrap_or(true);

                if ctx.master_audio_pcm.is_empty() && !ctx.audio_segments.is_empty() {
                    let refs: Vec<&[i16]> =
                        ctx.audio_segments.iter().map(|s| s.as_slice()).collect();
                    ctx.master_audio_pcm =
                        AudioMerger::concatenate_with_pause(&refs, ctx.sample_rate, 100);
                }

                // Check if a background audio file is provided
                if ctx.background_audio_pcm.is_empty() {
                    if let Some(bg_path) = node
                        .params
                        .get("background_audio_path")
                        .and_then(|p| p.as_str())
                    {
                        if let Ok(bytes) = std::fs::read(bg_path) {
                            if bytes.starts_with(b"RIFF") {
                                if let Ok((pcm, _sr, _ch)) =
                                    voxforg_audio::WavEncoder::decode_wav_to_pcm16(&bytes)
                                {
                                    ctx.background_audio_pcm = pcm;
                                }
                            }
                        }
                    }
                }

                // If no background audio track is present, cleanly scale master audio without injecting synthetic noise
                if ctx.background_audio_pcm.is_empty() {
                    for sample in &mut ctx.master_audio_pcm {
                        *sample = (*sample as f32 * voice_vol)
                            .clamp(i16::MIN as f32, i16::MAX as f32)
                            as i16;
                    }
                    return Ok(());
                }

                let max_len = ctx
                    .master_audio_pcm
                    .len()
                    .max(ctx.background_audio_pcm.len());
                let mut mixed = Vec::with_capacity(max_len);

                for i in 0..max_len {
                    let v = if i < ctx.master_audio_pcm.len() {
                        ctx.master_audio_pcm[i] as f32 * voice_vol
                    } else {
                        0.0
                    };

                    let effective_bg_vol = if ducking && v.abs() > 300.0 {
                        bg_vol * 0.3
                    } else {
                        bg_vol
                    };

                    let bg = if i < ctx.background_audio_pcm.len() {
                        ctx.background_audio_pcm[i] as f32 * effective_bg_vol
                    } else {
                        0.0
                    };

                    let sample = (v + bg).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                    mixed.push(sample);
                }

                ctx.master_audio_pcm = mixed;
                Ok(())
            }
        }
    }
}

fn time_stretch_linear(input: &[i16], target_len: usize) -> Vec<i16> {
    if input.is_empty() || target_len == 0 {
        return Vec::new();
    }
    if input.len() == target_len {
        return input.to_vec();
    }
    let mut output = Vec::with_capacity(target_len);
    let scale = (input.len() - 1) as f64 / (target_len - 1).max(1) as f64;
    for i in 0..target_len {
        let src_pos = i as f64 * scale;
        let idx0 = src_pos.floor() as usize;
        let idx1 = (idx0 + 1).min(input.len() - 1);
        let frac = (src_pos - idx0 as f64) as f32;
        let s0 = input[idx0] as f32;
        let s1 = input[idx1] as f32;
        let sample = s0 + frac * (s1 - s0);
        output.push(sample.clamp(i16::MIN as f32, i16::MAX as f32) as i16);
    }
    output
}
