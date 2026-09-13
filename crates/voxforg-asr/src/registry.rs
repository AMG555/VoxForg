use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::mock::MockAsrEngine;
use crate::traits::{AsrEngine, AsrEngineInfo};
use crate::whisper::WhisperAsrEngine;

/// Thread-safe registry for managing ASR engines.
pub struct AsrRegistry {
    engines: Arc<RwLock<HashMap<String, Arc<dyn AsrEngine>>>>,
    default_engine_id: String,
}

impl Default for AsrRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl AsrRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self {
            engines: Arc::new(RwLock::new(HashMap::new())),
            default_engine_id: "whisper-base".to_string(),
        }
    }

    /// Construct a registry seeded with standard built-in engines.
    pub fn with_defaults() -> Self {
        let mut map: HashMap<String, Arc<dyn AsrEngine>> = HashMap::new();
        let whisper = Arc::new(WhisperAsrEngine::default());
        let mock = Arc::new(MockAsrEngine::default());

        map.insert(whisper.id(), whisper);
        map.insert(mock.id(), mock);

        if let Some(openai_asr) = crate::openai::OpenAiAsrEngine::from_env() {
            let id = openai_asr.id();
            map.insert(id, Arc::new(openai_asr));
        }

        Self {
            engines: Arc::new(RwLock::new(map)),
            default_engine_id: "whisper-base".to_string(),
        }
    }

    /// Register or overwrite an ASR engine.
    pub async fn register(&self, engine: Arc<dyn AsrEngine>) {
        let id = engine.id();
        let mut map = self.engines.write().await;
        map.insert(id, engine);
    }

    /// Look up an engine by identifier.
    pub async fn get(&self, id: &str) -> Option<Arc<dyn AsrEngine>> {
        let map = self.engines.read().await;
        map.get(id).cloned()
    }

    /// Return the default fallback engine if available.
    pub async fn default_engine(&self) -> Option<Arc<dyn AsrEngine>> {
        let map = self.engines.read().await;
        if let Some(engine) = map.get(&self.default_engine_id) {
            return Some(engine.clone());
        }
        map.values().next().cloned()
    }

    /// List metadata for all registered engines.
    pub async fn list(&self) -> Vec<AsrEngineInfo> {
        let map = self.engines.read().await;
        map.values().map(|e| e.info()).collect()
    }

    /// Unregister an engine.
    pub async fn unregister(&self, id: &str) -> Option<Arc<dyn AsrEngine>> {
        let mut map = self.engines.write().await;
        map.remove(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_asr_registry_defaults_and_lookup() {
        let registry = AsrRegistry::with_defaults();
        let list = registry.list().await;
        assert!(list.len() >= 2);

        let whisper = registry.get("whisper-base").await;
        assert!(whisper.is_some());

        let default_eng = registry.default_engine().await;
        assert!(default_eng.is_some());
        assert_eq!(default_eng.unwrap().id(), "whisper-base");

        let missing = registry.get("non-existent").await;
        assert!(missing.is_none());
    }
}
