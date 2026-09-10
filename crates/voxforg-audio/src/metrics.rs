use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioQualityMetrics {
    pub duration_seconds: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub total_samples: usize,
    pub peak_amplitude: i16,
    pub peak_dbfs: f32,
    pub rms_amplitude: f32,
    pub rms_dbfs: f32,
    pub clipping_samples_count: usize,
    pub is_silent: bool,
}

pub struct AudioAnalyzer;

impl AudioAnalyzer {
    pub fn analyze_pcm16(samples: &[i16], sample_rate: u32, channels: u16) -> AudioQualityMetrics {
        if samples.is_empty() {
            return AudioQualityMetrics {
                duration_seconds: 0.0,
                sample_rate,
                channels,
                total_samples: 0,
                peak_amplitude: 0,
                peak_dbfs: -96.0,
                rms_amplitude: 0.0,
                rms_dbfs: -96.0,
                clipping_samples_count: 0,
                is_silent: true,
            };
        }

        let total_samples = samples.len();
        let safe_sample_rate = sample_rate.max(1);
        let safe_channels = channels.max(1);
        let duration_seconds =
            total_samples as f64 / (safe_sample_rate as f64 * safe_channels as f64);

        let mut max_abs: i16 = 0;
        let mut sum_squares: f64 = 0.0;
        let mut clipping_count = 0;

        for &sample in samples {
            let abs_val = sample.saturating_abs();
            if abs_val > max_abs {
                max_abs = abs_val;
            }
            if sample == i16::MAX || sample == i16::MIN {
                clipping_count += 1;
            }
            let norm = sample as f64 / 32768.0;
            sum_squares += norm * norm;
        }

        let rms_norm = (sum_squares / total_samples as f64).sqrt() as f32;
        let peak_norm = max_abs as f32 / 32768.0;

        let rms_dbfs = if rms_norm > 1e-5 {
            20.0 * rms_norm.log10()
        } else {
            -96.0
        };

        let peak_dbfs = if peak_norm > 1e-5 {
            20.0 * peak_norm.log10()
        } else {
            -96.0
        };

        AudioQualityMetrics {
            duration_seconds,
            sample_rate,
            channels,
            total_samples,
            peak_amplitude: max_abs,
            peak_dbfs,
            rms_amplitude: rms_norm,
            rms_dbfs,
            clipping_samples_count: clipping_count,
            is_silent: max_abs < 10,
        }
    }
}
