use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;
use voxforg_core::models::{Gender, VoiceProfile};

/// In-memory and disk-backed thread-safe store for cloned and persistent voice profiles.
#[derive(Clone)]
pub struct VoiceProfileStore {
    profiles: Arc<RwLock<HashMap<String, VoiceProfile>>>,
    storage_dir: Option<PathBuf>,
}

impl Default for VoiceProfileStore {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl VoiceProfileStore {
    /// Create empty in-memory profile store.
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            storage_dir: None,
        }
    }

    /// Construct store populated with default base profiles, checking environment for persistent folder.
    pub fn with_defaults() -> Self {
        if let Ok(dir_str) = std::env::var("VOXFORG_PROFILES_DIR") {
            let path = PathBuf::from(dir_str.trim());
            if !path.as_os_str().is_empty() {
                return Self::with_storage_dir(path);
            }
        }
        Self::in_memory_with_defaults()
    }

    /// Construct in-memory store populated with base profiles without disk writing.
    pub fn in_memory_with_defaults() -> Self {
        let mut map = HashMap::new();
        Self::seed_default_profiles(&mut map);
        Self {
            profiles: Arc::new(RwLock::new(map)),
            storage_dir: None,
        }
    }

    /// Construct store backed by persistent directory on disk.
    /// Loads any existing `.json` profile files on initialization.
    pub fn with_storage_dir(dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&dir);
        let mut map = HashMap::new();
        Self::seed_default_profiles(&mut map);

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    // Guard against unbounded file read
                    if let Ok(meta) = entry.metadata() {
                        if meta.len() > 50 * 1024 * 1024 {
                            warn!("Skipping profile {:?} exceeding 50MB ceiling", path);
                            continue;
                        }
                    }
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        match serde_json::from_str::<VoiceProfile>(&content) {
                            Ok(profile) => {
                                map.insert(profile.id.clone(), profile);
                            }
                            Err(e) => {
                                warn!("Failed parsing voice profile from {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        Self {
            profiles: Arc::new(RwLock::new(map)),
            storage_dir: Some(dir),
        }
    }

    fn seed_default_profiles(map: &mut HashMap<String, VoiceProfile>) {
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
    }

    /// Insert or update a voice profile in memory and persist atomically to disk if directory is configured.
    pub async fn insert(&self, profile: VoiceProfile) {
        let id = profile.id.clone();
        {
            let mut lock = self.profiles.write().await;
            lock.insert(id.clone(), profile.clone());
        }

        if let Some(ref dir) = self.storage_dir {
            if let Some(safe_id) = Self::sanitize_id(&id) {
                let file_path = dir.join(format!("{}.json", safe_id));
                let tmp_path = dir.join(format!("{}.json.tmp", safe_id));
                if let Ok(json_str) = serde_json::to_string_pretty(&profile) {
                    if std::fs::write(&tmp_path, json_str).is_ok() {
                        if std::fs::rename(&tmp_path, &file_path).is_err() {
                            let _ = std::fs::copy(&tmp_path, &file_path);
                            let _ = std::fs::remove_file(&tmp_path);
                        }
                    }
                }
            }
        }
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

    /// Remove a voice profile by ID in memory and from disk if configured.
    pub async fn remove(&self, id: &str) -> Option<VoiceProfile> {
        let removed = {
            let mut lock = self.profiles.write().await;
            lock.remove(id)
        };

        if let Some(ref dir) = self.storage_dir {
            if let Some(safe_id) = Self::sanitize_id(id) {
                let file_path = dir.join(format!("{}.json", safe_id));
                if file_path.exists() {
                    let _ = std::fs::remove_file(file_path);
                }
            }
        }

        removed
    }

    /// Sanitize ID to prevent path traversal or unsafe filesystem characters.
    pub fn sanitize_id(id: &str) -> Option<String> {
        let trimmed = id.trim();
        if trimmed.is_empty() || trimmed.len() > 128 {
            return None;
        }
        let safe: String = trimmed
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        if safe.is_empty() || safe == "." || safe == ".." {
            None
        } else {
            Some(safe)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_voice_profile_store_in_memory() {
        let store = VoiceProfileStore::in_memory_with_defaults();
        let profiles = store.list().await;
        assert_eq!(profiles.len(), 2);
        assert!(store.get("qwen3-base-female").await.is_some());
    }

    #[tokio::test]
    async fn test_voice_profile_disk_persistence() {
        let temp_dir =
            std::env::temp_dir().join(format!("voxforg_test_profiles_{}", uuid::Uuid::new_v4()));
        let store = VoiceProfileStore::with_storage_dir(temp_dir.clone());

        let custom = VoiceProfile {
            id: "custom-voice-123".to_string(),
            name: "Custom Speaker".to_string(),
            engine_id: "qwen3-tts".to_string(),
            description: Some("Test custom speaker".to_string()),
            language: "en-US".to_string(),
            gender: Some(Gender::Neutral),
            reference_audio_path: None,
            reference_audio_base64: None,
            reference_transcript: None,
            embedding: Some(vec![0.1; 512]),
            clone_capabilities: Some(vec!["zero-shot".to_string()]),
            metadata: HashMap::new(),
            created_at: Utc::now(),
        };

        store.insert(custom.clone()).await;
        assert!(temp_dir.join("custom-voice-123.json").exists());

        // Re-open store pointing to same directory
        let store2 = VoiceProfileStore::with_storage_dir(temp_dir.clone());
        let loaded = store2.get("custom-voice-123").await;
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "Custom Speaker");

        // Remove profile
        store2.remove("custom-voice-123").await;
        assert!(!temp_dir.join("custom-voice-123.json").exists());

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_sanitize_id_path_traversal() {
        assert_eq!(
            VoiceProfileStore::sanitize_id("../../evil_path"),
            Some("______evil_path".to_string())
        );
        assert_eq!(
            VoiceProfileStore::sanitize_id("voice 1/2"),
            Some("voice_1_2".to_string())
        );
        assert_eq!(
            VoiceProfileStore::sanitize_id("valid-voice_name-123"),
            Some("valid-voice_name-123".to_string())
        );
        assert!(VoiceProfileStore::sanitize_id("").is_none());
    }
}
