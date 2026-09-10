use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

use crate::state::AppState;

pub async fn get_metrics(State(state): State<AppState>) -> Response {
    let (hits, misses) = state.engine_registry.cache().stats();
    let body = state.metrics.render_prometheus(hits, misses).await;

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
        )
        .header(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"))
        .body(axum::body::Body::from(body))
        .unwrap_or_else(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "Error rendering metrics").into_response()
        })
}
