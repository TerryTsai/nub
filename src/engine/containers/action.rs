//! Container lifecycle ops: start / stop / restart / kill / remove.

use super::types::ContainerAction;
use crate::engine::{http::Query, Engine, Req, Result};

pub(super) async fn run(engine: &Engine, id: &str, action: ContainerAction) -> Result<()> {
    let mut conn = engine.conn().await?;
    match action {
        ContainerAction::Start => {
            let path = format!("/containers/{id}/start");
            conn.send_unary(Req::post(path).build()?).await?.ok()
        }
        ContainerAction::Stop { timeout } => {
            let mut q = Query::new();
            if let Some(t) = timeout {
                q.push("t", &t.to_string());
            }
            let path = format!("/containers/{id}/stop{}", q.finish());
            conn.send_unary(Req::post(path).build()?).await?.ok()
        }
        ContainerAction::Restart { timeout } => {
            let mut q = Query::new();
            if let Some(t) = timeout {
                q.push("t", &t.to_string());
            }
            let path = format!("/containers/{id}/restart{}", q.finish());
            conn.send_unary(Req::post(path).build()?).await?.ok()
        }
        ContainerAction::Kill { signal } => {
            let mut q = Query::new();
            if let Some(s) = signal {
                q.push("signal", &s);
            }
            let path = format!("/containers/{id}/kill{}", q.finish());
            conn.send_unary(Req::post(path).build()?).await?.ok()
        }
        ContainerAction::Remove { force, volumes } => {
            let mut q = Query::new();
            q.push_bool("force", force);
            q.push_bool("v", volumes);
            let path = format!("/containers/{id}{}", q.finish());
            conn.send_unary(Req::delete(path).build()?).await?.ok()
        }
    }
}
