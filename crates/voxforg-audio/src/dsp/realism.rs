//! Psychoacoustic Realism & Human Biometric Enhancements.
//!
//! Provides vocal fold micro-tremor (jitter & shimmer), vocal tract formant resonance modeling,
//! organic inhalation aspiration breath synthesis, and anti-click soft enveloping
//! to eliminate synthetic TTS artifacts.

use std::f32::consts::PI;
use super::eq::Biquad;

/// Lightweight deterministic xorshift PRNG for zero-allocation audio DSP loops.
struct FastRng(u64);

impl FastRng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 as f32) / (u64::MAX as f32)
    }

    /// Returns uniform sample in range [-1.0, 1.0].
    fn next_bipolar(&mut self) -> f32 {
        self.next_f32() * 2.0 - 1.0
    }
}

/// Injects subtle physiological vocal fold micro-tremor (organic pitch jitter & amplitude shimmer).
///
/// Eliminates rigid, machine-perfect pitch and volume profiles typical of synthetic neural TTS,
/// restoring natural neuromuscular elasticity and living human vocal presence.
pub struct HumanMicroTremor;

impl HumanMicroTremor {
    /// Injects neuromuscular vocal fold micro-tremor.
    ///
    /// - `jitter_depth`: Micro-pitch timing fluctuation factor (0.0 to 1.0, typically 0.2 to 0.5).
    /// - `shimmer_depth`: Micro-amplitude fluctuation factor (0.0 to 1.0, typically 0.2 to 0.5).
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn process(samples: &mut [i16], sample_rate: u32, jitter_depth: f32, shimmer_depth: f32) {
        if samples.len() < 32 || sample_rate == 0 {
            return;
        }

        let jitter = if jitter_depth.is_finite() {
            jitter_depth.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let shimmer = if shimmer_depth.is_finite() {
            shimmer_depth.clamp(0.0, 1.0)
        } else {
            0.0
        };

        if jitter < 0.01 && shimmer < 0.01 {
            return;
        }

        let sr = sample_rate as f32;
        let mut rng = FastRng::new(0x9E3779B97F4A7C15);

        // Physiological laryngeal tremor rate: ~5.3 Hz
        let tremor_rate_rad = 2.0 * PI * 5.3 / sr;
        let mut tremor_phase = 0.0f32;

        // Circular fractional delay line for sub-sample micro-jitter
        const DELAY_BUF_SIZE: usize = 32;
        let mut delay_buf = [0.0f32; DELAY_BUF_SIZE];
        let mut write_idx = 0usize;

        // Measure local energy envelope with smooth leaky follower
        let mut env = 0.0f32;
        let env_alpha = (-1.0 / (sr * 0.008)).exp(); // ~8ms window

        for sample in samples.iter_mut() {
            let in_val = *sample as f32 / 32768.0;
            let abs_in = in_val.abs();
            env = env_alpha * env + (1.0 - env_alpha) * abs_in;

            // Only modulate voiced regions (energy > -45 dBFS)
            let voicing_gate = ((env - 0.01) / 0.05).clamp(0.0, 1.0);

            // 1. Shimmer (amplitude micro-modulation)
            let lfo = tremor_phase.sin();
            let micro_noise = rng.next_bipolar() * 0.4;
            let shimmer_mod = (lfo * 0.012 + micro_noise * 0.008) * shimmer * voicing_gate;
            let amp_gain = 1.0 + shimmer_mod;

            // 2. Jitter (pitch timing micro-modulation)
            delay_buf[write_idx] = in_val * amp_gain;

            let jitter_samples = 4.0 + (lfo * 0.8 + rng.next_bipolar() * 0.4) * jitter * voicing_gate;
            let delay_clamped = jitter_samples.clamp(1.0, (DELAY_BUF_SIZE - 2) as f32);

            let int_delay = delay_clamped.floor() as usize;
            let frac = delay_clamped - int_delay as f32;

            let idx0 = (write_idx + DELAY_BUF_SIZE - int_delay) % DELAY_BUF_SIZE;
            let idx1 = (write_idx + DELAY_BUF_SIZE - int_delay - 1) % DELAY_BUF_SIZE;

            // Linear interpolation
            let out_val = delay_buf[idx0] * (1.0 - frac) + delay_buf[idx1] * frac;

            write_idx = (write_idx + 1) % DELAY_BUF_SIZE;
            tremor_phase += tremor_rate_rad;
            if tremor_phase > 2.0 * PI {
                tremor_phase -= 2.0 * PI;
            }

            *sample = (out_val * 32768.0).round().clamp(-32768.0, 32767.0) as i16;
        }
    }
}

/// Vocal Tract Resonator: sculpts formant acoustics matching the reference speaker's vocal anatomy.
pub struct VocalTractResonator;

impl VocalTractResonator {
    /// Applies 3-stage formant acoustic shaping to match speaker's throat, oral, and presence resonances.
    ///
    /// - `f1_hz`: Primary vowel/throat formant (typically 300 to 800 Hz).
    /// - `f1_gain_db`: Gain delta for F1 (-6.0 to +6.0 dB).
    /// - `f2_hz`: Secondary oral formant (typically 1000 to 2400 Hz).
    /// - `f2_gain_db`: Gain delta for F2 (-6.0 to +6.0 dB).
    /// - `presence_gain_db`: Singer's/actor presence boost at 3100 Hz (-4.0 to +4.0 dB).
    pub fn process(
        samples: &mut [i16],
        sample_rate: u32,
        f1_hz: f32,
        f1_gain_db: f32,
        f2_hz: f32,
        f2_gain_db: f32,
        presence_gain_db: f32,
    ) {
        if samples.is_empty() || sample_rate == 0 {
            return;
        }

        let sr = sample_rate as f32;
        let g1 = if f1_gain_db.is_finite() { f1_gain_db.clamp(-8.0, 8.0) } else { 0.0 };
        let g2 = if f2_gain_db.is_finite() { f2_gain_db.clamp(-8.0, 8.0) } else { 0.0 };
        let gp = if presence_gain_db.is_finite() { presence_gain_db.clamp(-6.0, 6.0) } else { 0.0 };

        if g1.abs() < 0.05 && g2.abs() < 0.05 && gp.abs() < 0.05 {
            return;
        }

        let safe_f1 = f1_hz.clamp(200.0, 1000.0);
        let safe_f2 = f2_hz.clamp(900.0, 2600.0);

        let mut b1 = Biquad::peaking(sr, safe_f1, g1, 2.0);
        let mut b2 = Biquad::peaking(sr, safe_f2, g2, 2.2);
        let mut bp = Biquad::peaking(sr, 3100.0, gp, 1.8);

        for sample in samples.iter_mut() {
            let in_val = *sample as f32;
            let out1 = b1.process(in_val);
            let out2 = b2.process(out1);
            let outp = bp.process(out2);

            if outp.is_finite() {
                *sample = outp.round().clamp(-32768.0, 32767.0) as i16;
            }
        }
    }
}

/// Soft Envelope: applies Hann cosine tapers to audio boundaries to eliminate digital onset/offset clicks.
pub struct SoftEnvelope;

impl SoftEnvelope {
    /// Applies cosine fade-in and fade-out ramps.
    ///
    /// - `attack_ms`: Fade-in duration in milliseconds (e.g. 8.0 ms).
    /// - `release_ms`: Fade-out duration in milliseconds (e.g. 24.0 ms).
    pub fn process(samples: &mut [i16], sample_rate: u32, attack_ms: f32, release_ms: f32) {
        if samples.is_empty() || sample_rate == 0 {
            return;
        }

        let sr = sample_rate as f32;
        let attack_len = ((attack_ms * 0.001 * sr).round() as usize).min(samples.len() / 2);
        let release_len = ((release_ms * 0.001 * sr).round() as usize).min(samples.len() / 2);

        // Cosine attack fade-in: 0.5 * (1 - cos(pi * i / N))
        for (i, sample) in samples.iter_mut().take(attack_len).enumerate() {
            let factor = 0.5 * (1.0 - (PI * i as f32 / attack_len as f32).cos());
            *sample = ((*sample as f32 * factor).round()) as i16;
        }

        // Cosine release fade-out: smoothly reaches exact zero at final sample
        let total = samples.len();
        let release_start = total - release_len;
        for (i, sample) in samples[release_start..].iter_mut().enumerate() {
            let factor = 0.5 * (1.0 + (PI * (i + 1) as f32 / release_len as f32).cos());
            *sample = ((*sample as f32 * factor).round()) as i16;
        }
    }
}

/// Natural Inhalation Breath Generator: produces realistic human vocal aspiration before speech or at pauses.
pub struct NaturalBreath;

impl NaturalBreath {
    /// Synthesizes an organic vocal tract inhalation breath sound.
    ///
    /// - `sample_rate`: Target audio sample rate.
    /// - `duration_ms`: Duration of breath in milliseconds (120ms to 220ms).
    /// - `level_dbfs`: Peak amplitude level (typically -40.0 to -34.0 dBFS).
    /// - `formant_center_hz`: Vocal tract resonance center frequency (1400 to 2200 Hz).
    pub fn generate(
        sample_rate: u32,
        duration_ms: f32,
        level_dbfs: f32,
        formant_center_hz: f32,
    ) -> Vec<i16> {
        if sample_rate == 0 {
            return Vec::new();
        }

        let sr = sample_rate as f32;
        let dur = duration_ms.clamp(60.0, 350.0);
        let num_samples = ((dur * 0.001 * sr).round() as usize).max(64);

        let target_peak_lin = 10.0f32.powf(level_dbfs.clamp(-55.0, -20.0) / 20.0) * 32768.0;
        let center = formant_center_hz.clamp(800.0, 3000.0);

        let mut bpf = Biquad::band_pass(sr, center, 1.4);
        let mut rng = FastRng::new(0x4D61726973686121);

        let mut breath_samples = Vec::with_capacity(num_samples);

        // Asymmetric inhalation envelope: 65% rise, 35% decay
        let rise_len = (num_samples as f32 * 0.65).round() as usize;
        let decay_len = num_samples - rise_len;

        for i in 0..num_samples {
            // Noise generator with pink coloration (two-pole filter)
            let white = rng.next_bipolar();
            let filtered = bpf.process(white);

            let env = if i < rise_len {
                // Slower organic breath intake rise
                0.5 * (1.0 - (PI * i as f32 / rise_len as f32).cos())
            } else {
                // Soft decay
                let decay_idx = i - rise_len;
                0.5 * (1.0 + (PI * decay_idx as f32 / decay_len as f32).cos())
            };

            let pcm = (filtered * env * target_peak_lin).round().clamp(-32768.0, 32767.0) as i16;
            breath_samples.push(pcm);
        }

        breath_samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_micro_tremor_finite_and_stable() {
        let mut samples = vec![12000i16; 1000];
        let original = samples.clone();
        HumanMicroTremor::process(&mut samples, 24000, 0.4, 0.3);

        assert_eq!(samples.len(), original.len());
        assert!(samples.iter().all(|&s| s != i16::MIN));
        // Tremor should introduce micro-variation into sustained constant tone
        assert_ne!(samples, original);
    }

    #[test]
    fn test_human_micro_tremor_zero_depth_noop() {
        let mut samples = vec![500, -1000, 2000];
        let original = samples.clone();
        HumanMicroTremor::process(&mut samples, 24000, 0.0, 0.0);
        assert_eq!(samples, original);
    }

    #[test]
    fn test_vocal_tract_resonator_process() {
        let mut samples = vec![8000i16; 500];
        VocalTractResonator::process(&mut samples, 24000, 500.0, 3.0, 1600.0, 2.0, 1.5);
        assert_eq!(samples.len(), 500);
        assert!(samples.iter().all(|&s| s != i16::MIN));
    }

    #[test]
    fn test_soft_envelope_tapers_boundaries() {
        let mut samples = vec![20000i16; 1000];
        SoftEnvelope::process(&mut samples, 24000, 10.0, 10.0);
        assert_eq!(samples[0], 0);
        assert_eq!(*samples.last().unwrap(), 0);
        assert!(samples[500] > 18000); // Core mid-speech remains intact
    }

    #[test]
    fn test_natural_breath_generates_valid_samples() {
        let breath = NaturalBreath::generate(24000, 150.0, -38.0, 1800.0);
        assert!(!breath.is_empty());
        let peak = breath.iter().map(|&s| s.abs()).max().unwrap_or(0);
        assert!(peak > 0 && peak < 2000, "Breath peak must be soft and within acoustic bounds");
    }
}
