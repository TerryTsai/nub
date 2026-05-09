//! Ed25519 issuer keypair — the signing identity nub trusts.
//!
//! Two modes:
//! - **Self-managed:** nub generates and persists its own keypair at
//!   `$XDG_DATA_HOME/nub/issuer.key` (PKCS#8 binary, mode 600). Can both
//!   sign (for `nub mint`) and verify.
//! - **External:** the operator supplies a public key in the config
//!   (`trusted_issuer = "<base64 ed25519 pubkey>"`); nub can only verify,
//!   not mint. Tokens come from somewhere else (a CLI on the operator's
//!   laptop, latch, etc.).

use std::os::unix::fs::OpenOptionsExt as _;
use std::path::Path;

use anyhow::{anyhow, ensure, Context, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};

/// Verifies signatures. Optionally signs (when nub holds the private key).
pub enum Issuer {
    /// nub owns the keypair — auto-generated mode.
    SelfManaged(Ed25519KeyPair),
    /// nub only has the public key — external issuer mode.
    External(Vec<u8>),
}

impl Issuer {
    /// Load (or generate-and-load) nub's own issuer keypair from disk.
    /// File mode is forced to 0600 on creation.
    pub fn load_or_generate(path: &Path) -> Result<Self> {
        if path.exists() {
            let pkcs8 = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
            let kp = Ed25519KeyPair::from_pkcs8(&pkcs8)
                .map_err(|e| anyhow!("parsing issuer key {}: {e}", path.display()))?;
            return Ok(Self::SelfManaged(kp));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).map_err(|e| anyhow!("generating ed25519 key: {e}"))?;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .with_context(|| format!("creating {}", path.display()))?;
        std::io::Write::write_all(&mut f, pkcs8.as_ref())?;
        let kp =
            Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).map_err(|e| anyhow!("parsing freshly-generated key: {e}"))?;
        Ok(Self::SelfManaged(kp))
    }

    /// Build a verify-only issuer from a base64url-encoded raw public key.
    pub fn from_public_key_b64(b64: &str) -> Result<Self> {
        let raw = URL_SAFE_NO_PAD.decode(b64.trim()).context("decoding base64 public key")?;
        ensure!(raw.len() == 32, "expected 32-byte ed25519 public key, got {}", raw.len());
        Ok(Self::External(raw))
    }

    pub fn public_key_bytes(&self) -> &[u8] {
        match self {
            Self::SelfManaged(kp) => kp.public_key().as_ref(),
            Self::External(b) => b.as_slice(),
        }
    }

    /// Public key as base64url, suitable for sharing or pasting into config.
    pub fn public_key_b64(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.public_key_bytes())
    }

    pub fn sign(&self, msg: &[u8]) -> Result<Vec<u8>> {
        match self {
            Self::SelfManaged(kp) => Ok(kp.sign(msg).as_ref().to_vec()),
            Self::External(_) => Err(anyhow!(
                "cannot sign: nub is configured with a trusted_issuer (verify-only)"
            )),
        }
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> Result<()> {
        UnparsedPublicKey::new(&ED25519, self.public_key_bytes())
            .verify(msg, sig)
            .map_err(|_| anyhow!("invalid signature"))
    }

    /// True if nub can mint new tokens (i.e. holds the private key).
    pub fn can_mint(&self) -> bool {
        matches!(self, Self::SelfManaged(_))
    }
}
