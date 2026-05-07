pub mod tls;
pub mod ui;
mod wire;
mod ws;

use crate::auth::{introspect, require_token, AuthState, Claims};
use crate::ops::{closed_input, HandlerOutput, OpHandler};
use crate::proto::*;
use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    routing::{get, post},
    Extension, Json, Router,
};
use std::sync::Arc;

use ws::ws_handler;

pub type Shared = Arc<dyn OpHandler>;

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
