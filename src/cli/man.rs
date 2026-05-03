//! `nub man` — generate a man-page (groff) on stdout. Pipe to
//! `/usr/local/share/man/man1/nub.1` and run `mandb`.

use anyhow::{Context, Result};
use clap::CommandFactory;
use clap_mangen::Man;

use super::Args;

pub fn run() -> Result<()> {
    let cmd = Args::command();
    let man = Man::new(cmd);
    let mut out = std::io::stdout().lock();
    match man.render(&mut out) {
        Ok(()) => Ok(()),
        // Piping into `head` / `less` and quitting closes our stdout —
        // not an error worth surfacing.
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e).context("rendering man page"),
    }
}
