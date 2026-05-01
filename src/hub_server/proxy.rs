use crate::proto::{Frame, Op, OpResult};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::oneshot;

const PROXY_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) async fn list_nodes(State(state): State<super::State>) -> Json<Vec<String>> {
    Json(state.registry.lock().unwrap().keys().cloned().collect())
}

pub(super) async fn op(
    Path(node_id): Path<String>,
    State(state): State<super::State>,
    Json(op): Json<Op>,
) -> Result<Json<OpResult>, StatusCode> {
    let conn = state.registry.lock().unwrap().get(&node_id).cloned();
    let Some(conn) = conn else {
        return Err(StatusCode::NOT_FOUND);
    };

    let id = conn.next_id.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = oneshot::channel::<OpResult>();
    conn.pending.lock().unwrap().insert(id, tx);

    let frame = Frame::Request { id, op };
    let json = serde_json::to_string(&frame).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if conn.out_tx.send(json).await.is_err() {
        conn.pending.lock().unwrap().remove(&id);
        return Err(StatusCode::BAD_GATEWAY);
    }

    let result = match tokio::time::timeout(PROXY_TIMEOUT, rx).await {
        Ok(Ok(r)) => r,
        _ => {
            conn.pending.lock().unwrap().remove(&id);
            return Err(StatusCode::GATEWAY_TIMEOUT);
        }
    };
    Ok(Json(result))
}
