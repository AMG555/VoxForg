use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use subtle::ConstantTimeEq;
use voxforg_core::error::ProblemDetails;
use crate::state::AppState;

pub async fn security_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    response
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ProblemDetails>)> {
    let expected_key = match &state.api_key {
        Some(key) if !key.is_empty() => key,
        _ => return Ok(next.run(req).await),
    };

    let path = req.uri().path();
    if path == "/health" || path == "/health/ready" {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let is_authorized = match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = &auth[7..];
            let token_bytes = token.as_bytes();
            let expected_bytes = expected_key.as_bytes();
            token_bytes.ct_eq(expected_bytes).into()
        }
        _ => false,
    };

    if is_authorized {
        Ok(next.run(req).await)
    } else {
        let err = ProblemDetails {
            problem_type: "https://voxforg.org/errors/unauthorized".to_string(),
            title: "Unauthorized".to_string(),
            status: StatusCode::UNAUTHORIZED.as_u16(),
            detail: "Invalid or missing Bearer API token".to_string(),
            instance: path.to_string(),
        };
        Err((StatusCode::UNAUTHORIZED, Json(err)))
    }
}
