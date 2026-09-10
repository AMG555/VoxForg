pub mod error;
pub mod models;
pub mod store;

pub use error::{ProblemDetails, Result, VoxForgError};
pub use models::*;
pub use store::{memory::MemoryStore, DataStore};

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_memory_store_pipeline_crud() {
        let store = MemoryStore::new();
        let pipeline_id = Uuid::new_v4();

        let pipeline = PipelineDefinition {
            id: pipeline_id,
            name: "Test Pipeline".to_string(),
            description: Some("Audio pipeline test".to_string()),
            nodes: vec![PipelineNode {
                id: "node-1".to_string(),
                name: "Input".to_string(),
                node_type: NodeType::TextInput,
                params: serde_json::json!({ "text": "Hello world" }),
                position: None,
            }],
            edges: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        store
            .save_pipeline(&pipeline)
            .await
            .expect("Save pipeline should succeed");

        let fetched = store
            .get_pipeline(&pipeline_id)
            .await
            .expect("Fetch should succeed");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "Test Pipeline");

        let list = store.list_pipelines().await.expect("List should succeed");
        assert_eq!(list.len(), 1);

        let deleted = store
            .delete_pipeline(&pipeline_id)
            .await
            .expect("Delete should succeed");
        assert!(deleted);

        let delete_again = store
            .delete_pipeline(&pipeline_id)
            .await
            .expect("Delete again should succeed");
        assert!(!delete_again);

        let refetched = store
            .get_pipeline(&pipeline_id)
            .await
            .expect("Fetch should succeed");
        assert!(refetched.is_none());
    }

    #[tokio::test]
    async fn test_voice_storage_and_filter() {
        let store = MemoryStore::new();

        let voice1 = Voice {
            id: "en-US-AriaNeural".to_string(),
            name: "Aria".to_string(),
            engine_id: "edge-tts".to_string(),
            language: "en-US".to_string(),
            gender: Gender::Female,
            sample_rate_hz: 24000,
            tags: vec!["news".to_string()],
            description: None,
        };

        let voice2 = Voice {
            id: "de-DE-Thorsten".to_string(),
            name: "Thorsten".to_string(),
            engine_id: "piper".to_string(),
            language: "de-DE".to_string(),
            gender: Gender::Male,
            sample_rate_hz: 22050,
            tags: vec!["broadcast".to_string()],
            description: None,
        };

        store.save_voice(&voice1).await.unwrap();
        store.save_voice(&voice2).await.unwrap();

        let en_voices = store.list_voices(Some("en"), None).await.unwrap();
        assert_eq!(en_voices.len(), 1);
        assert_eq!(en_voices[0].id, "en-US-AriaNeural");

        let piper_voices = store.list_voices(None, Some("piper")).await.unwrap();
        assert_eq!(piper_voices.len(), 1);
        assert_eq!(piper_voices[0].id, "de-DE-Thorsten");

        let no_voices = store.list_voices(Some("ja"), None).await.unwrap();
        assert!(no_voices.is_empty());
    }

    #[test]
    fn test_audio_container_formats() {
        assert_eq!(AudioContainerFormat::Wav.mime_type(), "audio/wav");
        assert_eq!(AudioContainerFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(AudioContainerFormat::Opus.mime_type(), "audio/opus");
        assert_eq!(AudioContainerFormat::Pcm.mime_type(), "audio/pcm");
        assert_eq!(AudioContainerFormat::Flac.mime_type(), "audio/flac");
        assert_eq!(AudioContainerFormat::Aac.mime_type(), "audio/aac");
    }

    #[test]
    fn test_error_to_problem_details() {
        let err = VoxForgError::VoiceNotFound("test-voice".to_string());
        let details = err.to_problem_details("/v1/audio/speech");
        assert_eq!(details.status, 404);
        assert_eq!(details.title, "Voice Not Found");
        assert_eq!(details.instance, "/v1/audio/speech");
        assert!(details.detail.contains("test-voice"));

        let val_err = VoxForgError::PipelineValidation("Cycle detected".to_string());
        let val_details = val_err.to_problem_details("/v1/pipeline/execute");
        assert_eq!(val_details.status, 422);

        let unauth_err = VoxForgError::Unauthorized("Bad token".to_string());
        let unauth_details = unauth_err.to_problem_details("/health");
        assert_eq!(unauth_details.status, 401);
    }
}
