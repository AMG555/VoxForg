pub mod edge_tts;
pub mod mock;
pub mod registry;
pub mod traits;

pub use edge_tts::EdgeTtsEngine;
pub use mock::MockTtsEngine;
pub use registry::EngineRegistry;
pub use traits::{SynthesisRequest, TtsEngine};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use voxforg_core::models::AudioContainerFormat;

    #[tokio::test]
    async fn test_mock_engine_synthesis() {
        let engine = MockTtsEngine::new(24000);
        let req = SynthesisRequest {
            text: "Testing speech synthesis engine".to_string(),
            voice_id: "mock-en-female".to_string(),
            speed: 1.0,
            pitch: 0.0,
            format: AudioContainerFormat::Wav,
        };

        let chunk = engine.synthesize(&req).await.expect("Synthesis must succeed");
        assert_eq!(chunk.sample_rate, 24000);
        assert!(!chunk.pcm_data.is_empty());
        assert!(chunk.is_final);
    }

    #[tokio::test]
    async fn test_registry_resolution() {
        let registry = EngineRegistry::new();
        let mock = Arc::new(MockTtsEngine::new(24000));
        let edge = Arc::new(EdgeTtsEngine::new());

        registry.register(mock).await;
        registry.register(edge).await;

        let engines = registry.list_engines().await;
        assert_eq!(engines.len(), 2);

        let (resolved_engine, voice) = registry
            .resolve_voice("en-US-AriaNeural")
            .await
            .expect("Should resolve AriaNeural to EdgeTTS");
        assert_eq!(resolved_engine.id(), "edge-tts");
        assert_eq!(voice.name, "Aria (Neural)");

        let not_found = registry.resolve_voice("non-existent-voice").await;
        assert!(not_found.is_err());
    }
}
