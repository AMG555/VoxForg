use std::sync::Arc;
use voxforg_audio::{
    AudioMerger, AudioNormalizer, BrickwallLimiter, DynamicCompressor, ParametricEq, SilenceTrimmer,
    WavEncoder,
};
use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::{NodeType, PipelineDefinition};
use voxforg_engine::{EngineRegistry, SynthesisRequest};

use crate::context::{ExecutionContext, ScriptSegment};
use crate::graph::GraphValidator;

pub struct PipelineExecutor {
    engine_registry: Arc<EngineRegistry>,
}

impl PipelineExecutor {
    pub fn new(engine_registry: Arc<EngineRegistry>) -> Self {
        Self { engine_registry }
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
                .ok_or_else(|| {
                    VoxForgError::PipelineExecution {
                        node_id: node_id.clone(),
                        reason: "Node missing in definition".to_string(),
                    }
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
                            reason: "TextInput exceeds maximum supported length of 50,000 characters".to_string(),
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
                let lines: Vec<&str> = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
                for line in lines {
                    if let Some((speaker, utterance)) = line.split_once(':') {
                        segments.push(ScriptSegment {
                            speaker: speaker.trim().to_string(),
                            text: utterance.trim().to_string(),
                            voice_id: None,
                            speed: None,
                            pitch: None,
                        });
                    } else {
                        segments.push(ScriptSegment {
                            speaker: "Narrator".to_string(),
                            text: line.to_string(),
                            voice_id: None,
                            speed: None,
                            pitch: None,
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
                    let voice_id = segment
                        .voice_id
                        .as_deref()
                        .unwrap_or("en-US-AriaNeural");

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
                } else if node.params.get("filter_type").and_then(|t| t.as_str()) == Some("normalize")
                    || node.params.is_null()
                    || node.params.as_object().map_or(true, |o| o.is_empty())
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
                        *segment_pcm = SilenceTrimmer::trim(segment_pcm, ctx.sample_rate, threshold, padding);
                    }
                } else if node.params.get("trim_silence").and_then(|b| b.as_bool()).unwrap_or(false) {
                    for segment_pcm in &mut ctx.audio_segments {
                        *segment_pcm = SilenceTrimmer::trim(segment_pcm, ctx.sample_rate, -45.0, 30);
                    }
                }

                // 4. 3-Band Parametric Equalizer
                if let Some(eq_obj) = node.params.get("eq") {
                    let low = eq_obj.get("low_gain_db").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let mid = eq_obj.get("mid_gain_db").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let high = eq_obj.get("high_gain_db").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

                    for segment_pcm in &mut ctx.audio_segments {
                        ParametricEq::process_3band(segment_pcm, ctx.sample_rate, low, mid, high);
                    }
                }

                // 5. Dynamic Range Compressor
                if let Some(comp_obj) = node.params.get("compressor") {
                    let threshold = comp_obj.get("threshold_dbfs").and_then(|v| v.as_f64()).unwrap_or(-18.0) as f32;
                    let ratio = comp_obj.get("ratio").and_then(|v| v.as_f64()).unwrap_or(3.0) as f32;
                    let attack = comp_obj.get("attack_ms").and_then(|v| v.as_f64()).unwrap_or(15.0) as f32;
                    let release = comp_obj.get("release_ms").and_then(|v| v.as_f64()).unwrap_or(100.0) as f32;
                    let makeup = comp_obj.get("makeup_gain_db").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

                    for segment_pcm in &mut ctx.audio_segments {
                        DynamicCompressor::process(segment_pcm, ctx.sample_rate, threshold, ratio, attack, release, makeup);
                    }
                }

                // 6. Brickwall Limiter
                if let Some(limiter_obj) = node.params.get("limiter") {
                    let ceiling = limiter_obj.get("ceiling_dbfs").and_then(|v| v.as_f64()).unwrap_or(-0.5) as f32;
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
                    let refs: Vec<&[i16]> = ctx.audio_segments.iter().map(|s| s.as_slice()).collect();
                    ctx.master_audio_pcm = AudioMerger::concatenate_with_pause(&refs, ctx.sample_rate, 100);
                }
                Ok(())
            }
        }
    }
}
