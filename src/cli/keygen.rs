//! `nub keygen` — generate (or rotate) the issuer keypair, plus
//! print the corresponding public key.

use anyhow::{Context, Result};

use crate::auth::Issuer;

pub fn run(rotate: bool) -> Result<()> {
    let path = crate::config::default_issuer_key();
    if path.exists() {
        if !rotate {
            let issuer = Issuer::load_or_generate(&path)?;
            println!("issuer key already exists at {}", path.display());
            println!("public key (base64): {}", issuer.public_key_b64());
            println!("(pass --rotate to replace it; this invalidates ALL tokens)");
            return Ok(());
        }
        // Rotation: drop the old key file. Any token signed by the old
        // key stops verifying immediately; bootstrap a fresh admin token
        // by also removing $XDG_DATA_HOME/nub/admin.jwt.
        std::fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
        let admin = crate::config::default_admin_jwt();
        if admin.exists() {
            std::fs::remove_file(&admin).ok();
        }
    }
    let issuer = Issuer::load_or_generate(&path)?;
    println!("wrote new issuer key to {}", path.display());
    println!("public key (base64): {}", issuer.public_key_b64());
    if rotate {
        println!("rotated — restart nub to mint a fresh admin token (old tokens are invalid)");
    }
    Ok(())
}
