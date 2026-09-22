//! Intelligent dynamic sibilance compressor (De-Esser) for neural speech.
//!
//! Mitigates piercing high-frequency sibilance ("s", "sh", "ch", "z") in the
//! 5.0 kHz to 8.5 kHz band produced by neural speech models on headphones.

use std::f32::consts::PI;

pub struct DeEsser;

impl DeEsser {
    /// Apply dynamic de-essing to 16-bit PCM audio.
    ///
    /// - `frequency_hz`: Center sibilance frequency (typically 5500.0 to 7500.0 Hz).
    /// - `threshold_dbfs`: Activation threshold in dBFS (e.g. -22.0 dBFS).
    /// - `max_reduction_db`: Maximum sibilance reduction depth in dB (e.g. 6.0 to 12.0 dB).
    pub fn process(
        samples: &mut [i16],
        sample_rate: u32,
        frequency_hz: f32,
        threshold_dbfs: f32,
        max_reduction_db: f32,
    ) {
        if samples.is_empty() {
            return;
        }

        let sr = sample_rate.max(8000) as f32;
        // Nyquist constraint: center frequency must be below sr / 2
        let center_hz = frequency_hz.clamp(2000.0, (sr * 0.45).max(2000.0));
        let thresh = threshold_dbfs.clamp(-60.0, 0.0);
        let max_red = max_reduction_db.clamp(0.0, 24.0);

        if max_red < 0.1 {
            return;
        }

        // 2nd-order bandpass filter for sibilance detection (Q = 1.8)
        let q = 1.8f32;
        let w0 = 2.0 * PI * (center_hz / sr);
        let alpha = w0.sin() / (2.0 * q);
        let b0 = alpha;
        let b1 = 0.0f32;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;

        let nb0 = b0 / a0;
        let nb1 = b1 / a0;
        let nb2 = b2 / a0;
        let na1 = a1 / a0;
        let na2 = a2 / a0;

        let mut x1 = 0.0f32;
        let mut x2 = 0.0f32;
        let mut y1 = 0.0f32;
        let mut y2 = 0.0f32;

        // Envelope follower parameters: fast attack (1ms) and moderate release (30ms)
        let att_coeff = (-1.0 / (sr * 0.001)).exp();
        let rel_coeff = (-1.0 / (sr * 0.030)).exp();
        let mut sibilance_envelope = 0.0f32;

        let linear_thresh = 10.0f32.powf(thresh / 20.0);
        let min_gain = 10.0f32.powf(-max_red / 20.0);

        for sample in samples.iter_mut() {
            let x0 = *sample as f32 / 32768.0;

            // Extract sibilance band via bandpass filter
            let band_val = nb0 * x0 + nb1 * x1 + nb2 * x2 - na1 * y1 - na2 * y2;
            x2 = x1;
            x1 = x0;
            y2 = y1;
            y1 = band_val;

            let band_energy = band_val.abs();

            // Track envelope
            if band_energy > sibilance_envelope {
                sibilance_envelope =
                    att_coeff * sibilance_envelope + (1.0 - att_coeff) * band_energy;
            } else {
                sibilance_envelope =
                    rel_coeff * sibilance_envelope + (1.0 - rel_coeff) * band_energy;
            }

            // Calculate dynamic attenuation if sibilance exceeds threshold
            let gain = if sibilance_envelope > linear_thresh {
                let over_ratio = sibilance_envelope / linear_thresh;
                (1.0 / over_ratio.sqrt()).clamp(min_gain, 1.0)
            } else {
                1.0
            };

            // Apply gain smoothly to speech sample
            let processed = (x0 * gain * 32768.0).round();
            *sample = processed.clamp(-32768.0, 32767.0) as i16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deesser_reduces_high_freq_burst() {
        let sample_rate = 24000;
        let mut samples = Vec::new();
        // Generate high amplitude 6.5kHz sibilant burst
        for i in 0..2400 {
            let t = i as f32 / sample_rate as f32;
            let val = (2.0 * PI * 6500.0 * t).sin() * 28000.0;
            samples.push(val as i16);
        }

        let max_before = samples.iter().map(|s| s.abs()).max().unwrap();
        DeEsser::process(&mut samples, sample_rate, 6500.0, -20.0, 10.0);
        let max_after = samples.iter().map(|s| s.abs()).max().unwrap();

        assert!(
            max_after < max_before,
            "De-Esser must attenuate harsh sibilant burst"
        );
    }

    #[test]
    fn test_deesser_leaves_low_freq_untouched() {
        let sample_rate = 24000;
        let mut samples = Vec::new();
        // Generate low frequency 200Hz tone
        for i in 0..1000 {
            let t = i as f32 / sample_rate as f32;
            let val = (2.0 * PI * 200.0 * t).sin() * 10000.0;
            samples.push(val as i16);
        }

        let orig = samples.clone();
        DeEsser::process(&mut samples, sample_rate, 6500.0, -20.0, 10.0);

        // Difference should be near zero because bandpass rejects 200Hz
        let diff: i32 = samples
            .iter()
            .zip(orig.iter())
            .map(|(a, b)| (*a as i32 - *b as i32).abs())
            .sum();
        let avg_diff = diff as f32 / samples.len() as f32;
        assert!(
            avg_diff < 50.0,
            "De-Esser must not alter non-sibilant low frequencies"
        );
    }
}
