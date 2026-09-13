use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use voxforg_core::models::{Gender, VoiceProfile};

/// In-memory thread-safe store for cloned and persistent voice profiles.
#[derive(Clone, Default)]
pub struct VoiceProfileStore {
    profiles: Arc<RwLock<HashMap<String, VoiceProfile>>>,
}

impl VoiceProfileStore {
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Construct store populated with default base profiles.
    pub fn with_defaults() -> Self {
        let mut map = HashMap::new();

        let p1 = VoiceProfile {
            id: "qwen3-base-female".to_string(),
            name: "Qwen3 Base Female".to_string(),
            engine_id: "qwen3-tts".to_string(),
            description: Some("Base conversational female neural profile".to_string()),
            language: "en-US".to_string(),
            gender: Some(Gender::Female),
            reference_audio_path: None,
            reference_audio_base64: None,
            reference_transcript: None,
            embedding: Some(vec![0.05f32; 512]),
            clone_capabilities: Some(vec!["zero-shot".to_string()]),
            metadata: HashMap::new(),
            created_at: Utc::now(),
        };

        let p2 = VoiceProfile {
            id: "qwen3-base-male".to_string(),
            name: "Qwen3 Base Male".to_string(),
            engine_id: "qwen3-tts".to_string(),
            description: Some("Base studio narrator male neural profile".to_string()),
            language: "en-US".to_string(),
            gender: Some(Gender::Male),
            reference_audio_path: None,
            reference_audio_base64: None,
            reference_transcript: None,
            embedding: Some(vec![-0.05f32; 512]),
            clone_capabilities: Some(vec!["zero-shot".to_string()]),
            metadata: HashMap::new(),
            created_at: Utc::now(),
        };

        map.insert(p1.id.clone(), p1);
        map.insert(p2.id.clone(), p2);

        Self {
            profiles: Arc::new(RwLock::new(map)),
        }
    }

    /// Insert or update a voice profile.
    pub async fn insert(&self, profile: VoiceProfile) {
        let mut lock = self.profiles.write().await;
        lock.insert(profile.id.clone(), profile);
    }

    /// Retrieve a voice profile by ID.
    pub async fn get(&self, id: &str) -> Option<VoiceProfile> {
        let lock = self.profiles.read().await;
        lock.get(id).cloned()
    }

    /// List all stored voice profiles sorted by ID.
    pub async fn list(&self) -> Vec<VoiceProfile> {
        let lock = self.profiles.read().await;
        let mut list: Vec<_> = lock.values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    /// Remove a voice profile by ID.
    pub async fn remove(&self, id: &str) -> Option<VoiceProfile> {
        let mut lock = self.profiles.write().await;
        lock.remove(id)
    }
}
