use crate::auth::{require_token, AuthState};
use crate::handler::{HandlerOutput, OpHandler};
use crate::proto::*;
use crate::ws::ws_handler;
use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

pub type Shared = Arc<dyn OpHandler>;

pub fn router(handler: Shared, auth: Arc<AuthState>) -> Router {
    Router::new()
        .route("/op", post(op))
        .route("/ws", get(ws_handler))
        .layer(middleware::from_fn_with_state(auth, require_token))
        .with_state(handler)
}

async fn op(
    State(h): State<Shared>,
    Json(op): Json<Op>,
) -> Result<Json<OpResult>, StatusCode> {
    match h.handle(op).await {
        HandlerOutput::Unary(r) => Ok(Json(r)),
        HandlerOutput::Stream(_) => Err(StatusCode::BAD_REQUEST),
    }
}
