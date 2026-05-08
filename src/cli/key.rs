//! `nub key` — manage the Ed25519 issuer keypair. `gen` is idempotent
//! (creates if missing, prints the public key either way); `rotate`
//! replaces the keypair, which invalidates every previously-issued
//! token.

use anyhow::{Context, Result};
use clap::Subcommand;

use crate::auth::Issuer;

#[derive(Subcommand)]
pub enum KeyCmd {
    /// Generate the keypair if missing; print the public key either way.
    Gen,
    /// Replace the keypair. Invalidates ALL previously-issued tokens.
    Rotate,
}

pub fn run(action: KeyCmd) -> Result<()> {
    match action {
        KeyCmd::Gen => gen_or_show(),
        KeyCmd::Rotate => rotate(),
    }
}

fn gen_or_show() -> Result<()> {
    let path = crate::config::default_issuer_key();
    let existed = path.exists();
    let issuer = Issuer::load_or_generate(&path)?;
    if existed {
        println!("issuer key at {}", path.display());
    } else {
        println!("wrote new issuer key to {}", path.display());
    }
    println!("public key (base64): {}", issuer.public_key_b64());
    Ok(())
}

fn rotate() -> Result<()> {
    let path = crate::config::default_issuer_key();
    if path.exists() {
        // Drop the old key. Any token signed by it stops verifying
        // immediately; clearing admin.jwt forces nub to mint a fresh
        // one on next start.
        std::fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
        let admin = crate::config::default_admin_jwt();
        if admin.exists() {
            std::fs::remove_file(&admin).ok();
        }
    }
    let issuer = Issuer::load_or_generate(&path)?;
    println!("wrote new issuer key to {}", path.display());
    println!("public key (base64): {}", issuer.public_key_b64());
    println!("rotated — restart nub to mint a fresh admin token (old tokens are invalid).");
    Ok(())
}
