use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "ui_dist/"]
pub struct UiAssets;

pub async fn static_handler(uri: Uri) -> Response {
    let raw_path = uri.path().trim_start_matches('/');
    let path = if raw_path.is_empty() {
        "index.html"
    } else {
        raw_path
    };

    match UiAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let cache_header = if path.starts_with("assets/") {
                "public, max-age=31536000, immutable"
            } else {
                "no-cache, no-store, must-revalidate"
            };

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, mime.as_ref()),
                    (header::CACHE_CONTROL, cache_header),
                ],
                content.data,
            )
                .into_response()
        }
        None => {
            // Single Page Application (SPA) fallback:
            // Route all unknown non-API paths back to index.html
            match UiAssets::get("index.html") {
                Some(content) => (
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                        (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
                    ],
                    content.data,
                )
                    .into_response(),
                None => (
                    StatusCode::NOT_FOUND,
                    [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                    "VoxForg UI bundle not found in binary distribution. Build UI with 'npm run build'.",
                )
                    .into_response(),
            }
        }
    }
}
