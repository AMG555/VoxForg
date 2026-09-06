pub struct AudioNormalizer;

impl AudioNormalizer {
    pub fn peak_normalize(samples: &mut [i16], target_peak_ratio: f32) {
        if samples.is_empty() {
            return;
        }

        let max_abs = samples
            .iter()
            .map(|&s| (s as i32).abs())
            .max()
            .unwrap_or(0);

        if max_abs == 0 {
            return;
        }

        let target_peak = (32767.0 * target_peak_ratio.clamp(0.0, 1.0)) as f32;
        let gain = target_peak / (max_abs as f32);

        if !gain.is_finite() || (gain - 1.0).abs() < 0.001 {
            return;
        }

        for sample in samples.iter_mut() {
            let scaled = (*sample as f32 * gain).round();
            *sample = scaled.clamp(-32768.0, 32767.0) as i16;
        }
    }

    pub fn apply_gain_db(samples: &mut [i16], gain_db: f32) {
        if !gain_db.is_finite() || gain_db.abs() < 0.01 {
            return;
        }

        let linear_factor = 10.0f32.powf(gain_db / 20.0);
        for sample in samples.iter_mut() {
            let scaled = (*sample as f32 * linear_factor).round();
            *sample = scaled.clamp(-32768.0, 32767.0) as i16;
        }
    }
}
