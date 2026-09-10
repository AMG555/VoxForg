pub mod ab_test;
pub mod edge_tts;
pub mod mock;
pub mod registry;
pub mod traits;

pub use ab_test::{AbTestComparison, AbTestRunner, AbTestScenario, VariantResult};
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
    async fn test_mock_engine_streaming() {
        let engine = MockTtsEngine::new(24000);
        let req = SynthesisRequest {
            text: "Long sentence testing streaming audio generation across multiple chunk boundaries."
                .to_string(),
            voice_id: "mock-en-male".to_string(),
            speed: 1.0,
            pitch: 0.0,
            format: AudioContainerFormat::Wav,
        };

        let mut rx = engine
            .synthesize_stream(&req)
            .await
            .expect("Stream synthesis must succeed");

        let mut chunks = Vec::new();
        while let Some(res) = rx.recv().await {
            let chunk = res.expect("Chunk must be Ok");
            chunks.push(chunk);
        }

        assert!(!chunks.is_empty());
        assert!(chunks.last().unwrap().is_final);
    }

    #[tokio::test]
    async fn test_edge_tts_ssml_generation() {
        let req = SynthesisRequest {
            text: "Hello <script>alert('xss')</script> & world".to_string(),
            voice_id: "en-US-AriaNeural".to_string(),
            speed: 1.25,
            pitch: 2.0,
            format: AudioContainerFormat::Wav,
        };

        let ssml = EdgeTtsEngine::build_ssml(&req);
        assert!(ssml.contains("&lt;script&gt;"));
        assert!(ssml.contains("&amp;"));
        assert!(ssml.contains("voice name='Microsoft Server Speech Text to Speech Voice (en-US, AriaNeural)'"));
        assert!(ssml.contains("rate='+25%'"));
    }

    #[tokio::test]
    async fn test_edge_tts_ssml_injection_prevention() {
        let req = SynthesisRequest {
            text: "Safe text".to_string(),
            voice_id: "en-US-AriaNeural' extra='attr".to_string(),
            speed: 1.0,
            pitch: 0.0,
            format: AudioContainerFormat::Wav,
        };

        let ssml = EdgeTtsEngine::build_ssml(&req);
        assert!(!ssml.contains("AriaNeural' extra"));
        assert!(ssml.contains("AriaNeural&apos; extra"));
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

    #[tokio::test]
    async fn test_ab_test_runner_comparison() {
        let registry = Arc::new(EngineRegistry::new());
        let mock = Arc::new(MockTtsEngine::new(24000));
        let edge = Arc::new(EdgeTtsEngine::new());

        registry.register(mock).await;
        registry.register(edge).await;

        let runner = AbTestRunner::new(registry);
        let scenario = AbTestScenario {
            name: "Model Comparison Benchmark".to_string(),
            text: "Audio QA and comparative synthesis quality test.".to_string(),
            variant_a: SynthesisRequest {
                text: "".to_string(),
                voice_id: "mock-en-female".to_string(),
                speed: 1.0,
                pitch: 0.0,
                format: AudioContainerFormat::Wav,
            },
            variant_b: SynthesisRequest {
                text: "".to_string(),
                voice_id: "en-US-AriaNeural".to_string(),
                speed: 1.1,
                pitch: 1.0,
                format: AudioContainerFormat::Wav,
            },
        };

        let result = runner.run_comparison(&scenario).await.expect("A/B comparison should succeed");
        assert_eq!(result.scenario_name, "Model Comparison Benchmark");
        assert_eq!(result.variant_a.voice_id, "mock-en-female");
        assert_eq!(result.variant_b.voice_id, "en-US-AriaNeural");
        assert!(result.variant_a.audio_duration_seconds > 0.0);
        assert!(result.variant_b.audio_duration_seconds > 0.0);
        assert!(!result.recommended_variant.is_empty());
        assert!(!result.summary.is_empty());
    }

    #[tokio::test]
    async fn test_ab_test_batch_runner() {
        let registry = Arc::new(EngineRegistry::new());
        let mock = Arc::new(MockTtsEngine::new(24000));
        registry.register(mock).await;

        let runner = AbTestRunner::new(registry);
        let scenarios = vec![
            AbTestScenario {
                name: "Batch Item 1".to_string(),
                text: "First test line".to_string(),
                variant_a: SynthesisRequest {
                    text: "".to_string(),
                    voice_id: "mock-en-female".to_string(),
                    speed: 1.0,
                    pitch: 0.0,
                    format: AudioContainerFormat::Wav,
                },
                variant_b: SynthesisRequest {
                    text: "".to_string(),
                    voice_id: "mock-en-male".to_string(),
                    speed: 1.0,
                    pitch: 0.0,
                    format: AudioContainerFormat::Wav,
                },
            },
            AbTestScenario {
                name: "Batch Item 2".to_string(),
                text: "Second test line".to_string(),
                variant_a: SynthesisRequest {
                    text: "".to_string(),
                    voice_id: "mock-en-female".to_string(),
                    speed: 1.2,
                    pitch: 2.0,
                    format: AudioContainerFormat::Wav,
                },
                variant_b: SynthesisRequest {
                    text: "".to_string(),
                    voice_id: "mock-en-male".to_string(),
                    speed: 0.8,
                    pitch: -2.0,
                    format: AudioContainerFormat::Wav,
                },
            },
        ];

        let results = runner.run_batch(&scenarios).await.expect("Batch run must succeed");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].scenario_name, "Batch Item 1");
        assert_eq!(results[1].scenario_name, "Batch Item 2");
    }

    #[test]
    fn test_sec_ms_gec_token_calculation() {
        use crate::edge_tts::generate_sec_ms_gec;

        let token = generate_sec_ms_gec();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase()));
    }

    #[test]
    fn test_binary_frame_header_parsing() {
        use crate::edge_tts::parse_binary_audio_payload;

        // Too short frame
        assert!(parse_binary_audio_payload(&[0, 1]).is_none());

        // Frame with header not containing Path:audio
        let bad_header = "Path:metadata\r\n";
        let bad_header_len = (bad_header.len() as u16).to_be_bytes();
        let mut bad_frame = Vec::new();
        bad_frame.extend_from_slice(&bad_header_len);
        bad_frame.extend_from_slice(bad_header.as_bytes());
        bad_frame.extend_from_slice(&[0, 0, 1, 1]);
        assert!(parse_binary_audio_payload(&bad_frame).is_none());

        // Frame with valid Path:audio and payload
        let good_header = "X-RequestId:abc\r\nPath:audio\r\n";
        let good_header_len = (good_header.len() as u16).to_be_bytes();
        let mut good_frame = Vec::new();
        good_frame.extend_from_slice(&good_header_len);
        good_frame.extend_from_slice(good_header.as_bytes());
        good_frame.extend_from_slice(&[0x49, 0x44, 0x33, 0x04]);

        let payload = parse_binary_audio_payload(&good_frame).expect("Should parse audio payload");
        assert_eq!(payload, &[0x49, 0x44, 0x33, 0x04]);
    }

    #[tokio::test]
    async fn test_edge_tts_voices_catalog() {
        let engine = EdgeTtsEngine::new();
        let voices = engine.voices().await.expect("Voices must load");
        assert!(voices.len() >= 10);
        assert!(voices.iter().any(|v| v.id == "en-US-AriaNeural"));
        assert!(voices.iter().any(|v| v.id == "en-US-GuyNeural"));
        assert!(voices.iter().any(|v| v.id == "ja-JP-NanamiNeural"));
        assert!(voices.iter().any(|v| v.id == "zh-CN-XiaoxiaoNeural"));
    }

    #[tokio::test]
    async fn test_edge_tts_synthesize_execution() {
        let engine = EdgeTtsEngine::new();
        let req = SynthesisRequest {
            text: "Hello from VoxForg neural engine testing".to_string(),
            voice_id: "en-US-AriaNeural".to_string(),
            speed: 1.0,
            pitch: 0.0,
            format: AudioContainerFormat::Wav,
        };

        let chunk = engine.synthesize(&req).await.expect("Synthesize must succeed");
        assert_eq!(chunk.sample_rate, 24000);
        assert_eq!(chunk.channels, 1);
        assert!(!chunk.pcm_data.is_empty());
        assert!(chunk.is_final);
    }
}
