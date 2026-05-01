mod nub_conn;
mod proxy;

use crate::auth::{require_token, AuthState};
use anyhow::Result;
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

use crate::proto::OpResult;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bind: String,
    pub phone_token: String,
    #[serde(default)]
    pub nubs: Vec<NubEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NubEntry {
    pub id: String,
    pub token: String,
}

pub(super) struct NubConn {
    pub out_tx: mpsc::Sender<String>,
    pub pending: Mutex<HashMap<u64, oneshot::Sender<OpResult>>>,
    pub next_id: AtomicU64,
}

pub(super) type Registry = Arc<Mutex<HashMap<String, Arc<NubConn>>>>;

#[derive(Clone)]
pub(super) struct State {
    pub registry: Registry,
    pub nubs: Arc<Vec<NubEntry>>,
}

pub async fn run(cfg: Config) -> Result<()> {
    let state = State {
        registry: Arc::new(Mutex::new(HashMap::new())),
        nubs: Arc::new(cfg.nubs),
    };
    let phone_auth = Arc::new(AuthState { token: cfg.phone_token });

    let phone_routes = Router::new()
        .route("/nubs", get(proxy::list_nubs))
        .route("/nubs/:id/op", post(proxy::op))
        .layer(middleware::from_fn_with_state(phone_auth, require_token))
        .with_state(state.clone());
    let nub_routes = Router::new().route("/nub", get(nub_conn::ws_handler)).with_state(state);
    let app = Router::new().merge(phone_routes).merge(nub_routes);

    let listener = tokio::net::TcpListener::bind(&cfg.bind).await?;
    tracing::info!("hub listening on {}", cfg.bind);
    axum::serve(listener, app).await?;
    Ok(())
}
