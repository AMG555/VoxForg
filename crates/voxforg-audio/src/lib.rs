pub mod dsp;
pub mod merge;
pub mod metrics;
pub mod normalizer;
pub mod wav;

pub use dsp::{BrickwallLimiter, DynamicCompressor, ParametricEq, SilenceTrimmer};
pub use merge::AudioMerger;
pub use metrics::{AudioAnalyzer, AudioQualityMetrics};
pub use normalizer::AudioNormalizer;
pub use wav::WavEncoder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_roundtrip() {
        let sample_rate = 24000;
        let channels = 1;
        let num_samples = (sample_rate as f32 * 0.1) as usize;
        let pcm: Vec<i16> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (f32::sin(2.0 * std::f32::consts::PI * 440.0 * t) * 16000.0) as i16
            })
            .collect();

        let wav_bytes = WavEncoder::encode_pcm16_to_wav(&pcm, sample_rate, channels)
            .expect("WAV encoding must succeed");
        assert!(!wav_bytes.is_empty());
        assert_eq!(&wav_bytes[0..4], b"RIFF");

        let (decoded_pcm, dec_rate, dec_channels) =
            WavEncoder::decode_wav_to_pcm16(&wav_bytes).expect("WAV decoding must succeed");
        assert_eq!(dec_rate, sample_rate);
        assert_eq!(dec_channels, channels);
        assert_eq!(decoded_pcm, pcm);
    }

    #[test]
    fn test_audio_normalizer_peak() {
        let mut samples = vec![1000i16, 2000, -4000, 3000];
        AudioNormalizer::peak_normalize(&mut samples, 1.0);
        let max_abs = samples.iter().map(|&s| (s as i32).abs()).max().unwrap();
        assert_eq!(max_abs, 32767);
    }

    #[test]
    fn test_audio_normalizer_i16_min_no_overflow() {
        let mut samples = vec![i16::MIN, 0, 1000];
        AudioNormalizer::peak_normalize(&mut samples, 0.5);
        assert_eq!(samples[0], -16384);
    }

    #[test]
    fn test_audio_normalizer_silent_handling() {
        let mut samples = vec![0i16, 0, 0];
        AudioNormalizer::peak_normalize(&mut samples, 0.95);
        assert_eq!(samples, vec![0, 0, 0]);
    }

    #[test]
    fn test_audio_normalizer_gain_db() {
        let mut samples = vec![1000i16, 2000];
        AudioNormalizer::apply_gain_db(&mut samples, 6.02); // ~2x gain
        assert!((samples[0] - 2000).abs() <= 10);
        assert!((samples[1] - 4000).abs() <= 10);
    }

    #[test]
    fn test_audio_merger_concatenation() {
        let seg1 = vec![100i16; 480]; // 10ms at 48kHz
        let seg2 = vec![200i16; 480];
        let merged = AudioMerger::concatenate_with_pause(&[&seg1, &seg2], 48000, 10);
        assert_eq!(merged.len(), 1440);
        assert_eq!(merged[0], 100);
        assert_eq!(merged[480], 0);
        assert_eq!(merged[960], 200);
    }

    #[test]
    fn test_audio_merger_huge_pause_bounded() {
        let seg1 = vec![100i16; 48];
        let seg2 = vec![200i16; 48];
        // Request astronomical pause - must be safely clamped to 30s max without OOM
        let merged = AudioMerger::concatenate_with_pause(&[&seg1, &seg2], 1000, u32::MAX);
        assert_eq!(merged.len(), 48 + 30000 + 48);
    }

    #[test]
    fn test_audio_merger_crossfade() {
        let seg_a = vec![1000i16; 480];
        let seg_b = vec![2000i16; 480];
        let crossfaded = AudioMerger::crossfade(&seg_a, &seg_b, 48000, 2); // 2ms crossfade = 96 samples
        assert!(crossfaded.len() < seg_a.len() + seg_b.len());
    }

    #[test]
    fn test_audio_metrics_analysis() {
        let sample_rate = 24000;
        let channels = 1;
        let num_samples = 24000; // 1 second
        let mut samples = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            let val = (f32::sin(2.0 * std::f32::consts::PI * 1000.0 * t) * 16384.0) as i16;
            samples.push(val);
        }

        let metrics = AudioAnalyzer::analyze_pcm16(&samples, sample_rate, channels);
        assert_eq!(metrics.duration_seconds, 1.0);
        assert_eq!(metrics.sample_rate, 24000);
        assert!(!metrics.is_silent);
        assert_eq!(metrics.clipping_samples_count, 0);
        assert!(metrics.peak_dbfs < 0.0 && metrics.peak_dbfs > -7.0);
        assert!(metrics.rms_dbfs < metrics.peak_dbfs);
    }

    #[test]
    fn test_audio_metrics_clipping_detection() {
        let samples = vec![32767i16, -32768, 1000, 0];
        let metrics = AudioAnalyzer::analyze_pcm16(&samples, 16000, 1);
        assert_eq!(metrics.clipping_samples_count, 2);
    }

    #[test]
    fn test_audio_metrics_empty() {
        let metrics = AudioAnalyzer::analyze_pcm16(&[], 24000, 1);
        assert_eq!(metrics.duration_seconds, 0.0);
        assert!(metrics.is_silent);
    }

    #[test]
    fn test_silence_trimmer() {
        // [silence, audio, silence]
        let mut samples = vec![0i16; 500];
        samples.extend(vec![15000i16; 1000]);
        samples.extend(vec![0i16; 500]);

        let trimmed = SilenceTrimmer::trim(&samples, 24000, -40.0, 10);
        assert!(trimmed.len() < samples.len());
        assert!(trimmed.len() >= 1000);
    }

    #[test]
    fn test_parametric_eq() {
        let mut samples = vec![5000i16; 2400];
        let original = samples.clone();
        ParametricEq::process_3band(&mut samples, 24000, 6.0, -3.0, 4.0);
        assert_ne!(samples, original);
    }

    #[test]
    fn test_dynamic_compressor() {
        // High amplitude signal
        let mut samples = vec![30000i16; 1200];
        DynamicCompressor::process(&mut samples, 24000, -20.0, 4.0, 5.0, 50.0, 0.0);
        let tail_amp = (samples[samples.len() - 1] as i32).abs();
        // After attack phase, compressor should have significantly reduced amplitude
        assert!(tail_amp < 15000);
    }

    #[test]
    fn test_brickwall_limiter() {
        let mut samples = vec![32767i16, -32768, 15000, -15000];
        // Limit to -6 dBFS (~16422 peak amplitude)
        BrickwallLimiter::process(&mut samples, -6.0);
        let max_amp = samples.iter().map(|&s| (s as i32).abs()).max().unwrap();
        assert!(max_amp <= 16500);
        assert!(max_amp > 16000);
    }
}
