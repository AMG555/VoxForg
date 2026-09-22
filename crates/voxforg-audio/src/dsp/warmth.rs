//! Analogue tube & tape warmth harmonic processor.
//!
//! Enriches digital neural speech with subtle even- and odd-order harmonics,
//! providing the acoustic warmth and body of professional condenser studio microphones.

pub struct HarmonicWarmth;

impl HarmonicWarmth {
    /// Apply harmonic saturation and acoustic warmth to 16-bit PCM speech.
    ///
    /// - `drive`: Saturation amount between 0.0 (clean linear bypass) and 1.0 (rich analogue warmth).
    /// - `warmth_blend`: Mix level between dry and saturated signal (0.0 to 1.0).
    pub fn process(samples: &mut [i16], drive: f32, warmth_blend: f32) {
        if samples.is_empty() {
            return;
        }

        let d = drive.clamp(0.0, 1.0);
        let blend = warmth_blend.clamp(0.0, 1.0);

        if d < 0.01 || blend < 0.01 {
            return;
        }

        let scale = 1.0 + d * 0.4;

        for sample in samples.iter_mut() {
            let x = *sample as f32 / 32768.0;

            // Soft-knee analogue polynomial saturation: rounds peaks gently and adds warm harmonics
            let sat = (x * scale) / (1.0 + d * x * x);

            // Blend dry and wet signals for transparent natural warmth
            let blended = x * (1.0 - blend) + sat * blend;

            let out = (blended * 32768.0).round();
            *sample = out.clamp(-32768.0, 32767.0) as i16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warmth_zero_drive_leaves_samples_unaltered() {
        let mut samples = vec![1000, -2000, 3000, -4000];
        let original = samples.clone();
        HarmonicWarmth::process(&mut samples, 0.0, 1.0);
        assert_eq!(samples, original);
    }

    #[test]
    fn test_warmth_saturates_loud_peaks_gently() {
        let mut samples = vec![30000, -30000];
        HarmonicWarmth::process(&mut samples, 0.8, 1.0);
        assert!(
            samples[0] < 30000,
            "Warmth saturation must gently compress loud peaks"
        );
        assert!(samples[1] > -30000);
    }
}
