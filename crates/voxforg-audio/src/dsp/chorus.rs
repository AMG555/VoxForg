//! Multi-voice modulated chorus and vocal thickening processor.
//!
//! Adds width, warmth, and subtle pitch-vibrato movement to synthesized
//! neural voices by modulating short delay lines with low-frequency oscillators.

pub struct VocalChorus;

impl VocalChorus {
    /// Apply vocal chorus modulation to 16-bit PCM samples.
    ///
    /// - `rate_hz`: LFO modulation rate in Hz (0.1 to 10.0 Hz). Typical: 1.5 Hz.
    /// - `depth_ms`: Delay modulation depth in milliseconds (0.5 to 15.0 ms). Typical: 3.5 ms.
    /// - `wet_mix`: Level of modulated voice blend (0.0 to 1.0).
    /// - `dry_mix`: Level of dry voice signal (0.0 to 1.0).
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn process(
        samples: &mut [i16],
        rate_hz: f32,
        depth_ms: f32,
        wet_mix: f32,
        dry_mix: f32,
        sample_rate: u32,
    ) {
        if samples.is_empty() || wet_mix <= 0.001 {
            return;
        }

        let base_delay_ms = 10.0f32;
        let max_delay_ms = base_delay_ms + depth_ms.clamp(0.5, 15.0);
        let max_delay_samples =
            ((sample_rate as f32) * (max_delay_ms / 1000.0)).ceil() as usize + 2;

        let base_delay_samples = (sample_rate as f32) * (base_delay_ms / 1000.0);
        let mod_depth_samples = (sample_rate as f32) * (depth_ms.clamp(0.5, 15.0) / 1000.0);

        let lfo_step = 2.0 * std::f32::consts::PI * rate_hz.clamp(0.1, 10.0) / (sample_rate as f32);
        let mut lfo_phase = 0.0f32;

        let mut buffer = vec![0.0f32; max_delay_samples];
        let mut write_idx = 0usize;

        let wet = wet_mix.clamp(0.0, 1.0);
        let dry = dry_mix.clamp(0.0, 1.0);

        for sample in samples.iter_mut() {
            let input = (*sample as f32) / 32768.0;
            buffer[write_idx] = input;

            // Compute modulated delay offset
            let mod_delay = base_delay_samples + (lfo_phase.sin() * mod_depth_samples);
            let read_pos = (write_idx as f32) - mod_delay;
            let read_pos_wrapped = if read_pos < 0.0 {
                read_pos
                    + (max_delay_samples as f32)
                        * ((-read_pos / max_delay_samples as f32).ceil() + 1.0)
            } else {
                read_pos
            } % (max_delay_samples as f32);

            let idx0 = read_pos_wrapped.floor() as usize % max_delay_samples;
            let idx1 = (idx0 + 1) % max_delay_samples;
            let frac = read_pos_wrapped - read_pos_wrapped.floor();

            // Linear interpolation of delayed signal
            let delayed = buffer[idx0] * (1.0 - frac) + buffer[idx1] * frac;

            write_idx = (write_idx + 1) % max_delay_samples;
            lfo_phase += lfo_step;
            if lfo_phase >= 2.0 * std::f32::consts::PI {
                lfo_phase -= 2.0 * std::f32::consts::PI;
            }

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
        VocalChorus::process(&mut samples, 1.5, 3.0, 0.0, 1.0, 24000);
        assert_eq!(samples, original);
    }

    #[test]
    fn test_chorus_modulates_output() {
        let mut samples = vec![15000; 1000];
        VocalChorus::process(&mut samples, 2.0, 4.0, 0.5, 0.5, 24000);
        // Due to delay ramp-up and LFO, samples should vary across time
        assert!(samples.iter().any(|&s| s != 15000));
    }
}
