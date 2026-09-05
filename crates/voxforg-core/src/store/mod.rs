pub mod memory;

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::Result;
use crate::models::{PipelineDefinition, Voice};

#[async_trait]
pub trait DataStore: Send + Sync {
    async fn get_pipeline(&self, id: &Uuid) -> Result<Option<PipelineDefinition>>;
    async fn save_pipeline(&self, pipeline: &PipelineDefinition) -> Result<()>;
    async fn list_pipelines(&self) -> Result<Vec<PipelineDefinition>>;
    async fn delete_pipeline(&self, id: &Uuid) -> Result<bool>;

    async fn get_voice(&self, id: &str) -> Result<Option<Voice>>;
    async fn save_voice(&self, voice: &Voice) -> Result<()>;
    async fn list_voices(&self, language: Option<&str>, engine: Option<&str>) -> Result<Vec<Voice>>;
}
