//! axum middleware that gates `/api/*` on a valid JWT bearer token.
//! Tokens arrive in the `Authorization` header for HTTP and in the
//! `Sec-WebSocket-Protocol` subprotocol list for browser-initiated
//! WebSockets (the standard workaround — `new WebSocket()` can't set
//! `Authorization`).
//!
//! Also home of the `introspect` hook, which lets the auth layer answer
//! introspection ops (`whoami`) before dispatch ever sees them.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

use super::issuer::Issuer;
use super::jwt::{self, Claims};
use crate::proto::{Op, OpResult, WhoamiInfo};

/// State injected into the auth middleware.
pub struct AuthState {
    pub issuer: Arc<Issuer>,
    /// Audience this nub validates against — typically the host id.
    pub audience: String,
}

/// Pre-dispatch hook for ops the auth layer answers itself, bypassing
/// both the per-op scope check and the engine handler. Today only
/// `Op::Whoami` qualifies — `whoami` is "what does this token say?",
/// which any holder of a valid token may ask regardless of scope.
///
/// Both transports call this before checking `claims.allows(&op)`. If
/// you add a new introspection op, add it here AND update the dispatch
/// match in `ops::EngineHandler::handle`.
pub fn introspect(claims: &Claims, op: &Op) -> Option<OpResult> {
    match op {
        Op::Whoami => Some(OpResult::Whoami(WhoamiInfo {
            id: claims.sub.clone(),
            allowed: claims.scopes(),
        })),
        _ => None,
    }
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
