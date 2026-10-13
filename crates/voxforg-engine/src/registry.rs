use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::{AudioChunk, Voice};

use crate::cache::AudioCache;
use crate::traits::{SynthesisRequest, TtsEngine};

#[derive(Clone)]
pub struct EngineRegistry {
    engines: Arc<RwLock<HashMap<String, Arc<dyn TtsEngine>>>>,
    cache: Arc<AudioCache>,
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self {
            engines: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(AudioCache::default()),
        }
    }
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cache_capacity(capacity: usize) -> Self {
        Self {
            engines: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(AudioCache::new(capacity)),
        }
    }

    pub fn cache(&self) -> Arc<AudioCache> {
        self.cache.clone()
    }

    pub async fn register(&self, engine: Arc<dyn TtsEngine>) {
        let mut lock = self.engines.write().await;
        lock.insert(engine.id().to_string(), engine);
    }

    pub fn register_sync(&self, engine: Arc<dyn TtsEngine>) {
        if let Ok(mut lock) = self.engines.try_write() {
            lock.insert(engine.id().to_string(), engine);
        }
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
        // If voice_id is explicitly a custom, zero-shot, or cloned voice profile, resolve to qwen3-tts first
        if voice_id.starts_with("cloned-")
            || voice_id.starts_with("qwen3-cloned-")
            || voice_id.starts_with("designed-")
            || voice_id.starts_with("persona-")
            || voice_id.starts_with("custom-")
            || !voice_id.contains('-')
        {
            if let Some(engine) = lock
                .get("qwen3-tts")
                .or_else(|| lock.get("edge-tts"))
                .or_else(|| lock.values().next())
                .cloned()
            {
                let fallback_voice = Voice {
                    id: voice_id.to_string(),
                    name: format!("{} (Custom)", voice_id),
                    engine_id: engine.id().to_string(),
                    language: "en-US".to_string(),
                    gender: voxforg_core::models::Gender::Neutral,
                    sample_rate_hz: 24000,
                    tags: vec!["cloned".to_string(), "custom".to_string()],
                    description: Some("Custom zero-shot voice profile".to_string()),
                };
                return Ok((engine, fallback_voice));
            }
        }
        Err(VoxForgError::VoiceNotFound(format!(
            "Voice '{}' not found in any registered engine",
            voice_id
        )))
    }

    pub async fn synthesize_cached(
        &self,
        engine: Arc<dyn TtsEngine>,
        req: &SynthesisRequest,
    ) -> Result<AudioChunk> {
        if let Some(cached) = self
            .cache
            .get(engine.id(), &req.voice_id, req.speed, req.pitch, &req.text)
            .await
        {
            return Ok(cached);
        }

        let chunk = engine.synthesize(req).await?;
        self.cache
            .insert(
                engine.id(),
                &req.voice_id,
                req.speed,
                req.pitch,
                &req.text,
                chunk.clone(),
            )
            .await;
        Ok(chunk)
    }
}
