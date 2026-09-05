use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use async_trait::async_trait;

use super::DataStore;
use crate::error::Result;
use crate::models::{PipelineDefinition, Voice};

#[derive(Clone, Default)]
pub struct MemoryStore {
    pipelines: Arc<RwLock<HashMap<Uuid, PipelineDefinition>>>,
    voices: Arc<RwLock<HashMap<String, Voice>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl DataStore for MemoryStore {
    async fn get_pipeline(&self, id: &Uuid) -> Result<Option<PipelineDefinition>> {
        let lock = self.pipelines.read().await;
        Ok(lock.get(id).cloned())
    }

    async fn save_pipeline(&self, pipeline: &PipelineDefinition) -> Result<()> {
        let mut lock = self.pipelines.write().await;
        lock.insert(pipeline.id, pipeline.clone());
        Ok(())
    }

    async fn list_pipelines(&self) -> Result<Vec<PipelineDefinition>> {
        let lock = self.pipelines.read().await;
        Ok(lock.values().cloned().collect())
    }

    async fn delete_pipeline(&self, id: &Uuid) -> Result<bool> {
        let mut lock = self.pipelines.write().await;
        Ok(lock.remove(id).is_some())
    }

    async fn get_voice(&self, id: &str) -> Result<Option<Voice>> {
        let lock = self.voices.read().await;
        Ok(lock.get(id).cloned())
    }

    async fn save_voice(&self, voice: &Voice) -> Result<()> {
        let mut lock = self.voices.write().await;
        lock.insert(voice.id.clone(), voice.clone());
        Ok(())
    }

    async fn list_voices(&self, language: Option<&str>, engine: Option<&str>) -> Result<Vec<Voice>> {
        let lock = self.voices.read().await;
        let filtered = lock
            .values()
            .filter(|v| {
                if let Some(lang) = language {
                    if !v.language.starts_with(lang) {
                        return false;
                    }
                }
                if let Some(eng) = engine {
                    if v.engine_id != eng {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        Ok(filtered)
    }
}
