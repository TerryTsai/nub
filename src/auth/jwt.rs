//! Compact JWT (JWS) implementation tailored to nub's needs:
//! Ed25519 signatures (`alg=EdDSA`), the standard registered claims plus
//! an OAuth-style `scope` claim, and validation of `exp` / `nbf` / `aud`.
//!
//! Rolled by hand instead of pulling a JWT crate — the format is small
//! and the validation rules are easy to keep correct in 80 lines.

use anyhow::{anyhow, bail, Context, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde::{Deserialize, Serialize};

use super::issuer::Issuer;

/// JWT claims emitted by nub. Field order mirrors the registered IANA
/// claims (RFC 7519) followed by OAuth's `scope` (RFC 6749 §3.3).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub nbf: i64,
    pub iat: i64,
    /// Space-separated op names. `*` means all ops.
    pub scope: String,
}

impl Claims {
    /// True if the token authorizes invoking `op_name`.
    pub fn allows(&self, op_name: &str) -> bool {
        self.scope.split_ascii_whitespace().any(|s| s == "*" || s == op_name)
    }

    /// Scope claim split into individual op names.
    pub fn scopes(&self) -> Vec<String> {
        self.scope.split_ascii_whitespace().map(str::to_string).collect()
    }
}

/// Encode and sign a JWT.
pub fn encode(claims: &Claims, signer: &Issuer) -> Result<String> {
    let header = br#"{"alg":"EdDSA","typ":"JWT"}"#;
    let h64 = URL_SAFE_NO_PAD.encode(header);
    let p64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims)?);
    let signing_input = format!("{h64}.{p64}");
    let sig = signer.sign(signing_input.as_bytes())?;
    let s64 = URL_SAFE_NO_PAD.encode(&sig);
    Ok(format!("{signing_input}.{s64}"))
}

/// Verify signature, expiry, audience, and not-before. Returns the parsed
/// claims on success.
pub fn verify(token: &str, verifier: &Issuer, audience: &str) -> Result<Claims> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        bail!("malformed JWT");
    }
    // Header — only `alg` matters for nub. We refuse anything other than
    // EdDSA to dodge the alg-confusion footgun.
    let header_bytes = URL_SAFE_NO_PAD.decode(parts[0]).context("decoding header")?;
    let header: Header = serde_json::from_slice(&header_bytes).context("parsing header")?;
    if header.alg != "EdDSA" {
        bail!("unsupported alg: {}", header.alg);
    }
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let sig = URL_SAFE_NO_PAD.decode(parts[2]).context("decoding signature")?;
    verifier.verify(signing_input.as_bytes(), &sig)?;

    let payload_bytes = URL_SAFE_NO_PAD.decode(parts[1]).context("decoding payload")?;
    let claims: Claims = serde_json::from_slice(&payload_bytes).context("parsing payload")?;
    let now = current_unix_seconds();
    if claims.exp <= now {
        return Err(anyhow!("token expired"));
    }
    if claims.nbf > now + 60 {
        return Err(anyhow!("token not yet valid"));
    }
    if claims.aud != audience {
        return Err(anyhow!("audience mismatch (expected {audience}, got {})", claims.aud));
    }
    Ok(claims)
}

#[derive(Deserialize)]
struct Header {
    alg: String,
}

pub fn current_unix_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::rand::SystemRandom;
    use ring::signature::{Ed25519KeyPair, KeyPair as _};

    fn make_issuer() -> Issuer {
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let kp = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let _ = kp.public_key();
        Issuer::SelfManaged(kp)
    }

    fn fresh_claims(host: &str, ttl: i64) -> Claims {
        let now = current_unix_seconds();
        Claims {
            iss: "nub".into(),
            sub: "test".into(),
            aud: host.into(),
            exp: now + ttl,
            nbf: now,
            iat: now,
            scope: "*".into(),
        }
    }

    #[test]
    fn roundtrip_verifies() {
        let issuer = make_issuer();
        let claims = fresh_claims("host-a", 3600);
        let jwt = encode(&claims, &issuer).unwrap();
        let decoded = verify(&jwt, &issuer, "host-a").unwrap();
        assert_eq!(decoded.sub, "test");
        assert!(decoded.allows("anything"));
    }

    #[test]
    fn audience_mismatch_rejected() {
        let issuer = make_issuer();
        let claims = fresh_claims("host-a", 3600);
        let jwt = encode(&claims, &issuer).unwrap();
        assert!(verify(&jwt, &issuer, "host-b").is_err());
    }

    #[test]
    fn expired_rejected() {
        let issuer = make_issuer();
        let claims = fresh_claims("host-a", -10);
        let jwt = encode(&claims, &issuer).unwrap();
        assert!(verify(&jwt, &issuer, "host-a").is_err());
    }

    #[test]
    fn tampered_signature_rejected() {
        let issuer = make_issuer();
        let claims = fresh_claims("host-a", 3600);
        let jwt = encode(&claims, &issuer).unwrap();
        let mut bytes = jwt.into_bytes();
        let last = bytes.last_mut().unwrap();
        *last = if *last == b'A' { b'B' } else { b'A' };
        assert!(verify(std::str::from_utf8(&bytes).unwrap(), &issuer, "host-a").is_err());
    }

    #[test]
    fn scope_allows() {
        let mut claims = fresh_claims("host-a", 3600);
        claims.scope = "host_info list_containers".into();
        assert!(claims.allows("host_info"));
        assert!(claims.allows("list_containers"));
        assert!(!claims.allows("create_container"));

        let mut wild = fresh_claims("host-a", 3600);
        wild.scope = "*".into();
        assert!(wild.allows("anything"));
    }
}
