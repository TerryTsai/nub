//! `docker container start/stop/restart/kill/rm`. All POST (or DELETE for
//! remove) on `/containers/{id}/{verb}`. Compat path works on both engines.

use anyhow::Result;

use crate::client::{Query, Req};
use crate::ops::EngineHandler;
use crate::proto::Action;

pub(crate) async fn run(h: &EngineHandler, id: String, action: Action) -> Result<()> {
    let mut conn = h.engine.conn().await?;
    let req = match action {
        Action::Start => Req::post(format!("/containers/{id}/start")),
        Action::Stop { timeout } => Req::post(stop_path(&id, timeout)),
        Action::Restart { timeout } => Req::post(restart_path(&id, timeout)),
        Action::Kill { signal } => Req::post(kill_path(&id, signal.as_deref())),
        Action::Remove { force, volumes } => Req::delete(remove_path(&id, force, volumes)),
    };
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

fn remove_path(id: &str, force: bool, volumes: bool) -> String {
    let mut q = Query::new();
    q.push_bool("force", force);
    q.push_bool("v", volumes);
    format!("/containers/{id}{}", q.finish())
}
