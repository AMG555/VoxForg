pub mod merge;
pub mod normalizer;
pub mod wav;

pub use merge::AudioMerger;
pub use normalizer::AudioNormalizer;
pub use wav::WavEncoder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_roundtrip() {
        let sample_rate = 24000;
        let channels = 1;
        // Generate a 100ms 440Hz sine tone
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

        let (decoded_pcm, dec_rate, dec_channels) = WavEncoder::decode_wav_to_pcm16(&wav_bytes)
            .expect("WAV decoding must succeed");
        assert_eq!(dec_rate, sample_rate);
        assert_eq!(dec_channels, channels);
        assert_eq!(decoded_pcm, pcm);
    }

    #[test]
    fn test_audio_normalizer() {
        let mut samples = vec![1000i16, 2000, -4000, 3000];
        AudioNormalizer::peak_normalize(&mut samples, 1.0);
        let max_abs = samples.iter().map(|&s| s.abs()).max().unwrap();
        assert_eq!(max_abs, 32767);
    }

    #[test]
    fn test_audio_merger_concatenation() {
        let seg1 = vec![100i16; 480]; // 10ms at 48kHz
        let seg2 = vec![200i16; 480];
        let merged = AudioMerger::concatenate_with_pause(&[&seg1, &seg2], 48000, 10);
        // 480 + 480 + (48000 * 0.01 = 480 pause) = 1440
        assert_eq!(merged.len(), 1440);
        assert_eq!(merged[0], 100);
        assert_eq!(merged[480], 0); // pause buffer
        assert_eq!(merged[960], 200);
    }
}
