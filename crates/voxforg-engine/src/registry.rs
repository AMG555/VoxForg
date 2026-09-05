use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::Voice;

use crate::traits::TtsEngine;

#[derive(Clone, Default)]
pub struct EngineRegistry {
    engines: Arc<RwLock<HashMap<String, Arc<dyn TtsEngine>>>>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn register(&self, engine: Arc<dyn TtsEngine>) {
        let mut lock = self.engines.write().await;
        lock.insert(engine.id().to_string(), engine);
    }

    pub async fn get(&self, engine_id: &str) -> Option<Arc<dyn TtsEngine>> {
        let lock = self.engines.read().await;
        lock.get(engine_id).cloned()
    }

    pub async fn list_engines(&self) -> Vec<String> {
        let lock = self.engines.read().await;
        lock.keys().cloned().collect()
    }

    pub async fn list_all_voices(&self) -> Result<Vec<Voice>> {
        let lock = self.engines.read().await;
        let mut all_voices = Vec::new();
        for engine in lock.values() {
            let voices = engine.voices().await?;
            all_voices.extend(voices);
        }
        Ok(all_voices)
    }

    pub async fn resolve_voice(&self, voice_id: &str) -> Result<(Arc<dyn TtsEngine>, Voice)> {
        let lock = self.engines.read().await;
        for engine in lock.values() {
            let voices = engine.voices().await?;
            if let Some(voice) = voices.into_iter().find(|v| v.id == voice_id) {
                return Ok((engine.clone(), voice));
            }
        }
        Err(VoxForgError::VoiceNotFound(format!(
            "Voice '{}' not found in any registered engine",
            voice_id
        )))
    }
}
