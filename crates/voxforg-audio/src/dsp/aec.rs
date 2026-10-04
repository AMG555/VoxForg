//! Acoustic Echo Cancellation (AEC) using Normalized Least Mean Squares (NLMS).
//!
//! Eliminates loudspeaker feedback from microphone capture streams during live dictation
//! and interactive duplex speech sessions.

/// Adaptive NLMS Acoustic Echo Canceller with Double-Talk Detection (DTD).
pub struct AcousticEchoCanceller {
    filter_length: usize,
    weights: Vec<f32>,
    x_history: Vec<f32>,
    history_idx: usize,
    step_size_mu: f32,
    leakage: f32,
    double_talk_hold_frames: usize,
    double_talk_counter: usize,
}

impl AcousticEchoCanceller {
    /// Creates a new AEC engine with a default filter length of 512 taps (~21ms at 24kHz).
    pub fn new(filter_length: usize) -> Self {
        let len = filter_length.clamp(64, 2048);
        Self {
            filter_length: len,
            weights: vec![0.0; len],
            x_history: vec![0.0; len],
            history_idx: 0,
            step_size_mu: 0.15,
            leakage: 0.9999,
            double_talk_hold_frames: 40,
            double_talk_counter: 0,
        }
    }

    /// Process a single frame of samples.
    /// - `mic_samples`: Raw microphone signal $d(n)$ containing near-end speech + far-end echo.
    /// - `speaker_samples`: Reference loudspeaker audio $x(n)$ being played out.
    ///
    /// Returns the cleaned error signal $e(n)$ with echo subtracted.
    pub fn process_frame(&mut self, mic_samples: &[i16], speaker_samples: &[i16]) -> Vec<i16> {
        let n = mic_samples.len().min(speaker_samples.len());
        let mut clean_output = Vec::with_capacity(n);

        for i in 0..n {
            let mic_val = mic_samples[i] as f32 / 32768.0;
            let spk_val = speaker_samples[i] as f32 / 32768.0;

            let clean_sample = self.process_sample(mic_val, spk_val);
            let pcm = (clean_sample * 32767.0).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            clean_output.push(pcm);
        }

        clean_output
    }

    /// Process a single scalar sample pair.
    pub fn process_sample(&mut self, mic: f32, speaker_ref: f32) -> f32 {
        // 1. Update circular buffer with new reference sample
        self.x_history[self.history_idx] = speaker_ref;

        // 2. Compute predicted echo y_hat = W^T * X
        let mut echo_estimate = 0.0f32;
        let mut power_x = 0.0f32;

        let len = self.filter_length;
        for k in 0..len {
            let buf_pos = (self.history_idx + len - k) % len;
            let x_k = self.x_history[buf_pos];
            echo_estimate += self.weights[k] * x_k;
            power_x += x_k * x_k;
        }

        // 3. Compute error residual e(n) = mic - echo_estimate
        let error = mic - echo_estimate;

        // 4. Double-talk detector: Geigel condition
        // If microphone energy is significantly higher than reference signal,
        // local speaker is active -> freeze filter updates to prevent divergence.
        let mic_power = mic * mic;
        let is_double_talk = mic_power > (power_x * 4.0 + 0.005);

        if is_double_talk {
            self.double_talk_counter = self.double_talk_hold_frames;
        } else if self.double_talk_counter > 0 {
            self.double_talk_counter -= 1;
        }

        // 5. Update weights using Normalized LMS if double-talk is inactive
        if self.double_talk_counter == 0 && power_x > 1e-5 {
            let norm = power_x + 1e-4;
            let factor = (self.step_size_mu * error) / norm;

            for k in 0..len {
                let buf_pos = (self.history_idx + len - k) % len;
                let x_k = self.x_history[buf_pos];
                // Apply slight leakage to prevent drift in null sub-bands
                self.weights[k] = self.weights[k] * self.leakage + factor * x_k;
            }
        }

        // Advance circular index
        self.history_idx = (self.history_idx + 1) % len;

        error
    }

    /// Reset internal filter state and histories.
    pub fn reset(&mut self) {
        self.weights.fill(0.0);
        self.x_history.fill(0.0);
        self.history_idx = 0;
        self.double_talk_counter = 0;
    }
}
