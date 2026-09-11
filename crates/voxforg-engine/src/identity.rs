use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::{Voice, VoiceEngineMapping, VoiceIdentity};

use crate::registry::EngineRegistry;
use crate::traits::TtsEngine;

/// Manages portable voice identities and resolves them to concrete
/// engine/voice pairs based on engine availability and priority order.
#[derive(Clone)]
pub struct VoiceIdentityResolver {
    identities: Arc<RwLock<HashMap<String, VoiceIdentity>>>,
}

impl Default for VoiceIdentityResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceIdentityResolver {
    pub fn new() -> Self {
        Self {
            identities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Construct resolver with default portable voice identities.
    pub fn with_defaults() -> Self {
        let defaults = vec![
            VoiceIdentity {
                id: "default-female".to_string(),
                display_name: "Default Female Voice".to_string(),
                quality: voxforg_core::models::VoiceQuality::High,
                style: "conversational".to_string(),
                accent: Some("en-US".to_string()),
                gender: Some(voxforg_core::models::Gender::Female),
                engine_mappings: vec![
                    VoiceEngineMapping {
                        engine_id: "edge-tts".to_string(),
                        voice_id: "en-US-JennyNeural".to_string(),
                        priority: 1,
                    },
                    VoiceEngineMapping {
                        engine_id: "mock-tts".to_string(),
                        voice_id: "mock-en-female".to_string(),
                        priority: 2,
                    },
                ],
            },
            VoiceIdentity {
                id: "default-male".to_string(),
                display_name: "Default Male Voice".to_string(),
                quality: voxforg_core::models::VoiceQuality::High,
                style: "conversational".to_string(),
                accent: Some("en-US".to_string()),
                gender: Some(voxforg_core::models::Gender::Male),
                engine_mappings: vec![
                    VoiceEngineMapping {
                        engine_id: "edge-tts".to_string(),
                        voice_id: "en-US-GuyNeural".to_string(),
                        priority: 1,
                    },
                    VoiceEngineMapping {
                        engine_id: "mock-tts".to_string(),
                        voice_id: "mock-en-male".to_string(),
                        priority: 2,
                    },
                ],
            },
        ];

        let mut map = HashMap::new();
        for id in defaults {
            map.insert(id.id.clone(), id);
        }

        Self {
            identities: Arc::new(RwLock::new(map)),
        }
    }

    /// Register or update a voice identity.
    pub async fn register(&self, identity: VoiceIdentity) {
        let mut lock = self.identities.write().await;
        lock.insert(identity.id.clone(), identity);
    }

    /// Fetch a voice identity by ID.
    pub async fn get(&self, id: &str) -> Option<VoiceIdentity> {
        let lock = self.identities.read().await;
        lock.get(id).cloned()
    }

    /// List all registered voice identities.
    pub async fn list(&self) -> Vec<VoiceIdentity> {
        let lock = self.identities.read().await;
        let mut list: Vec<_> = lock.values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    /// Remove a voice identity by ID.
    pub async fn remove(&self, id: &str) -> Option<VoiceIdentity> {
        let mut lock = self.identities.write().await;
        lock.remove(id)
    }

    /// Resolve a voice identity ID to the best available concrete engine and voice.
    /// Evaluates engine mappings in ascending order of priority (1 = highest preference).
    pub async fn resolve(
        &self,
        identity_id: &str,
        registry: &EngineRegistry,
    ) -> Result<(Arc<dyn TtsEngine>, Voice)> {
        let identity = {
            let lock = self.identities.read().await;
            lock.get(identity_id).cloned()
        };

        let identity = identity.ok_or_else(|| {
            VoxForgError::VoiceNotFound(format!("Voice identity '{}' not found", identity_id))
        })?;

        let mut sorted_mappings = identity.engine_mappings.clone();
        sorted_mappings.sort_by_key(|m| m.priority);

        for mapping in sorted_mappings {
            if let Some(engine) = registry.get(&mapping.engine_id).await {
                if let Ok(voices) = engine.voices().await {
                    if let Some(voice) = voices.into_iter().find(|v| v.id == mapping.voice_id) {
                        return Ok((engine, voice));
                    }
                }
            }
        }

        Err(VoxForgError::VoiceNotFound(format!(
            "No available engine mapping found for voice identity '{}'",
            identity_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockTtsEngine;

    #[tokio::test]
    async fn test_voice_identity_registration_and_resolution() {
        let registry = EngineRegistry::new();
        let mock_engine = Arc::new(MockTtsEngine::new(24000));
        registry.register(mock_engine).await;

        let resolver = VoiceIdentityResolver::new();
        let identity = VoiceIdentity {
            id: "alice".to_string(),
            display_name: "Alice Reporter".to_string(),
            quality: voxforg_core::models::VoiceQuality::High,
            style: "news".to_string(),
            accent: Some("en-US".to_string()),
            gender: Some(voxforg_core::models::Gender::Female),
            engine_mappings: vec![
                VoiceEngineMapping {
                    engine_id: "non-existent-engine".to_string(),
                    voice_id: "voice-1".to_string(),
                    priority: 1,
                },
                VoiceEngineMapping {
                    engine_id: "mock-tts".to_string(),
                    voice_id: "mock-en-female".to_string(),
                    priority: 2,
                },
            ],
        };

        resolver.register(identity).await;
        assert_eq!(resolver.list().await.len(), 1);

        let (engine, voice) = resolver.resolve("alice", &registry).await.unwrap();
        assert_eq!(engine.id(), "mock-tts");
        assert_eq!(voice.id, "mock-en-female");

        // Non-existent identity returns error
        assert!(resolver.resolve("bob", &registry).await.is_err());
    }
}
