//! `docker container start/stop/restart/kill/rm`. All POST (or DELETE for
//! remove) on `/containers/{id}/{verb}`. Compat path works on both engines.
//!
//! One function per action variant — each network-exposed Op maps to
//! exactly one function here, no shared dispatch envelope.

use anyhow::Result;

use crate::client::{Query, Req};
use crate::ops::EngineHandler;

pub(crate) async fn start(h: &EngineHandler, id: String) -> Result<()> {
    send(h, Req::post(format!("/containers/{id}/start"))).await
}

pub(crate) async fn stop(h: &EngineHandler, id: String, timeout: Option<i64>) -> Result<()> {
    send(h, Req::post(stop_path(&id, timeout))).await
}

pub(crate) async fn restart(h: &EngineHandler, id: String, timeout: Option<i64>) -> Result<()> {
    send(h, Req::post(restart_path(&id, timeout))).await
}

pub(crate) async fn kill(h: &EngineHandler, id: String, signal: Option<String>) -> Result<()> {
    send(h, Req::post(kill_path(&id, signal.as_deref()))).await
}

pub(crate) async fn remove(h: &EngineHandler, id: String, force: bool) -> Result<()> {
    send(h, Req::delete(remove_path(&id, force))).await
}

async fn send(h: &EngineHandler, req: crate::client::Req) -> Result<()> {
    let mut conn = h.engine.conn().await?;
    conn.send_unary(req.build()?).await?.ok()?;
    Ok(())
}

fn stop_path(id: &str, timeout: Option<i64>) -> String {
    let mut q = Query::new();
    if let Some(t) = timeout {
        q.push("t", &t.to_string());
    }
    format!("/containers/{id}/stop{}", q.finish())
}

fn restart_path(id: &str, timeout: Option<i64>) -> String {
    let mut q = Query::new();
    if let Some(t) = timeout {
        q.push("t", &t.to_string());
    }
    format!("/containers/{id}/restart{}", q.finish())
}

fn kill_path(id: &str, signal: Option<&str>) -> String {
    let mut q = Query::new();
    if let Some(s) = signal {
        q.push("signal", s);
    }
    format!("/containers/{id}/kill{}", q.finish())
}

fn remove_path(id: &str, force: bool) -> String {
    let mut q = Query::new();
    q.push_bool("force", force);
    format!("/containers/{id}{}", q.finish())
}
