use serde::{Deserialize, Serialize};

/// Fixed 16-bit signature payload identifying VoxForg as the origin: "VX" (0x5658)
pub const DEFAULT_SIGNATURE_PAYLOAD: u16 = 0x5658;

/// Audio provenance watermark detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkDetectionResult {
    /// True if the correlation confidence exceeds threshold
    pub is_detected: bool,
    /// Confidence metric from 0.0 to 1.0
    pub confidence: f32,
    /// Extracted 16-bit payload if detected
    pub payload: Option<u16>,
    /// True if extracted payload matches the VoxForg signature
    pub signature_match: bool,
    /// Sample rate processed
    pub sample_rate: u32,
    /// Duration of audio analyzed in seconds
    pub duration_seconds: f32,
}

/// Invisible Spread-Spectrum Audio Watermark Embedder and Detector
#[derive(Debug, Clone)]
pub struct AudioWatermark {
    sample_rate: u32,
    chip_rate: usize,
    carrier_seed: u64,
    embedding_strength: f32,
    detection_threshold: f32,
}

impl AudioWatermark {
    /// Creates a new watermark processor with default parameters
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            chip_rate: 128,               // Samples per bit chip
            carrier_seed: 0x9E3779B97F4A7C15, // Golden ratio constant
            embedding_strength: 0.0035,   // ~ -49 dBFS, imperceptible to human ear
            detection_threshold: 0.45,    // Minimum normalized correlation for detection
        }
    }

    /// Sets the embedding strength (recommended range: 0.001 to 0.01)
    pub fn with_strength(mut self, strength: f32) -> Self {
        self.embedding_strength = strength.clamp(0.0005, 0.02);
        self
    }

    /// Generate pseudo-random carrier sequence for a given bit index
    fn generate_chip_carrier(&self, bit_index: usize, length: usize) -> Vec<f32> {
        let mut seed = self.carrier_seed.wrapping_add((bit_index as u64).wrapping_mul(0x517cc1b727220a95));
        let mut carrier = Vec::with_capacity(length);
        for _ in 0..length {
            // Xorshift64 PRNG
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            // Bipolar {-1.0, 1.0}
            let bit = (seed & 1) == 1;
            carrier.push(if bit { 1.0 } else { -1.0 });
        }
        carrier
    }

    /// Embeds a 16-bit payload into audio samples in-place
    ///
    /// Loops the 16-bit message across the entire audio length for high redundancy
    pub fn embed(&self, samples: &mut [f32], payload: u16) -> usize {
        let bit_count = 16;
        let bits: Vec<bool> = (0..bit_count).map(|i| ((payload >> (15 - i)) & 1) == 1).collect();
        let total_samples = samples.len();
        let bits_total = total_samples / self.chip_rate;

        if bits_total == 0 {
            return 0;
        }

        for (b_idx, chunk) in samples.chunks_mut(self.chip_rate).enumerate() {
            let bit_val = bits[b_idx % bit_count];
            let sign = if bit_val { 1.0 } else { -1.0 };
            let carrier = self.generate_chip_carrier(b_idx % bit_count, chunk.len());

            for (sample, &c) in chunk.iter_mut().zip(carrier.iter()) {
                // Additive spread-spectrum watermark
                *sample += sign * c * self.embedding_strength;
                // Soft clip to [-1.0, 1.0] to prevent overflow
                *sample = sample.clamp(-1.0, 1.0);
            }
        }

        bits_total / bit_count
    }

    /// Detects and extracts a watermark payload from audio samples
    pub fn detect(&self, samples: &[f32]) -> WatermarkDetectionResult {
        let duration_seconds = samples.len() as f32 / self.sample_rate as f32;
        let bit_count = 16;
        let total_samples = samples.len();
        let total_chips = total_samples / self.chip_rate;

        if total_chips < bit_count {
            return WatermarkDetectionResult {
                is_detected: false,
                confidence: 0.0,
                payload: None,
                signature_match: false,
                sample_rate: self.sample_rate,
                duration_seconds,
            };
        }

        let mut bit_correlations = vec![0.0f32; bit_count];
        let mut bit_counts = vec![0usize; bit_count];

        for (b_idx, chunk) in samples.chunks(self.chip_rate).enumerate() {
            let slot = b_idx % bit_count;
            let carrier = self.generate_chip_carrier(slot, chunk.len());

            let mut corr = 0.0f32;
            for (&s, &c) in chunk.iter().zip(carrier.iter()) {
                corr += s * c;
            }
            if !chunk.is_empty() {
                corr /= chunk.len() as f32;
            }

            bit_correlations[slot] += corr;
            bit_counts[slot] += 1;
        }

        // Average correlation per bit slot
        let mut extracted_payload: u16 = 0;
        let mut avg_confidence = 0.0f32;

        for slot in 0..bit_count {
            if bit_counts[slot] > 0 {
                let mean_corr = bit_correlations[slot] / bit_counts[slot] as f32;
                // Normalized relative to embedding strength
                let norm_val = mean_corr / self.embedding_strength;
                if norm_val > 0.0 {
                    extracted_payload |= 1 << (15 - slot);
                }
                avg_confidence += norm_val.abs();
            }
        }

        avg_confidence = (avg_confidence / bit_count as f32).clamp(0.0, 1.0);
        let is_detected = avg_confidence >= self.detection_threshold;
        let signature_match = is_detected && (extracted_payload == DEFAULT_SIGNATURE_PAYLOAD);

        WatermarkDetectionResult {
            is_detected,
            confidence: avg_confidence,
            payload: if is_detected { Some(extracted_payload) } else { None },
            signature_match,
            sample_rate: self.sample_rate,
            duration_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_embed_and_detect() {
        let sample_rate = 24000;
        let mut audio = vec![0.0f32; sample_rate as usize * 3]; // 3 seconds of audio

        // Fill with sine wave
        for (i, sample) in audio.iter_mut().enumerate() {
            let t = i as f32 / sample_rate as f32;
            *sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 0.5;
        }

        let wm = AudioWatermark::new(sample_rate);
        
        // Before embedding: should not be detected
        let pre_result = wm.detect(&audio);
        assert!(!pre_result.is_detected || pre_result.confidence < 0.3);

        // Embed signature payload
        let repetitions = wm.embed(&mut audio, DEFAULT_SIGNATURE_PAYLOAD);
        assert!(repetitions > 0);

        // After embedding: should be detected with signature match
        let post_result = wm.detect(&audio);
        assert!(post_result.is_detected);
        assert!(post_result.signature_match);
        assert_eq!(post_result.payload, Some(DEFAULT_SIGNATURE_PAYLOAD));
        assert!(post_result.confidence > 0.45);
    }
}
