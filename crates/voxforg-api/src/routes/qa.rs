use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use voxforg_core::error::ProblemDetails;
use voxforg_engine::{AbTestComparison, AbTestRunner, AbTestScenario};

use crate::state::AppState;

pub async fn run_ab_test(
    State(state): State<AppState>,
    Json(scenario): Json<AbTestScenario>,
) -> Result<Json<AbTestComparison>, (StatusCode, Json<ProblemDetails>)> {
    if scenario.text.trim().is_empty() {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/empty-scenario-text".to_string(),
            title: "Empty Scenario Text".to_string(),
            status: StatusCode::BAD_REQUEST.as_u16(),
            detail: "A/B test scenario requires non-empty test text".to_string(),
            instance: "/v1/qa/ab-test".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(err)));
    }

    let runner = AbTestRunner::new(state.engine_registry.clone());
    let comparison = runner.run_comparison(&scenario).await.map_err(|e| {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/ab-test-failed".to_string(),
            title: "A/B Test Execution Failed".to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            detail: e.to_string(),
            instance: "/v1/qa/ab-test".to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
    })?;

    Ok(Json(comparison))
}
