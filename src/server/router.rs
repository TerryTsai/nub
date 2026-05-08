//! axum `Router` factory and the unary `/api/op` HTTP handler. WebSocket
//! framing lives in `wire.rs` / `ws.rs`; this file is just the public
//! HTTP surface.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};

use super::ws::ws_handler;
use crate::auth::{introspect, require_token, AuthState, Claims};
use crate::ops::{closed_input, HandlerOutput, Shared};
use crate::proto::{Op, OpResult};

pub fn router(handler: Shared, auth: Arc<AuthState>) -> Router {
    Router::new()
        .route("/api/op", post(op))
        .route("/api/ws", get(ws_handler))
        .layer(middleware::from_fn_with_state(auth, require_token))
        .with_state(handler)
}

async fn op(
    State(h): State<Shared>,
    Extension(claims): Extension<Claims>,
    Json(op): Json<Op>,
) -> Result<Json<OpResult>, StatusCode> {
    if let Some(result) = introspect(&claims, &op) {
        return Ok(Json(result));
    }
    if !claims.allows(&op) {
        tracing::warn!("caller {} denied op {}", claims.sub, op.name());
        return Err(StatusCode::FORBIDDEN);
    }
    match h.handle(op, &claims, closed_input()).await {
        HandlerOutput::Unary(r) => Ok(Json(r)),
        HandlerOutput::Stream(_) => Err(StatusCode::BAD_REQUEST),
    }
}
