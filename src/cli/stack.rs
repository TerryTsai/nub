//! `nub stack` — CLI lifecycle for compose-shaped stacks. The web UI
//! is the primary interactive surface; the CLI exists for codified /
//! scriptable deploys (`nub stack deploy app.yml` from CI or cron) and
//! for SSH'd-in operators who want a quick `ls`/`rm`/`logs` from the
//! shell.

use anyhow::{Context, Result};
use clap::Subcommand;
use futures::StreamExt as _;
use std::io::{BufRead as _, IsTerminal as _, Read, Write as _};

use crate::auth::Claims;
use crate::config;
use crate::ops;
use crate::proto::StreamChunk;

#[derive(Subcommand)]
pub enum StackCmd {
    /// Deploy a compose file as a stack.
    Deploy {
        /// Stack name. Lowercase alphanumeric, dash, underscore.
        #[arg(value_name = "NAME")]
        name: String,
        /// Path to compose file. Use `-` for stdin.
        #[arg(value_name = "FILE")]
        file: String,
    },
    /// List stacks on this host.
    Ls,
    /// Tear down a stack: stop and remove its containers, drop the
    /// stack network, delete the manifest. Named volumes are preserved.
    Rm {
        #[arg(value_name = "NAME")]
        name: String,
        /// Skip the confirmation prompt.
        #[arg(long)]
        yes: bool,
    },
    /// Redeploy a stack from its stored manifest. Always-recreate.
    Redeploy {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Stream interleaved logs from every container in a stack.
    Logs {
        #[arg(value_name = "NAME")]
        name: String,
        /// Keep streaming after current logs flush.
        #[arg(long, short)]
        follow: bool,
        /// Lines of history to replay. Default: 100.
        #[arg(long, value_name = "N")]
        tail: Option<u32>,
    },
}

pub fn run(action: StackCmd) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(dispatch(action))
}

async fn dispatch(action: StackCmd) -> Result<()> {
    let handler = connect().await?;
    match action {
        StackCmd::Deploy { name, file } => deploy(&handler, name, file).await,
        StackCmd::Ls => list(&handler).await,
        StackCmd::Rm { name, yes } => remove(&handler, name, yes).await,
        StackCmd::Redeploy { name } => redeploy(&handler, name).await,
        StackCmd::Logs { name, follow, tail } => logs(&handler, name, follow, tail).await,
    }
}

async fn connect() -> Result<ops::EngineHandler> {
    let cfg = config::Config::load(None)?.unwrap_or_default();
    ops::EngineHandler::connect(ops::Policy::from_config(&cfg)).await
}

async fn deploy(h: &ops::EngineHandler, name: String, file: String) -> Result<()> {
    let yaml = read_yaml(&file)?;
    let result = ops::stacks::create::run(h, &Claims::local_admin(), name, yaml).await?;
    println!("deployed stack `{}`", result.name);
    for id in result.container_ids {
        println!("  started {id}");
    }
    Ok(())
}

async fn list(h: &ops::EngineHandler) -> Result<()> {
    let stacks = ops::stacks::list::run(h).await?;
    if stacks.is_empty() {
        println!("(no stacks)");
        return Ok(());
    }
    let name_w = stacks.iter().map(|s| s.name.len()).max().unwrap_or(4).max(4);
    let status_w = stacks.iter().map(|s| s.status.len()).max().unwrap_or(6).max(6);
    println!(
        "{:<name_w$}  {:<status_w$}  {:>10}  MODIFIED",
        "NAME", "STATUS", "CONTAINERS"
    );
    for s in stacks {
        println!(
            "{:<name_w$}  {:<status_w$}  {:>10}  {}",
            s.name, s.status, s.container_count, s.modified_at
        );
    }
    Ok(())
}

async fn remove(h: &ops::EngineHandler, name: String, yes: bool) -> Result<()> {
    if !yes
        && !confirm(&format!(
            "remove stack `{name}`? containers will be stopped and the manifest deleted"
        ))?
    {
        println!("aborted");
        return Ok(());
    }
    ops::stacks::delete::run(h, &Claims::local_admin(), name.clone()).await?;
    println!("removed stack `{name}`");
    Ok(())
}

async fn redeploy(h: &ops::EngineHandler, name: String) -> Result<()> {
    let result = ops::stacks::redeploy::run(h, &Claims::local_admin(), name).await?;
    println!("redeployed stack `{}`", result.name);
    for id in result.container_ids {
        println!("  started {id}");
    }
    Ok(())
}

async fn logs(h: &ops::EngineHandler, name: String, follow: bool, tail: Option<u32>) -> Result<()> {
    let mut stream = ops::stacks::logs::run(h, name, follow, tail);
    while let Some(chunk) = stream.next().await {
        match chunk {
            StreamChunk::Log { stderr: is_err, data } => {
                if is_err {
                    let _ = std::io::stderr().write_all(data.as_bytes());
                } else {
                    let _ = std::io::stdout().write_all(data.as_bytes());
                }
            }
            StreamChunk::Lagging { dropped } => {
                let _ = writeln!(
                    std::io::stderr(),
                    "[nub: dropped {dropped} log chunks under backpressure]"
                );
            }
            StreamChunk::End { ok: false, err } => {
                let msg = err.unwrap_or_else(|| "stream ended without success".into());
                anyhow::bail!("stream ended: {msg}");
            }
            StreamChunk::End { ok: true, .. } => break,
            _ => {}
        }
    }
    Ok(())
}

fn read_yaml(file: &str) -> Result<String> {
    if file == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).context("reading stdin")?;
        return Ok(buf);
    }
    std::fs::read_to_string(file).with_context(|| format!("reading {file}"))
}

fn confirm(prompt: &str) -> Result<bool> {
    let stdin = std::io::stdin();
    anyhow::ensure!(
        stdin.is_terminal(),
        "refusing to prompt on a non-terminal stdin; pass --yes to skip"
    );
    eprint!("{prompt} [y/N]: ");
    std::io::stderr().flush().ok();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes"))
}
