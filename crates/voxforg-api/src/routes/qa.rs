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

    if scenario.text.len() > 50_000 {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/input-too-large".to_string(),
            title: "Scenario Text Exceeds Limit".to_string(),
            status: StatusCode::BAD_REQUEST.as_u16(),
            detail: "The 'text' parameter cannot exceed 50,000 characters per scenario".to_string(),
            instance: "/v1/qa/ab-test".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(err)));
    }

    let runner = AbTestRunner::new(state.engine_registry.clone());
    let comparison = runner.run_comparison(&scenario).await.map_err(|e| {
        let problem = e.to_problem_details("/v1/qa/ab-test");
        (StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(problem))
    })?;

    Ok(Json(comparison))
}
