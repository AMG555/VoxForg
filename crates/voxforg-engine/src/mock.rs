use async_trait::async_trait;
use tokio::sync::mpsc;
use voxforg_core::error::Result;
use voxforg_core::models::{AudioChunk, Gender, Voice};

use crate::traits::{SynthesisRequest, TtsEngine};

pub struct MockTtsEngine {
    sample_rate: u32,
}

impl MockTtsEngine {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }
}

impl Default for MockTtsEngine {
    fn default() -> Self {
        Self::new(24000)
    }
}

#[async_trait]
impl TtsEngine for MockTtsEngine {
    fn id(&self) -> &'static str {
        "mock-tts"
    }

    fn name(&self) -> &'static str {
        "Mock Synthesis Engine"
    }

    fn is_local(&self) -> bool {
        true
    }

    async fn voices(&self) -> Result<Vec<Voice>> {
        Ok(vec![
            Voice {
                id: "mock-en-male".to_string(),
                name: "Mock Male Voice".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Male,
                sample_rate_hz: self.sample_rate,
                tags: vec!["test".to_string(), "offline".to_string()],
                description: Some("Deterministic synthetic voice for automated tests".to_string()),
            },
            Voice {
                id: "mock-en-female".to_string(),
                name: "Mock Female Voice".to_string(),
                engine_id: self.id().to_string(),
                language: "en-US".to_string(),
                gender: Gender::Female,
                sample_rate_hz: self.sample_rate,
                tags: vec!["test".to_string(), "offline".to_string()],
                description: Some("Deterministic synthetic voice for automated tests".to_string()),
            },
        ])
    }

    async fn synthesize(&self, request: &SynthesisRequest) -> Result<AudioChunk> {
        // Generate audio proportional to text length (approx 150 words per minute => ~10 chars per second)
        let duration_secs = (request.text.len() as f32 * 0.05).max(0.1);
        let num_samples = ((self.sample_rate as f32) * duration_secs) as usize;

        let safe_pitch = if request.pitch.is_finite() {
            request.pitch.clamp(-24.0, 24.0)
        } else {
            0.0
        };

        // 440Hz base tone modulated by pitch
        let freq = 440.0 * 2.0f32.powf(safe_pitch / 12.0);
        let pcm_data: Vec<i16> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / self.sample_rate as f32;
                let envelope = (1.0 - (i as f32 / num_samples as f32))
                    .min(i as f32 / 500.0)
                    .clamp(0.0, 1.0);
                (f32::sin(2.0 * std::f32::consts::PI * freq * t) * 12000.0 * envelope) as i16
            })
            .collect();

        Ok(AudioChunk {
            sample_rate: self.sample_rate,
            channels: 1,
            pcm_data,
            is_final: true,
        })
    }

    async fn synthesize_stream(
        &self,
        request: &SynthesisRequest,
    ) -> Result<mpsc::Receiver<Result<AudioChunk>>> {
        let (tx, rx) = mpsc::channel(4);
        let full_chunk = self.synthesize(request).await?;

        tokio::spawn(async move {
            let chunk_size = 2400; // 100ms chunks
            let mut offset = 0;
            let total = full_chunk.pcm_data.len();

            while offset < total {
                let end = (offset + chunk_size).min(total);
                let is_final = end >= total;
                let slice = full_chunk.pcm_data[offset..end].to_vec();

                let chunk = AudioChunk {
                    sample_rate: full_chunk.sample_rate,
                    channels: full_chunk.channels,
                    pcm_data: slice,
                    is_final,
                };

                if tx.send(Ok(chunk)).await.is_err() {
                    break;
                }
                offset = end;
            }
        });

        Ok(rx)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}
