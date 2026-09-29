//! Algorithmic studio reverberation processor.
//!
//! Provides spatial room acoustic reflections based on a multi-stage comb
//! and allpass filter network (Schroeder/Freeverb architecture), engineered
//! for low latency vocal and studio speech rendering.

pub struct AlgorithmicReverb;

struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    filter_store: f32,
}

impl CombFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size.max(1)],
            index: 0,
            filter_store: 0.0,
        }
    }

    fn process(&mut self, input: f32, feedback: f32, damp: f32) -> f32 {
        let output = self.buffer[self.index];
        self.filter_store = output * (1.0 - damp) + self.filter_store * damp;
        self.buffer[self.index] = input + self.filter_store * feedback;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

struct AllpassFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
}

impl AllpassFilter {
    fn new(size: usize, feedback: f32) -> Self {
        Self {
            buffer: vec![0.0; size.max(1)],
            index: 0,
            feedback,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buf_out = self.buffer[self.index];
        let output = -input + buf_out;
        self.buffer[self.index] = input + (buf_out * self.feedback);
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

impl AlgorithmicReverb {
    /// Apply algorithmic room reverberation to 16-bit PCM audio samples.
    ///
    /// - `room_size`: Room reflection decay factor (0.0 to 1.0).
    /// - `damping`: High-frequency absorption / softness (0.0 to 1.0).
    /// - `wet_mix`: Level of reverberated acoustics (0.0 to 1.0).
    /// - `dry_mix`: Level of pristine original signal (0.0 to 1.0).
    /// - `sample_rate`: Audio sample rate in Hz (e.g. 24000, 48000).
    pub fn process(
        samples: &mut [i16],
        room_size: f32,
        damping: f32,
        wet_mix: f32,
        dry_mix: f32,
        sample_rate: u32,
    ) {
        if samples.is_empty() || wet_mix <= 0.001 {
            return;
        }

        let rate_scale = (sample_rate as f32) / 44100.0;
        let room = room_size.clamp(0.0, 0.98);
        let damp = damping.clamp(0.0, 0.95);
        let wet = wet_mix.clamp(0.0, 1.0);
        let dry = dry_mix.clamp(0.0, 1.0);

        // Tuned comb filter delay lengths (in samples at 44.1kHz)
        let comb_tunings = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
        let allpass_tunings = [556, 441, 341, 225];

        let mut combs: Vec<CombFilter> = comb_tunings
            .iter()
            .map(|&t| CombFilter::new(((t as f32) * rate_scale).round() as usize))
            .collect();

        let mut allpasses: Vec<AllpassFilter> = allpass_tunings
            .iter()
            .map(|&t| AllpassFilter::new(((t as f32) * rate_scale).round() as usize, 0.5))
            .collect();

        let feedback = 0.7 + (room * 0.28);

        for sample in samples.iter_mut() {
            let input = (*sample as f32) / 32768.0;

            // Parallel comb filter stage
            let mut comb_sum = 0.0f32;
            for comb in combs.iter_mut() {
                comb_sum += comb.process(input, feedback, damp);
            }
            comb_sum *= 0.125; // Normalise 8 comb outputs

            // Serial allpass filter stage for diffusion
            let mut diffused = comb_sum;
            for allpass in allpasses.iter_mut() {
                diffused = allpass.process(diffused);
            }

            let mixed = (input * dry) + (diffused * wet);
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
        let mut samples = vec![500, -1000, 1500, -2000];
        let original = samples.clone();
        AlgorithmicReverb::process(&mut samples, 0.7, 0.3, 0.0, 1.0, 24000);
        assert_eq!(samples, original);
    }

    #[test]
    fn test_reverb_generates_tail_decay() {
        // Feed an impulse into silence: reverb should fill subsequent silence
        let mut samples = vec![0i16; 4000];
        samples[0] = 30000;
        AlgorithmicReverb::process(&mut samples, 0.8, 0.2, 0.5, 1.0, 24000);
        let tail_energy: i64 = samples[500..1500].iter().map(|&s| s.abs() as i64).sum();
        assert!(
            tail_energy > 0,
            "Reverb must produce audible reflection decay tail"
        );
    }
}
