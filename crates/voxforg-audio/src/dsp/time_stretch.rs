//! Pitch-preserving WSOLA (Waveform Similarity Overlap-Add) time-stretching algorithm.
//!
//! Enables high-fidelity vocal tempo scaling (speedup / slowdown) from 0.5x to 2.0x
//! while maintaining pitch and formant frequencies.

/// WSOLA pitch-preserving time stretcher.
pub struct WsolaTimeStretch {
    frame_size: usize,
    overlap: usize,
    search_window: usize,
}

impl WsolaTimeStretch {
    /// Creates a new WSOLA processor tuned for conversational speech.
    ///
    /// For 24kHz / 22.05kHz speech:
    /// - frame_size: 512 samples (~21ms)
    /// - overlap: 256 samples (~50% overlap)
    /// - search_window: 128 samples (~5ms search radius for pitch-synchronous alignment)
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000) as usize;
        // Target ~25ms frame, ~12.5ms hop, ~6ms search window
        let frame_size = (sr * 25 / 1000).next_power_of_two().clamp(256, 2048);
        let overlap = frame_size / 2;
        let search_window = overlap / 2;

        Self {
            frame_size,
            overlap,
            search_window,
        }
    }

    /// Time-stretch 16-bit PCM speech to an exact target number of samples.
    ///
    /// Preserves natural vocal pitch and timbre without metallic artifacts.
    pub fn stretch_to_len(&self, input: &[i16], target_len: usize) -> Vec<i16> {
        if input.is_empty() || target_len == 0 {
            return Vec::new();
        }
        if input.len() == target_len {
            return input.to_vec();
        }

        let speed_ratio = input.len() as f64 / target_len as f64;
        self.stretch_by_ratio(input, speed_ratio, target_len)
    }

    /// Time-stretch 16-bit PCM speech by a speed ratio (e.g. 1.25 for 25% faster).
    pub fn stretch_by_ratio(&self, input: &[i16], speed_ratio: f64, exact_target_len: usize) -> Vec<i16> {
        let speed = speed_ratio.clamp(0.4, 2.5);
        if (speed - 1.0).abs() < 0.005 && (exact_target_len == 0 || exact_target_len == input.len()) {
            return input.to_vec();
        }

        let n_in = input.len();
        if n_in < self.frame_size + self.search_window {
            // Audio too short for WSOLA frames; fall back to linear interpolation
            return Self::linear_resample(input, exact_target_len.max((n_in as f64 / speed) as usize));
        }

        let hop_out = self.overlap;
        let hop_in = (hop_out as f64 * speed).round() as usize;
        if hop_in == 0 {
            return input.to_vec();
        }

        let estimated_out_len = if exact_target_len > 0 {
            exact_target_len
        } else {
            (n_in as f64 / speed).round() as usize
        };

        let mut output = Vec::with_capacity(estimated_out_len + self.frame_size);
        let mut window = Vec::with_capacity(self.frame_size);
        for i in 0..self.frame_size {
            // Hann window for smooth overlap-add
            let w = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / (self.frame_size - 1) as f64).cos());
            window.push(w as f32);
        }

        // Initialize output with the first frame
        let mut out_accum = vec![0.0f32; estimated_out_len + self.frame_size * 2];
        let mut weight_accum = vec![0.0f32; estimated_out_len + self.frame_size * 2];

        // Seed with first frame
        for i in 0..self.frame_size {
            let s = input[i] as f32 * window[i];
            out_accum[i] += s;
            weight_accum[i] += window[i];
        }

        let mut out_pos = hop_out;
        let mut in_pos = hop_in;

        while in_pos + self.frame_size + self.search_window < n_in && out_pos + self.frame_size < out_accum.len() {
            // Find best matching offset within [-search_window, +search_window]
            let search_start = in_pos.saturating_sub(self.search_window);
            let search_end = (in_pos + self.search_window).min(n_in - self.frame_size);

            let mut best_offset = in_pos;
            let mut best_corr = f64::MIN;

            // Target template is the previously synthesized overlap segment
            let template_start = out_pos;
            let compare_len = self.overlap.min(self.frame_size);

            for candidate in search_start..=search_end {
                let mut dot_prod = 0.0f64;
                let mut norm_cand = 0.0f64;

                for k in 0..compare_len {
                    let s_prev = out_accum[template_start + k] as f64;
                    let s_cand = input[candidate + k] as f64;
                    dot_prod += s_prev * s_cand;
                    norm_cand += s_cand * s_cand;
                }

                let norm = (norm_cand.max(1e-6)).sqrt();
                let score = dot_prod / norm;

                if score > best_corr {
                    best_corr = score;
                    best_offset = candidate;
                }
            }

            // Overlap-add the best matching frame
            for i in 0..self.frame_size {
                let sample = input[best_offset + i] as f32 * window[i];
                out_accum[out_pos + i] += sample;
                weight_accum[out_pos + i] += window[i];
            }

            out_pos += hop_out;
            in_pos += hop_in;
        }

        // Normalize by accumulated window weights and convert back to i16
        let final_len = if exact_target_len > 0 {
            exact_target_len
        } else {
            out_pos.min(estimated_out_len)
        };

        for i in 0..final_len {
            if i < out_accum.len() {
                let w = weight_accum[i];
                let val = if w > 1e-4 {
                    out_accum[i] / w
                } else {
                    out_accum[i]
                };
                output.push(val.clamp(i16::MIN as f32, i16::MAX as f32) as i16);
            } else {
                output.push(0);
            }
        }

        output
    }

    fn linear_resample(input: &[i16], target_len: usize) -> Vec<i16> {
        if input.is_empty() || target_len == 0 {
            return Vec::new();
        }
        let mut output = Vec::with_capacity(target_len);
        let scale = (input.len() - 1) as f64 / (target_len - 1).max(1) as f64;
        for i in 0..target_len {
            let src_pos = i as f64 * scale;
            let idx0 = src_pos.floor() as usize;
            let idx1 = (idx0 + 1).min(input.len() - 1);
            let frac = (src_pos - idx0 as f64) as f32;
            let s0 = input[idx0] as f32;
            let s1 = input[idx1] as f32;
            let sample = s0 + frac * (s1 - s0);
            output.push(sample.clamp(i16::MIN as f32, i16::MAX as f32) as i16);
        }
        output
    }
}
