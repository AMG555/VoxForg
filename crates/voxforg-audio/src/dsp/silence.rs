pub struct SilenceTrimmer;

impl SilenceTrimmer {
    pub fn trim(
        samples: &[i16],
        sample_rate: u32,
        threshold_dbfs: f32,
        padding_ms: u32,
    ) -> Vec<i16> {
        if samples.is_empty() {
            return Vec::new();
        }

        let safe_threshold_dbfs = if threshold_dbfs.is_finite() {
            threshold_dbfs.clamp(-96.0, 0.0)
        } else {
            -45.0
        };

        // Linear threshold amplitude: 32767 * 10^(threshold_dbfs / 20)
        let linear_threshold = (32767.0 * 10.0f32.powf(safe_threshold_dbfs / 20.0)).round() as i32;
        let threshold_amp = linear_threshold.max(1);

        let safe_sample_rate = sample_rate.max(1);
        let safe_padding_ms = padding_ms.min(2000);
        let padding_samples = ((safe_sample_rate as f32) * (safe_padding_ms as f32 / 1000.0)) as usize;

        // Find first sample above threshold
        let first_loud = samples.iter().position(|&s| (s as i32).abs() >= threshold_amp);
        let last_loud = samples.iter().rposition(|&s| (s as i32).abs() >= threshold_amp);

        match (first_loud, last_loud) {
            (Some(start_idx), Some(end_idx)) if start_idx <= end_idx => {
                let start = start_idx.saturating_sub(padding_samples);
                let end = (end_idx + 1 + padding_samples).min(samples.len());
                samples[start..end].to_vec()
            }
            _ => {
                // Entire signal below threshold
                Vec::new()
            }
        }
    }
}
