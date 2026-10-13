pub struct AudioNormalizer;

impl AudioNormalizer {
    pub fn peak_normalize(samples: &mut [i16], target_peak_ratio: f32) {
        if samples.is_empty() || !target_peak_ratio.is_finite() {
            return;
        }

        let max_abs = samples.iter().map(|&s| (s as i32).abs()).max().unwrap_or(0);

        if max_abs == 0 {
            return;
        }

        let target_peak = 32767.0 * target_peak_ratio.clamp(0.0, 1.0);
        let gain = target_peak / (max_abs as f32);

        if !gain.is_finite() || (gain - 1.0).abs() < 0.001 {
            return;
        }

        for sample in samples.iter_mut() {
            let scaled = (*sample as f32 * gain).round();
            if scaled.is_finite() {
                *sample = scaled.clamp(-32768.0, 32767.0) as i16;
            }
        }
    }

    pub fn apply_gain_db(samples: &mut [i16], gain_db: f32) {
        if !gain_db.is_finite() || gain_db.abs() < 0.01 {
            return;
        }

        let safe_gain_db = gain_db.clamp(-120.0, 60.0);
        let linear_factor = 10.0f32.powf(safe_gain_db / 20.0);
        if !linear_factor.is_finite() {
            return;
        }

        for sample in samples.iter_mut() {
            let scaled = (*sample as f32 * linear_factor).round();
            if scaled.is_finite() {
                *sample = scaled.clamp(-32768.0, 32767.0) as i16;
            }
        }
    }

    /// Normalizes audio so its Root Mean Square (RMS) energy matches `target_rms_ratio` (0.0 to 1.0 relative to full scale)
    pub fn rms_normalize(samples: &mut [i16], target_rms_ratio: f32) {
        if samples.is_empty() || !target_rms_ratio.is_finite() {
            return;
        }

        let sum_sq: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
        let rms = (sum_sq / samples.len() as f64).sqrt();
        if rms < 1.0 {
            return;
        }

        let target_rms = 32767.0 * target_rms_ratio.clamp(0.01, 1.0) as f64;
        let gain = (target_rms / rms) as f32;
        if !gain.is_finite() || (gain - 1.0).abs() < 0.001 {
            return;
        }

        for sample in samples.iter_mut() {
            let scaled = (*sample as f32 * gain).round();
            if scaled.is_finite() {
                *sample = scaled.clamp(-32768.0, 32767.0) as i16;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_and_nan_peak_normalize() {
        let mut samples = vec![];
        AudioNormalizer::peak_normalize(&mut samples, 0.9);
        assert!(samples.is_empty());

        let mut samples = vec![100, 200];
        let original = samples.clone();
        AudioNormalizer::peak_normalize(&mut samples, f32::NAN);
        assert_eq!(samples, original);
    }

    #[test]
    fn test_all_zeros_peak_normalize() {
        let mut samples = vec![0, 0, 0];
        AudioNormalizer::peak_normalize(&mut samples, 1.0);
        assert_eq!(samples, vec![0, 0, 0]);
    }

    #[test]
    fn test_rms_normalize_nan_and_silence() {
        let mut samples = vec![0, 0, 0];
        AudioNormalizer::rms_normalize(&mut samples, 0.2);
        assert_eq!(samples, vec![0, 0, 0]);

        let mut samples = vec![1000, 2000];
        let original = samples.clone();
        AudioNormalizer::rms_normalize(&mut samples, f32::NAN);
        assert_eq!(samples, original);
    }

    #[test]
    fn test_apply_gain_db_boundary() {
        let mut samples = vec![10000];
        // Large boost clamps to i16::MAX safely
        AudioNormalizer::apply_gain_db(&mut samples, 40.0);
        assert_eq!(samples[0], 32767);

        // Huge attenuation brings to near-zero
        AudioNormalizer::apply_gain_db(&mut samples, -100.0);
        assert!(samples[0].abs() <= 1);
    }
}
