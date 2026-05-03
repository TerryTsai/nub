//! Authentication: Ed25519-signed JWT bearer tokens. nub trusts a single
//! issuer key (auto-generated and persisted, or pinned externally via
//! `trusted_issuer` in config). Authorization derives from the token's
//! `scope` claim.

pub mod issuer;
pub mod jwt;

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

pub use issuer::Issuer;
pub use jwt::Claims;

/// State injected into the auth middleware.
pub struct AuthState {
    pub issuer: Arc<Issuer>,
    /// Audience this nub validates against — typically the host id.
    pub audience: String,
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
    match jwt::verify(&token, &state.issuer, &state.audience) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(e) => {
            tracing::debug!("jwt verify failed: {e}");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
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
