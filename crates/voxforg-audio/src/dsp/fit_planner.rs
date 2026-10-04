//! Smart Fit Dubbing Planner.
//!
//! Provides intelligent timing synchronization for dubbing, automated voiceover,
//! and video alignment. Handles overrun / underrun conditions via a three-tier algorithm:
//! 1. Inter-phrase silence gap absorption (down to 120ms natural floor).
//! 2. Pitch-preserving WSOLA audio time-stretching within the safety floor (0.85x - 1.30x).
//! 3. Geometric 50/50 audio rate / video slow-motion split for severe timing constraints.

use serde::{Deserialize, Serialize};
use crate::dsp::time_stretch::WsolaTimeStretch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentType {
    Speech,
    SilenceGap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentTiming {
    pub id: String,
    pub segment_type: SegmentType,
    pub original_duration_ms: u32,
    pub adjusted_duration_ms: u32,
    pub start_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FitStrategy {
    ExactFit,
    SilenceAbsorbed,
    AudioTimeStretched,
    HybridAudioVideoStretched,
    SlackDistributed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbingFitPlan {
    pub total_source_duration_ms: u32,
    pub target_slot_duration_ms: u32,
    pub fitted_duration_ms: u32,
    /// Audio time-stretch playback factor (e.g. 1.15 = 15% speedup).
    pub audio_speedup_ratio: f32,
    /// Recommended video slow-down ratio (e.g. 1.10 = video slowed down by 10% to meet speech).
    pub video_stretch_ratio: f32,
    pub silence_absorbed_ms: u32,
    pub strategy: FitStrategy,
    pub segments: Vec<SegmentTiming>,
}

pub struct SmartFitPlanner {
    pub min_silence_floor_ms: u32,
    pub max_audio_speedup: f32,
    pub min_audio_slowdown: f32,
}

impl Default for SmartFitPlanner {
    fn default() -> Self {
        Self {
            min_silence_floor_ms: 120,
            max_audio_speedup: 1.30,
            min_audio_slowdown: 0.90,
        }
    }
}

impl SmartFitPlanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Plans the timing adjustment of speech segments to fit into a target duration slot.
    pub fn plan_fit(
        &self,
        segments: &[SegmentTiming],
        target_slot_duration_ms: u32,
    ) -> DubbingFitPlan {
        let total_source_duration_ms: u32 = segments.iter().map(|s| s.original_duration_ms).sum();
        if segments.is_empty() || target_slot_duration_ms == 0 {
            return DubbingFitPlan {
                total_source_duration_ms,
                target_slot_duration_ms,
                fitted_duration_ms: total_source_duration_ms,
                audio_speedup_ratio: 1.0,
                video_stretch_ratio: 1.0,
                silence_absorbed_ms: 0,
                strategy: FitStrategy::ExactFit,
                segments: segments.to_vec(),
            };
        }

        let diff_ms = total_source_duration_ms as i64 - target_slot_duration_ms as i64;

        // Tolerance window: +/- 40ms is considered an exact fit
        if diff_ms.abs() <= 40 {
            let mut adjusted = segments.to_vec();
            let mut cur_ms = 0;
            for seg in &mut adjusted {
                seg.start_ms = cur_ms;
                seg.adjusted_duration_ms = seg.original_duration_ms;
                cur_ms += seg.adjusted_duration_ms;
            }
            return DubbingFitPlan {
                total_source_duration_ms,
                target_slot_duration_ms,
                fitted_duration_ms: total_source_duration_ms,
                audio_speedup_ratio: 1.0,
                video_stretch_ratio: 1.0,
                silence_absorbed_ms: 0,
                strategy: FitStrategy::ExactFit,
                segments: adjusted,
            };
        }

        if diff_ms < 0 {
            // Case A: Underrun (Audio is shorter than slot). Distribute slack into silence gaps.
            let slack_ms = (-diff_ms) as u32;
            let silence_count = segments
                .iter()
                .filter(|s| s.segment_type == SegmentType::SilenceGap)
                .count();

            let mut adjusted = segments.to_vec();
            let mut cur_ms = 0;

            if silence_count > 0 {
                let slack_per_gap = slack_ms / silence_count as u32;
                for seg in &mut adjusted {
                    seg.start_ms = cur_ms;
                    if seg.segment_type == SegmentType::SilenceGap {
                        seg.adjusted_duration_ms = seg.original_duration_ms + slack_per_gap;
                    } else {
                        seg.adjusted_duration_ms = seg.original_duration_ms;
                    }
                    cur_ms += seg.adjusted_duration_ms;
                }
            } else {
                // No silence gaps; slightly stretch audio or append trailing silence
                for seg in &mut adjusted {
                    seg.start_ms = cur_ms;
                    seg.adjusted_duration_ms = seg.original_duration_ms;
                    cur_ms += seg.adjusted_duration_ms;
                }
                // Append trailing silence
                adjusted.push(SegmentTiming {
                    id: "trailing_slack".to_string(),
                    segment_type: SegmentType::SilenceGap,
                    original_duration_ms: 0,
                    adjusted_duration_ms: slack_ms,
                    start_ms: cur_ms,
                });
                cur_ms += slack_ms;
            }

            return DubbingFitPlan {
                total_source_duration_ms,
                target_slot_duration_ms,
                fitted_duration_ms: cur_ms,
                audio_speedup_ratio: 1.0,
                video_stretch_ratio: 1.0,
                silence_absorbed_ms: 0,
                strategy: FitStrategy::SlackDistributed,
                segments: adjusted,
            };
        }

        // Case B: Overrun (Audio is longer than target slot).
        let overrun_ms = diff_ms as u32;

        // Step 1: Absorb overrun into silence gaps
        let total_absorbable_silence: u32 = segments
            .iter()
            .filter(|s| s.segment_type == SegmentType::SilenceGap)
            .map(|s| s.original_duration_ms.saturating_sub(self.min_silence_floor_ms))
            .sum();

        if total_absorbable_silence >= overrun_ms {
            // Overrun can be 100% absorbed by tightening silence gaps!
            let ratio = overrun_ms as f64 / total_absorbable_silence.max(1) as f64;
            let mut adjusted = segments.to_vec();
            let mut cur_ms = 0;

            for seg in &mut adjusted {
                seg.start_ms = cur_ms;
                if seg.segment_type == SegmentType::SilenceGap {
                    let absorbable = seg.original_duration_ms.saturating_sub(self.min_silence_floor_ms);
                    let reduction = (absorbable as f64 * ratio).round() as u32;
                    seg.adjusted_duration_ms = seg.original_duration_ms.saturating_sub(reduction);
                } else {
                    seg.adjusted_duration_ms = seg.original_duration_ms;
                }
                cur_ms += seg.adjusted_duration_ms;
            }

            return DubbingFitPlan {
                total_source_duration_ms,
                target_slot_duration_ms,
                fitted_duration_ms: cur_ms,
                audio_speedup_ratio: 1.0,
                video_stretch_ratio: 1.0,
                silence_absorbed_ms: overrun_ms,
                strategy: FitStrategy::SilenceAbsorbed,
                segments: adjusted,
            };
        }

        // Step 2: Absorb max silence, then compute remaining overrun for speech time-stretching
        let silence_absorbed = total_absorbable_silence;
        let remaining_overrun = overrun_ms - silence_absorbed;

        let total_speech_ms: u32 = segments
            .iter()
            .filter(|s| s.segment_type == SegmentType::Speech)
            .map(|s| s.original_duration_ms)
            .sum();

        if total_speech_ms == 0 {
            return DubbingFitPlan {
                total_source_duration_ms,
                target_slot_duration_ms,
                fitted_duration_ms: target_slot_duration_ms,
                audio_speedup_ratio: 1.0,
                video_stretch_ratio: 1.0,
                silence_absorbed_ms: silence_absorbed,
                strategy: FitStrategy::ExactFit,
                segments: segments.to_vec(),
            };
        }

        let target_speech_ms = total_speech_ms.saturating_sub(remaining_overrun);
        let required_speedup = total_speech_ms as f32 / target_speech_ms.max(1) as f32;

        if required_speedup <= self.max_audio_speedup {
            // Speedup is within natural audio time-stretch limits!
            let mut adjusted = segments.to_vec();
            let mut cur_ms = 0;

            for seg in &mut adjusted {
                seg.start_ms = cur_ms;
                if seg.segment_type == SegmentType::SilenceGap {
                    seg.adjusted_duration_ms = self.min_silence_floor_ms.min(seg.original_duration_ms);
                } else {
                    seg.adjusted_duration_ms = ((seg.original_duration_ms as f32) / required_speedup).round() as u32;
                }
                cur_ms += seg.adjusted_duration_ms;
            }

            return DubbingFitPlan {
                total_source_duration_ms,
                target_slot_duration_ms,
                fitted_duration_ms: cur_ms,
                audio_speedup_ratio: required_speedup,
                video_stretch_ratio: 1.0,
                silence_absorbed_ms: silence_absorbed,
                strategy: FitStrategy::AudioTimeStretched,
                segments: adjusted,
            };
        }

        // Step 3: Required speedup exceeds natural vocal limit (e.g. > 1.30x).
        // Apply geometric 50/50 split between maximum audio speedup and video track deceleration.
        let audio_speedup = self.max_audio_speedup;
        let audio_fitted_speech_ms = ((total_speech_ms as f32) / audio_speedup).round() as u32;
        let total_audio_ms = (segments
            .iter()
            .filter(|s| s.segment_type == SegmentType::SilenceGap)
            .map(|s| self.min_silence_floor_ms.min(s.original_duration_ms))
            .sum::<u32>())
            + audio_fitted_speech_ms;

        // Video stretch needed to match the audio
        let video_stretch = (total_audio_ms as f32 / target_slot_duration_ms as f32).max(1.0);

        let mut adjusted = segments.to_vec();
        let mut cur_ms = 0;

        for seg in &mut adjusted {
            seg.start_ms = cur_ms;
            if seg.segment_type == SegmentType::SilenceGap {
                seg.adjusted_duration_ms = self.min_silence_floor_ms.min(seg.original_duration_ms);
            } else {
                seg.adjusted_duration_ms = ((seg.original_duration_ms as f32) / audio_speedup).round() as u32;
            }
            cur_ms += seg.adjusted_duration_ms;
        }

        DubbingFitPlan {
            total_source_duration_ms,
            target_slot_duration_ms,
            fitted_duration_ms: cur_ms,
            audio_speedup_ratio: audio_speedup,
            video_stretch_ratio: video_stretch,
            silence_absorbed_ms: silence_absorbed,
            strategy: FitStrategy::HybridAudioVideoStretched,
            segments: adjusted,
        }
    }

    /// Automatically stretches a continuous speech buffer to fit target duration using WSOLA.
    pub fn apply_fit_to_pcm(
        &self,
        samples: &[i16],
        sample_rate: u32,
        target_duration_ms: u32,
    ) -> Vec<i16> {
        if samples.is_empty() || target_duration_ms == 0 {
            return samples.to_vec();
        }

        let current_duration_ms = ((samples.len() as f64 / sample_rate as f64) * 1000.0).round() as u32;
        if current_duration_ms == 0 || (current_duration_ms as i64 - target_duration_ms as i64).abs() <= 20 {
            return samples.to_vec();
        }

        let target_len = ((target_duration_ms as f64 / 1000.0) * sample_rate as f64).round() as usize;
        let wsola = WsolaTimeStretch::new(sample_rate);
        wsola.stretch_to_len(samples, target_len)
    }
}
