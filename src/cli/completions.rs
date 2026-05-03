//! `nub completions <shell>` — emit a shell completion script on stdout.

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

use super::Args;

pub fn run(shell: Shell) -> Result<()> {
    let mut cmd = Args::command();
    generate(shell, &mut cmd, "nub", &mut std::io::stdout());
    Ok(())
}
