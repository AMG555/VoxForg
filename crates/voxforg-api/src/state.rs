use std::sync::Arc;
use voxforg_core::store::DataStore;
use voxforg_engine::EngineRegistry;
use voxforg_hardware::HardwareProfile;
use voxforg_pipeline::PipelineExecutor;

use crate::metrics::MetricsCollector;

#[derive(Clone)]
pub struct AppState {
    pub engine_registry: Arc<EngineRegistry>,
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
        Self {
            engine_registry,
            data_store,
            hardware_profile,
            pipeline_executor,
            api_key,
            metrics,
        }
    }
}
