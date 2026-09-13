use async_trait::async_trait;
use voxforg_core::error::VoxForgError;

use crate::traits::{AsrEngine, AsrEngineInfo};
use crate::types::{
    TranscriptionOptions, TranscriptionResult, TranscriptionSegment, WordTimestamp,
};

/// Deterministic mock ASR engine for test suites, CI, and local emulation.
pub struct MockAsrEngine {
    id: String,
    name: String,
    fixed_transcript: Option<String>,
}

impl Default for MockAsrEngine {
    fn default() -> Self {
        Self::new("mock-asr", "VoxForg Mock ASR Engine")
    }
}

impl MockAsrEngine {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            fixed_transcript: None,
        }
    }

    pub fn with_fixed_transcript(mut self, transcript: impl Into<String>) -> Self {
        self.fixed_transcript = Some(transcript.into());
        self
    }
}

#[async_trait]
impl AsrEngine for MockAsrEngine {
    fn info(&self) -> AsrEngineInfo {
        AsrEngineInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: "Deterministic test ASR engine with synthetic word alignment".to_string(),
            supported_languages: vec![
                "en".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "de".to_string(),
            ],
            is_local: true,
            sample_rate: 16000,
        }
    }

    async fn transcribe(
        &self,
        audio_pcm: &[i16],
        sample_rate: u32,
        options: &TranscriptionOptions,
    ) -> Result<TranscriptionResult, VoxForgError> {
        let sample_rate = if sample_rate == 0 { 16000 } else { sample_rate };
        let duration_seconds = audio_pcm.len() as f32 / sample_rate as f32;
        let total_ms = (duration_seconds * 1000.0) as u64;

        if audio_pcm.is_empty() {
            return Ok(TranscriptionResult {
                text: String::new(),
                task: "transcribe".to_string(),
                language: options.language.clone().unwrap_or_else(|| "en".to_string()),
                duration_seconds: 0.0,
                segments: Vec::new(),
                words: Vec::new(),
            });
        }

        let raw_text = self.fixed_transcript.clone().unwrap_or_else(|| {
            if let Some(prompt) = &options.prompt {
                format!("Transcribed: {prompt}")
            } else {
                "The quick brown fox jumps over the lazy dog.".to_string()
            }
        });

        let words_raw: Vec<&str> = raw_text.split_whitespace().collect();
        let word_count = words_raw.len().max(1);
        let time_per_word = (total_ms / word_count as u64).max(50);

        let mut words = Vec::new();
        for (i, w) in words_raw.iter().enumerate() {
            let start = i as u64 * time_per_word;
            let end = ((i as u64 + 1) * time_per_word).min(total_ms.max(start + 50));
            words.push(WordTimestamp {
                word: w.to_string(),
                start_ms: start,
                end_ms: end,
                probability: 0.98,
            });
        }

        let segment = TranscriptionSegment {
            id: 0,
            seek: 0,
            start_ms: 0,
            end_ms: total_ms.max(50),
            text: raw_text.clone(),
            tokens: vec![50364, 100, 200, 300, 50424],
            temperature: options.temperature.unwrap_or(0.0),
            avg_logprob: -0.10,
            compression_ratio: 1.05,
            no_speech_prob: 0.001,
            speaker: Some("SPEAKER_00".to_string()),
            words: if options.word_timestamps {
                words.clone()
            } else {
                Vec::new()
            },
        };

        Ok(TranscriptionResult {
            text: raw_text,
            task: "transcribe".to_string(),
            language: options.language.clone().unwrap_or_else(|| "en".to_string()),
            duration_seconds,
            segments: vec![segment],
            words: if options.word_timestamps {
                words
            } else {
                Vec::new()
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_asr_transcribe() {
        let engine = MockAsrEngine::default();
        let fake_pcm: Vec<i16> = (0..16000).map(|i| ((i % 100) * 50) as i16).collect();
        let opts = TranscriptionOptions {
            word_timestamps: true,
            ..Default::default()
        };

        let res = engine.transcribe(&fake_pcm, 16000, &opts).await.unwrap();
        assert_eq!(res.duration_seconds, 1.0);
        assert!(!res.text.is_empty());
        assert_eq!(res.segments.len(), 1);
        assert!(!res.words.is_empty());
    }
}
