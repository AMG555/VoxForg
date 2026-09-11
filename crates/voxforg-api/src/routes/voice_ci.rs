/// REST endpoints for voice CI/CD regression testing.
///
/// | Method | Path                           | Description                              |
/// |--------|--------------------------------|------------------------------------------|
/// | GET    | `/v1/voice-ci/profiles`        | List all stored voice quality profiles   |
/// | POST   | `/v1/voice-ci/profiles`        | Create / update a voice quality profile  |
/// | POST   | `/v1/voice-ci/compare`         | Compare current engine output vs profile |
use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use voxforg_core::error::ProblemDetails;
use voxforg_core::models::AudioContainerFormat;
use voxforg_engine::SynthesisRequest;

use crate::state::AppState;

// ─── Types ──────────────────────────────────────────────────────────────────

/// A stored baseline voice quality profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceProfile {
    /// Unique profile identifier (typically `{engine_id}:{voice_id}`).
    pub id: String,
    /// Engine this profile was captured from.
    pub engine_id: String,
    /// Voice ID within that engine.
    pub voice_id: String,
    /// Mean latency when the profile was captured (ms).
    pub baseline_latency_ms: u64,
    /// Synthesis success rate at capture time (0.0–1.0).
    pub baseline_success_rate: f32,
    /// Number of sentences used to build baseline.
    pub sentence_count: usize,
    /// Timestamp of capture.
    pub captured_at: DateTime<Utc>,
}

/// Result of a single sentence regression check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceResult {
    pub sentence_id: String,
    pub latency_ms: u64,
    pub ok: bool,
}

/// Full regression report comparing current output against a stored profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceRegressionReport {
    pub profile_id: String,
    pub engine_id: String,
    pub voice_id: String,
    /// Whether the current run PASSED all regression thresholds.
    pub passed: bool,
    pub current_latency_ms: u64,
    pub baseline_latency_ms: u64,
    /// Relative latency change: positive = slower, negative = faster.
    pub latency_delta_pct: f32,
    pub current_success_rate: f32,
    pub baseline_success_rate: f32,
    pub sentences: Vec<SentenceResult>,
    pub run_at: DateTime<Utc>,
}

// ─── In-process profile store ────────────────────────────────────────────────

/// Very lightweight in-memory profile store.  
/// In production this should be backed by `DataStore`.
#[derive(Default, Clone)]
pub struct ProfileStore(Arc<RwLock<HashMap<String, VoiceProfile>>>);

impl ProfileStore {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn upsert(&self, p: VoiceProfile) {
        self.0.write().await.insert(p.id.clone(), p);
    }
    pub async fn list(&self) -> Vec<VoiceProfile> {
        let lock = self.0.read().await;
        let mut v: Vec<_> = lock.values().cloned().collect();
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v
    }
    pub async fn get(&self, id: &str) -> Option<VoiceProfile> {
        self.0.read().await.get(id).cloned()
    }
}

// ─── Request / response shapes ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateProfileRequest {
    pub engine_id: String,
    pub voice_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CompareRequest {
    pub profile_id: String,
    /// Latency regression threshold (default 20%).
    #[serde(default = "default_latency_threshold")]
    pub latency_threshold_pct: f32,
    /// Minimum allowed success rate (default 1.0 = 100%).
    #[serde(default = "default_success_threshold")]
    pub success_threshold: f32,
}

fn default_latency_threshold() -> f32 {
    20.0
}
fn default_success_threshold() -> f32 {
    1.0
}

// ─── Handlers ────────────────────────────────────────────────────────────────

/// `GET /v1/voice-ci/profiles`
pub async fn list_profiles(State(state): State<AppState>) -> Json<Vec<VoiceProfile>> {
    Json(state.voice_ci_store.list().await)
}

/// `POST /v1/voice-ci/profiles` — run benchmark sentences and capture baseline.
pub async fn create_profile(
    State(state): State<AppState>,
    Json(body): Json<CreateProfileRequest>,
) -> Result<(StatusCode, Json<VoiceProfile>), (StatusCode, Json<ProblemDetails>)> {
    use std::time::Instant;
    use voxforg_benchmark::BENCHMARK_SENTENCES;

    let engine = state
        .engine_registry
        .get(&body.engine_id)
        .await
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/engine-not-found".to_string(),
                    title: "Engine Not Found".to_string(),
                    status: 404,
                    detail: format!("Engine '{}' is not registered", body.engine_id),
                    instance: "/v1/voice-ci/profiles".to_string(),
                }),
            )
        })?;

    let mut latencies: Vec<u64> = Vec::new();
    let mut ok_count = 0usize;

    for (_id, text) in BENCHMARK_SENTENCES {
        let req = SynthesisRequest {
            text: text.to_string(),
            voice_id: body.voice_id.clone(),
            speed: 1.0,
            pitch: 0.0,
            format: AudioContainerFormat::Wav,
        };
        let t0 = Instant::now();
        if engine.synthesize(&req).await.is_ok() {
            latencies.push(t0.elapsed().as_millis() as u64);
            ok_count += 1;
        }
    }

    let total = BENCHMARK_SENTENCES.len();
    let mean_latency = if latencies.is_empty() {
        0
    } else {
        latencies.iter().sum::<u64>() / latencies.len() as u64
    };
    let success_rate = ok_count as f32 / total as f32;

    let profile_id = format!("{}:{}", body.engine_id, body.voice_id);
    let profile = VoiceProfile {
        id: profile_id,
        engine_id: body.engine_id,
        voice_id: body.voice_id,
        baseline_latency_ms: mean_latency,
        baseline_success_rate: success_rate,
        sentence_count: total,
        captured_at: Utc::now(),
    };

    let created = state.voice_ci_store.get(&profile.id).await.is_none();
    state.voice_ci_store.upsert(profile.clone()).await;
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(profile)))
}

/// `POST /v1/voice-ci/compare` — compare current engine output against a stored profile.
pub async fn compare(
    State(state): State<AppState>,
    Json(body): Json<CompareRequest>,
) -> Result<Json<VoiceRegressionReport>, (StatusCode, Json<ProblemDetails>)> {
    use std::time::Instant;
    use voxforg_benchmark::BENCHMARK_SENTENCES;

    let profile = state
        .voice_ci_store
        .get(&body.profile_id)
        .await
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/profile-not-found".to_string(),
                    title: "Profile Not Found".to_string(),
                    status: 404,
                    detail: format!("Voice profile '{}' not found", body.profile_id),
                    instance: "/v1/voice-ci/compare".to_string(),
                }),
            )
        })?;

    let engine = state
        .engine_registry
        .get(&profile.engine_id)
        .await
        .ok_or_else(|| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/engine-not-found".to_string(),
                    title: "Engine Not Registered".to_string(),
                    status: 422,
                    detail: format!(
                        "Engine '{}' referenced by profile is not registered",
                        profile.engine_id
                    ),
                    instance: "/v1/voice-ci/compare".to_string(),
                }),
            )
        })?;

    let mut sentences = Vec::new();
    let mut latencies: Vec<u64> = Vec::new();

    for (sid, text) in BENCHMARK_SENTENCES {
        let req = SynthesisRequest {
            text: text.to_string(),
            voice_id: profile.voice_id.clone(),
            speed: 1.0,
            pitch: 0.0,
            format: AudioContainerFormat::Wav,
        };
        let t0 = Instant::now();
        let ok = engine.synthesize(&req).await.is_ok();
        let latency_ms = t0.elapsed().as_millis() as u64;
        if ok {
            latencies.push(latency_ms);
        }
        sentences.push(SentenceResult {
            sentence_id: sid.to_string(),
            latency_ms,
            ok,
        });
    }

    let ok_count = latencies.len();
    let total = BENCHMARK_SENTENCES.len();
    let current_latency_ms = if latencies.is_empty() {
        0
    } else {
        latencies.iter().sum::<u64>() / latencies.len() as u64
    };
    let current_success_rate = ok_count as f32 / total as f32;

    let latency_delta_pct = if profile.baseline_latency_ms == 0 {
        0.0
    } else {
        (current_latency_ms as f32 - profile.baseline_latency_ms as f32)
            / profile.baseline_latency_ms as f32
            * 100.0
    };

    let passed = latency_delta_pct <= body.latency_threshold_pct
        && current_success_rate >= body.success_threshold;

    Ok(Json(VoiceRegressionReport {
        profile_id: profile.id,
        engine_id: profile.engine_id,
        voice_id: profile.voice_id,
        passed,
        current_latency_ms,
        baseline_latency_ms: profile.baseline_latency_ms,
        latency_delta_pct,
        current_success_rate,
        baseline_success_rate: profile.baseline_success_rate,
        sentences,
        run_at: Utc::now(),
    }))
}
