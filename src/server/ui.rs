//! Embedded UI assets. Only compiled when the `embed-ui` feature is on.
//!
//! When the feature is off, `ui_fallback()` returns `None` and the binary
//! serves no UI; everything outside `/api/*` 404s.

use axum::Router;

pub fn ui_fallback<S: Clone + Send + Sync + 'static>() -> Option<Router<S>> {
    #[cfg(feature = "embed-ui")]
    {
        Some(Router::new().fallback(axum::routing::any(embedded::serve)))
    }
    #[cfg(not(feature = "embed-ui"))]
    {
        None
    }
}

#[cfg(feature = "embed-ui")]
mod embedded {
    use axum::body::Body;
    use axum::http::{header, HeaderValue, StatusCode, Uri};
    use axum::response::{IntoResponse, Response};

    #[derive(rust_embed::Embed)]
    #[folder = "ui/dist"]
    struct Assets;

    pub async fn serve(uri: Uri) -> Response {
        let path = uri.path().trim_start_matches('/');
        let path = if path.is_empty() { "index.html" } else { path };
        match Assets::get(path) {
            Some(file) => respond(path, Body::from(file.data)),
            // SPA fallback: unknown path → index.html so client-side routing works.
            None => match Assets::get("index.html") {
                Some(file) => respond("index.html", Body::from(file.data)),
                None => StatusCode::NOT_FOUND.into_response(),
            },
        }
    }

    fn respond(path: &str, data: Body) -> Response {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let mut resp = data.into_response();
        let header_value =
            HeaderValue::from_str(mime.as_ref()).unwrap_or(HeaderValue::from_static("application/octet-stream"));
        resp.headers_mut().insert(header::CONTENT_TYPE, header_value);
        resp
    }
}
