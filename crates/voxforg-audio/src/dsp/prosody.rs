//! Prosody Mirroring & Acoustic Feature Analysis.
//!
//! Extracts speaker-relative acoustic z-scores (F0 pitch, RMS dynamics, cadence, voicing ratio)
//! and provides bidirectional mapping with the 5-axis Director AI taxonomy
//! (energy, emotion, pace, intimacy, formality).

use serde::{Deserialize, Serialize};

/// 5-Axis Director AI taxonomic vocal controls (-1.0 to 1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DirectorTaxonomy {
    /// Energy / intensity: -1.0 (subdued, exhausted) to 1.0 (explosive, shouting).
    #[serde(default)]
    pub energy: f32,
    /// Emotion valence / coloration: -1.0 (melancholic/somber) to 1.0 (euphoric/cheerful).
    #[serde(default)]
    pub emotion: f32,
    /// Tempo / cadence: -1.0 (deliberate, slow) to 1.0 (rapid, urgent).
    #[serde(default)]
    pub pace: f32,
    /// Intimacy / proximity: -1.0 (broadcast public projection) to 1.0 (whispered in-ear intimacy).
    #[serde(default)]
    pub intimacy: f32,
    /// Formality / articulation: -1.0 (casual, slurred colloquial) to 1.0 (precise, classical broadcast).
    #[serde(default)]
    pub formality: f32,
}

impl Default for DirectorTaxonomy {
    fn default() -> Self {
        Self {
            energy: 0.0,
            emotion: 0.0,
            pace: 0.0,
            intimacy: 0.0,
            formality: 0.0,
        }
    }
}

/// Extracted acoustic feature profile of an audio recording.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcousticFeatures {
    /// Estimated fundamental pitch mean in Hertz.
    pub f0_mean_hz: f32,
    /// Pitch variance in semitones (intonation contour dynamics).
    pub f0_std_semitones: f32,
    /// Overall Root Mean Square loudness in dBFS.
    pub rms_dbfs: f32,
    /// Estimated syllable cadence in syllables per second.
    pub syllable_rate: f32,
    /// Ratio of voiced speech frames to total speech frames (0.0 to 1.0).
    pub voicing_ratio: f32,
    /// Relative z-score of loudness compared to standard conversational norm.
    pub loudness_z_score: f32,
    /// Relative z-score of speech tempo.
    pub pace_z_score: f32,
    /// Measured vocal fold pitch micro-jitter percentage (cycle-to-cycle instability).
    #[serde(default = "default_jitter")]
    pub f0_jitter_percent: f32,
    /// Spectral tilt / slope balance in dB.
    #[serde(default)]
    pub spectral_tilt_db: f32,
    /// Estimated throat/chest formant frequency F1 in Hz.
    #[serde(default = "default_f1")]
    pub formant_f1_hz: f32,
    /// Estimated oral formant frequency F2 in Hz.
    #[serde(default = "default_f2")]
    pub formant_f2_hz: f32,
}

fn default_jitter() -> f32 {
    0.6
}
fn default_f1() -> f32 {
    500.0
}
fn default_f2() -> f32 {
    1500.0
}

pub struct ProsodyMirror;

impl ProsodyMirror {
    /// Extracts acoustic features and z-scores from 16-bit PCM speech.
    pub fn analyze(samples: &[i16], sample_rate: u32) -> AcousticFeatures {
        if samples.is_empty() || sample_rate == 0 {
            return AcousticFeatures {
                f0_mean_hz: 140.0,
                f0_std_semitones: 0.0,
                rms_dbfs: -45.0,
                syllable_rate: 3.5,
                voicing_ratio: 0.0,
                loudness_z_score: 0.0,
                pace_z_score: 0.0,
                f0_jitter_percent: 0.6,
                spectral_tilt_db: 0.0,
                formant_f1_hz: 500.0,
                formant_f2_hz: 1500.0,
            };
        }

        let sr = sample_rate as f32;
        let frame_size = (sr * 0.030) as usize; // 30ms window
        let hop_size = (sr * 0.015) as usize; // 15ms hop

        let mut rms_sum = 0.0f64;
        for &s in samples {
            let norm = s as f64 / 32768.0;
            rms_sum += norm * norm;
        }
        let total_rms = (rms_sum / samples.len() as f64).sqrt().max(1e-6);
        let rms_dbfs = (20.0 * total_rms.log10()) as f32;

        // Autocorrelation pitch extraction (F0) across frames
        let min_lag = (sr / 450.0).round() as usize; // Max F0 ~ 450Hz
        let max_lag = (sr / 65.0).round() as usize; // Min F0 ~ 65Hz

        let mut f0_estimates = Vec::new();
        let mut voiced_frames = 0;
        let mut total_frames = 0;

        let mut offset = 0;
        while offset + frame_size <= samples.len() {
            total_frames += 1;
            let frame = &samples[offset..offset + frame_size];

            // Frame energy
            let mut frame_energy = 0.0f64;
            for &s in frame {
                let v = s as f64;
                frame_energy += v * v;
            }

            if frame_energy > 1e7 {
                // Speech threshold
                // Normalized autocorrelation
                let mut best_lag = 0;
                let mut best_r = 0.0f64;

                for lag in min_lag..=max_lag.min(frame_size / 2) {
                    let mut r = 0.0f64;
                    for i in 0..(frame_size - lag) {
                        r += frame[i] as f64 * frame[i + lag] as f64;
                    }
                    if r > best_r {
                        best_r = r;
                        best_lag = lag;
                    }
                }

                let norm_r = best_r / frame_energy;
                if norm_r > 0.35 && best_lag > 0 {
                    let pitch_hz = sr / best_lag as f32;
                    f0_estimates.push(pitch_hz);
                    voiced_frames += 1;
                }
            }

            offset += hop_size;
        }

        let voicing_ratio = if total_frames > 0 {
            voiced_frames as f32 / total_frames as f32
        } else {
            0.0
        };

        let f0_mean_hz = if !f0_estimates.is_empty() {
            f0_estimates.iter().sum::<f32>() / f0_estimates.len() as f32
        } else {
            140.0
        };

        let f0_std_semitones = if f0_estimates.len() > 1 {
            let mean = f0_mean_hz;
            let variance = f0_estimates
                .iter()
                .map(|&f| {
                    let semitones = 12.0 * (f / mean.max(1.0)).log2();
                    semitones * semitones
                })
                .sum::<f32>()
                / f0_estimates.len() as f32;
            variance.sqrt()
        } else {
            1.5
        };

        // Estimate syllable rate via spectral peak dynamics
        let duration_sec = samples.len() as f32 / sr;
        let estimated_syllables = (voiced_frames as f32 * 0.4).max(1.0);
        let syllable_rate = (estimated_syllables / duration_sec.max(0.5)).clamp(1.5, 7.5);

        // Conversational baseline z-score mappings
        // Mean conversational RMS: -20 dBFS, std: 5 dB
        let loudness_z_score = ((rms_dbfs - (-20.0)) / 5.0).clamp(-3.0, 3.0);
        // Mean conversational syllable rate: 4.2 syll/sec, std: 0.8 syll/sec
        let pace_z_score = ((syllable_rate - 4.2) / 0.8).clamp(-3.0, 3.0);

        // Vocal fold micro-jitter calculation (cycle-to-cycle F0 perturbation)
        let f0_jitter_percent = if f0_estimates.len() > 2 {
            let mut diff_sum = 0.0f32;
            for i in 1..f0_estimates.len() {
                diff_sum += (f0_estimates[i] - f0_estimates[i - 1]).abs();
            }
            let mean_diff = diff_sum / (f0_estimates.len() - 1) as f32;
            ((mean_diff / f0_mean_hz.max(50.0)) * 100.0).clamp(0.1, 4.0)
        } else {
            0.6
        };

        // Formant resonance & spectral tilt estimation
        let (formant_f1_hz, formant_f2_hz, spectral_tilt_db) = if samples.len() >= 256 {
            let mut low_energy = 0.0f64;
            let mut mid_energy = 0.0f64;
            let mut high_energy = 0.0f64;

            let n = samples.len().min(48000);
            for i in 2..n {
                let s = samples[i] as f64;
                let s_prev = samples[i - 1] as f64;
                let s_prev2 = samples[i - 2] as f64;

                let low = (s + 2.0 * s_prev + s_prev2) * 0.25;
                let high = s - s_prev;
                let mid = s - low;

                low_energy += low * low;
                mid_energy += mid * mid;
                high_energy += high * high;
            }

            let tilt =
                (10.0 * (low_energy / high_energy.max(1.0)).log10()).clamp(-12.0, 12.0) as f32;

            let f1 = if f0_mean_hz < 165.0 {
                (380.0 + (mid_energy / low_energy.max(1.0) * 150.0).min(200.0)) as f32
            } else {
                (520.0 + (mid_energy / low_energy.max(1.0) * 180.0).min(250.0)) as f32
            };

            let f2 = if f0_mean_hz < 165.0 {
                (1400.0 + (high_energy / mid_energy.max(1.0) * 300.0).min(500.0)) as f32
            } else {
                (1750.0 + (high_energy / mid_energy.max(1.0) * 350.0).min(600.0)) as f32
            };

            (f1, f2, tilt)
        } else {
            (500.0, 1500.0, 0.0)
        };

        AcousticFeatures {
            f0_mean_hz,
            f0_std_semitones,
            rms_dbfs,
            syllable_rate,
            voicing_ratio,
            loudness_z_score,
            pace_z_score,
            f0_jitter_percent,
            spectral_tilt_db,
            formant_f1_hz,
            formant_f2_hz,
        }
    }

    /// Derives 5-axis Director AI taxonomy from observed acoustic features.
    pub fn features_to_director(features: &AcousticFeatures) -> DirectorTaxonomy {
        let energy = (features.loudness_z_score * 0.4 + (features.f0_std_semitones - 2.0) * 0.3)
            .clamp(-1.0, 1.0);
        let pace = (features.pace_z_score * 0.5).clamp(-1.0, 1.0);
        let intimacy = (-features.loudness_z_score * 0.5 + (1.0 - features.voicing_ratio) * 0.3)
            .clamp(-1.0, 1.0);
        let emotion = (features.f0_std_semitones * 0.3 - 0.5).clamp(-1.0, 1.0);
        let formality = (features.voicing_ratio * 0.5 - 0.2).clamp(-1.0, 1.0);

        DirectorTaxonomy {
            energy,
            emotion,
            pace,
            intimacy,
            formality,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prosody_mirror_empty_or_zero_sr() {
        let features = ProsodyMirror::analyze(&[], 24000);
        assert_eq!(features.f0_mean_hz, 140.0);
        assert_eq!(features.voicing_ratio, 0.0);
        let director = ProsodyMirror::features_to_director(&features);
        assert!(director.energy.is_finite());
        assert!(director.pace.is_finite());

        let features_zero_sr = ProsodyMirror::analyze(&[100, 200, 300], 0);
        assert_eq!(features_zero_sr.voicing_ratio, 0.0);
    }

    #[test]
    fn test_prosody_mirror_synthetic_speech_tone() {
        let sample_rate = 24000;
        let mut samples = Vec::with_capacity(sample_rate as usize);
        // Synthesize 1s of 220Hz tone at strong amplitude
        for i in 0..sample_rate {
            let t = i as f32 / sample_rate as f32;
            let s = ((2.0 * std::f32::consts::PI * 220.0 * t).sin() * 15000.0) as i16;
            samples.push(s);
        }

        let features = ProsodyMirror::analyze(&samples, sample_rate);
        assert!(features.rms_dbfs > -30.0);
        assert!(features.f0_mean_hz > 200.0 && features.f0_mean_hz < 240.0);
        assert!(features.voicing_ratio > 0.5);

        let director = ProsodyMirror::features_to_director(&features);
        assert!(director.energy >= -1.0 && director.energy <= 1.0);
        assert!(director.formality >= -1.0 && director.formality <= 1.0);
    }
}
