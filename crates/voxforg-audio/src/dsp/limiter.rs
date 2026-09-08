pub struct BrickwallLimiter;

impl BrickwallLimiter {
    pub fn process(samples: &mut [i16], ceiling_dbfs: f32) {
        if samples.is_empty() {
            return;
        }

        let safe_ceiling_dbfs = if ceiling_dbfs.is_finite() {
            ceiling_dbfs.clamp(-24.0, 0.0)
        } else {
            -0.5
        };

        let ceiling_amplitude = (32767.0 * 10.0f32.powf(safe_ceiling_dbfs / 20.0)).round() as i32;
        let max_amp = ceiling_amplitude.clamp(1, 32767);
        let min_amp = -max_amp;

        for sample in samples.iter_mut() {
            let val = *sample as i32;
            if val > max_amp {
                *sample = max_amp as i16;
            } else if val < min_amp {
                *sample = min_amp as i16;
            }
        }
    }
}
