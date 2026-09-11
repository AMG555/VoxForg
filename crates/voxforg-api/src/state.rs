use std::sync::Arc;
use voxforg_core::store::DataStore;
use voxforg_engine::EngineRegistry;
use voxforg_hardware::HardwareProfile;
use voxforg_pipeline::PipelineExecutor;
use voxforg_pronunciation::{ProcessorConfig, PronunciationProcessor};
use voxforg_router::VoiceRouter;

use crate::metrics::MetricsCollector;

#[derive(Clone)]
pub struct AppState {
    pub engine_registry: Arc<EngineRegistry>,
    pub voice_router: Arc<VoiceRouter>,
    pub pronunciation: Arc<PronunciationProcessor>,
    pub data_store: Arc<dyn DataStore>,
    pub hardware_profile: HardwareProfile,
    pub pipeline_executor: Arc<PipelineExecutor>,
    pub api_key: Option<String>,
    pub metrics: Arc<MetricsCollector>,
}

impl AppState {
    pub fn new(
        engine_registry: Arc<EngineRegistry>,
        data_store: Arc<dyn DataStore>,
        hardware_profile: HardwareProfile,
        api_key: Option<String>,
    ) -> Self {
        let pipeline_executor = Arc::new(PipelineExecutor::new(engine_registry.clone()));
        let metrics = Arc::new(MetricsCollector::new());
        let voice_router = Arc::new(VoiceRouter::new((*engine_registry).clone()));
        let pronunciation = Arc::new(PronunciationProcessor::new(ProcessorConfig::default()));
        Self {
            engine_registry,
            voice_router,
            pronunciation,
            data_store,
            hardware_profile,
            pipeline_executor,
            api_key,
            metrics,
        }
    }
}
