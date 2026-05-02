use crate::config::TrustEntry;
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub struct AuthState {
    pub trust: Vec<TrustEntry>,
}

pub async fn require_token(
    State(state): State<Arc<AuthState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let presented = bearer_from_header(req.headers()).or_else(|| ws_bearer_subprotocol(req.headers()));
    let Some(token) = presented else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let entry = state
        .trust
        .iter()
        .find(|e| ct_eq(e.token.as_bytes(), token.as_bytes()))
        .cloned();
    let Some(entry) = entry else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    req.extensions_mut().insert(entry);
    Ok(next.run(req).await)
}

fn bearer_from_header(h: &HeaderMap) -> Option<String> {
    h.get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(str::to_owned)
}

// Browsers can't set Authorization on `new WebSocket()`. The standard
// workaround is to smuggle the bearer in Sec-WebSocket-Protocol as one of
// the offered subprotocols, e.g. `bearer.<token>`.
fn ws_bearer_subprotocol(h: &HeaderMap) -> Option<String> {
    h.get_all(header::SEC_WEBSOCKET_PROTOCOL)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|s| s.split(','))
        .map(str::trim)
        .find_map(|s| s.strip_prefix("bearer.").map(str::to_owned))
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
