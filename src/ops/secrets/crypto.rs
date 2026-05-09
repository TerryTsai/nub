//! Encryption layer for secrets — age (https://age-encryption.org/v1)
//! with a per-host X25519 identity stored at `<secrets_root>/.identity`.
//!
//! Threat model: at-rest encryption against backup leaks, accidental
//! `git add`, and non-root filesystem reads. Root-on-host can read the
//! identity file and decrypt, by design — same posture as Docker Swarm
//! workers, Kubernetes nodes, Vault Agent, et al. (See README/security.)

use std::io::{Read as _, Write as _};
use std::path::Path;

use age::secrecy::ExposeSecret as _;
use age::x25519;
use anyhow::{anyhow, bail, Context, Result};
use tokio::fs;

use super::store;

/// Load the per-host identity, generating it on first access. The file
/// is mode 0600. Returns the parsed identity ready for decryption.
pub async fn load_or_generate_identity(root: &Path) -> Result<x25519::Identity> {
    let path = store::identity_path(root);
    if path.exists() {
        let s = fs::read_to_string(&path).await.with_context(|| format!("reading {}", path.display()))?;
        return s
            .trim()
            .parse::<x25519::Identity>()
            .map_err(|e| anyhow!("parsing identity at {}: {e}", path.display()));
    }
    fs::create_dir_all(root).await.ok();
    let id = x25519::Identity::generate();
    let serialized = id.to_string();
    write_identity(&path, serialized.expose_secret()).await?;
    Ok(id)
}

#[cfg(unix)]
async fn write_identity(path: &Path, contents: &str) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .with_context(|| format!("creating {}", path.display()))?;
    f.write_all(contents.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}

#[cfg(not(unix))]
async fn write_identity(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents).await?;
    Ok(())
}

/// Encrypt `plaintext` to the identity's public recipient. Output is
/// the binary age format (not armored — saves a few bytes and we never
/// transmit these blobs through text-only channels).
pub fn encrypt(identity: &x25519::Identity, plaintext: &[u8]) -> Result<Vec<u8>> {
    let recipient = identity.to_public();
    let encryptor = age::Encryptor::with_recipients(vec![Box::new(recipient)])
        .ok_or_else(|| anyhow!("age encryptor refused recipient (no recipients?)"))?;
    let mut out = Vec::with_capacity(plaintext.len() + 256);
    let mut writer = encryptor.wrap_output(&mut out).context("starting age encryption")?;
    writer.write_all(plaintext).context("writing plaintext to age stream")?;
    writer.finish().context("finalizing age stream")?;
    Ok(out)
}

pub fn decrypt(identity: &x25519::Identity, blob: &[u8]) -> Result<Vec<u8>> {
    let decryptor = match age::Decryptor::new(blob).context("opening age blob")? {
        age::Decryptor::Recipients(d) => d,
        age::Decryptor::Passphrase(_) => bail!("unexpected passphrase-encrypted blob"),
    };
    let mut reader =
        decryptor.decrypt(std::iter::once(identity as &dyn age::Identity)).context("starting age decryption")?;
    let mut out = Vec::new();
    reader.read_to_end(&mut out).context("reading age plaintext")?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let id = x25519::Identity::generate();
        let plain = b"hunter2";
        let blob = encrypt(&id, plain).unwrap();
        // age blobs are non-empty and not the plaintext.
        assert!(!blob.is_empty());
        assert_ne!(blob.as_slice(), plain.as_slice());
        let back = decrypt(&id, &blob).unwrap();
        assert_eq!(back.as_slice(), plain.as_slice());
    }

    #[test]
    fn wrong_identity_cannot_decrypt() {
        let writer = x25519::Identity::generate();
        let attacker = x25519::Identity::generate();
        let blob = encrypt(&writer, b"sealed").unwrap();
        assert!(decrypt(&attacker, &blob).is_err());
    }
}
