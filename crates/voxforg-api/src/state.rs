use std::sync::Arc;
use voxforg_asr::AsrRegistry;
use voxforg_benchmark::store::BenchmarkStore;
use voxforg_core::store::DataStore;
use voxforg_engine::{EngineRegistry, VoiceIdentityResolver, VoiceProfileStore};
use voxforg_hardware::HardwareProfile;
use voxforg_pipeline::PipelineExecutor;
use voxforg_pronunciation::{ProcessorConfig, PronunciationProcessor};
use voxforg_router::VoiceRouter;
use voxforg_worker::{WorkerClient, WorkerPool};

use crate::metrics::MetricsCollector;
use crate::routes::voice_ci::ProfileStore;

#[derive(Clone)]
pub struct AppState {
    pub engine_registry: Arc<EngineRegistry>,
    pub voice_router: Arc<VoiceRouter>,
    pub voice_identities: Arc<VoiceIdentityResolver>,
    pub voice_profiles: Arc<VoiceProfileStore>,
    pub asr_registry: Arc<AsrRegistry>,
    pub pronunciation: Arc<PronunciationProcessor>,
    pub benchmark_store: Arc<BenchmarkStore>,
    pub voice_ci_store: ProfileStore,
    pub worker_pool: Arc<WorkerPool>,
    pub worker_client: Arc<WorkerClient>,
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
        let asr_registry = Arc::new(AsrRegistry::with_defaults());
        let pipeline_executor = Arc::new(
            PipelineExecutor::new(engine_registry.clone()).with_asr_registry(asr_registry.clone()),
        );
        let metrics = Arc::new(MetricsCollector::new());
        let voice_router = Arc::new(VoiceRouter::new((*engine_registry).clone()));
        let voice_identities = Arc::new(VoiceIdentityResolver::with_defaults());
        let voice_profiles = Arc::new(VoiceProfileStore::with_defaults());
        let pronunciation = Arc::new(PronunciationProcessor::new(ProcessorConfig::default()));
        let benchmark_store = Arc::new(BenchmarkStore::new());
        let voice_ci_store = ProfileStore::new();
        let worker_pool = Arc::new(WorkerPool::new());
        let worker_client = Arc::new(WorkerClient::new());
        Self {
            engine_registry,
            voice_router,
            voice_identities,
            voice_profiles,
            asr_registry,
            pronunciation,
            benchmark_store,
            voice_ci_store,
            worker_pool,
            worker_client,
            data_store,
            hardware_profile,
            pipeline_executor,
            api_key,
            metrics,
        }
    }
}
