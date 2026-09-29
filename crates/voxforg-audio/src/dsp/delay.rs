//! Studio feedback echo and delay line processor.
//!
//! Enriches vocal and ambient acoustics with adjustable echo delay,
//! feedback recirculation, and stereo-ready wet/dry blend.

pub struct EchoDelay;

impl EchoDelay {
    /// Apply feedback echo delay to 16-bit PCM samples.
    ///
    /// - `delay_ms`: Echo delay time in milliseconds (1.0 to 2000.0 ms).
    /// - `feedback`: Feedback recirculation coefficient (0.0 to 0.95).
    /// - `wet_mix`: Echo level mix (0.0 to 1.0).
    /// - `dry_mix`: Direct signal level mix (0.0 to 1.0).
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn process(
        samples: &mut [i16],
        delay_ms: f32,
        feedback: f32,
        wet_mix: f32,
        dry_mix: f32,
        sample_rate: u32,
    ) {
        if samples.is_empty() || wet_mix <= 0.001 {
            return;
        }

        let delay_sec = (delay_ms.clamp(1.0, 2000.0)) / 1000.0;
        let delay_samples = ((sample_rate as f32) * delay_sec).round() as usize;
        if delay_samples == 0 {
            return;
        }

        let fb = feedback.clamp(0.0, 0.95);
        let wet = wet_mix.clamp(0.0, 1.0);
        let dry = dry_mix.clamp(0.0, 1.0);

        let mut buffer = vec![0.0f32; delay_samples];
        let mut write_idx = 0usize;

        for sample in samples.iter_mut() {
            let input = (*sample as f32) / 32768.0;
            let delayed = buffer[write_idx];

            let new_delayed_val = input + (delayed * fb);
            buffer[write_idx] = new_delayed_val;

            write_idx = (write_idx + 1) % delay_samples;

            let mixed = (input * dry) + (delayed * wet);
            let clamped = (mixed * 32768.0).round().clamp(-32768.0, 32767.0);
            *sample = clamped as i16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_wet_leaves_samples_unaltered() {
        let mut samples = vec![1000, -2000, 3000, -4000];
        let original = samples.clone();
        EchoDelay::process(&mut samples, 100.0, 0.5, 0.0, 1.0, 24000);
        assert_eq!(samples, original);
    }

    #[test]
    fn test_echo_recirculates_signal() {
        let mut samples = vec![0i16; 5000];
        samples[0] = 30000;
        let delay_ms = 50.0; // 50ms at 24kHz = 1200 samples
        EchoDelay::process(&mut samples, delay_ms, 0.5, 0.8, 1.0, 24000);
        // Delay hit should appear around index 1200
        assert!(
            samples[1200].abs() > 5000,
            "Echo must create delayed replica"
        );
    }
}
