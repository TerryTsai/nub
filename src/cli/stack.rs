//! `nub stack` — CLI deploys for compose-shaped stacks. The phone UI is
//! the primary surface for stack ops; the CLI exists for codified /
//! scriptable deploys (`nub stack deploy app.yml` from CI or cron).

use anyhow::{Context, Result};
use std::io::Read;

use crate::config;
use crate::ops;

use super::StackCmd;

pub fn run(action: StackCmd) -> Result<()> {
    match action {
        StackCmd::Deploy { name, file } => deploy(name, file),
    }
}

fn deploy(name: String, file: String) -> Result<()> {
    let yaml = read_yaml(&file)?;
    let cfg = config::Config::load(None)?.unwrap_or_default();
    let policy = ops::Policy {
        allowed_binds: cfg.allowed_binds,
        dockerfiles_root: cfg.dockerfiles.unwrap_or_else(config::default_dockerfiles_dir),
        stacks_root: cfg.stacks.unwrap_or_else(config::default_stacks_dir),
    };
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(async move {
        let handler = ops::EngineHandler::connect(policy).await?;
        let result = ops::stacks::create::run(&handler, name, yaml).await?;
        println!("deployed stack `{}`", result.name);
        for id in result.container_ids {
            println!("  started {id}");
        }
        Ok::<_, anyhow::Error>(())
    })
}

fn read_yaml(file: &str) -> Result<String> {
    if file == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).context("reading stdin")?;
        return Ok(buf);
    }
    std::fs::read_to_string(file).with_context(|| format!("reading {file}"))
}
