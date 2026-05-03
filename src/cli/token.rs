//! `nub token` — token operations. Today: `mint` only. Future slots:
//! `list` and `revoke` (need a JWT-revocation backend; reserved here
//! so the noun-verb shape is stable).

use anyhow::{anyhow, Context, Result};

use crate::auth::{jwt, Issuer};

use super::TokenCmd;

pub fn run(action: TokenCmd) -> Result<()> {
    match action {
        TokenCmd::Mint {
            sub,
            scope,
            expires,
            aud,
        } => mint(sub, scope, expires, aud),
    }
}

fn mint(sub: String, scope: String, expires: String, audience: Option<String>) -> Result<()> {
    let issuer_path = crate::config::default_issuer_key();
    let issuer = Issuer::load_or_generate(&issuer_path)?;
    if !issuer.can_mint() {
        return Err(anyhow!(
            "nub holds only a verifying public key (trusted_issuer is set); \
             mint tokens with the corresponding private key elsewhere"
        ));
    }
    let aud = match audience {
        Some(a) => a,
        None => crate::cli::hostname(),
    };
    let ttl = parse_duration(&expires)?;
    let now = jwt::current_unix_seconds();
    let claims = jwt::Claims {
        iss: "nub".into(),
        sub,
        aud,
        exp: now + ttl,
        nbf: now,
        iat: now,
        scope,
    };
    let token = jwt::encode(&claims, &issuer)?;
    println!("{token}");
    Ok(())
}

/// Parse a duration like `90d`, `12h`, `30m`, `1y`. Bare integer = seconds.
fn parse_duration(s: &str) -> Result<i64> {
    let s = s.trim();
    if s.is_empty() {
        return Err(anyhow!("empty duration"));
    }
    let (num_part, unit) = match s.chars().last() {
        Some(c) if c.is_ascii_alphabetic() => (&s[..s.len() - 1], c),
        _ => (s, 's'),
    };
    let n: i64 = num_part.parse().with_context(|| format!("parsing duration {s:?}"))?;
    let secs = match unit.to_ascii_lowercase() {
        's' => n,
        'm' => n * 60,
        'h' => n * 3600,
        'd' => n * 86400,
        'w' => n * 86400 * 7,
        'y' => n * 86400 * 365,
        c => return Err(anyhow!("unknown duration unit {c:?}; use s|m|h|d|w|y")),
    };
    if secs <= 0 {
        return Err(anyhow!("duration must be positive"));
    }
    Ok(secs)
}

#[cfg(test)]
mod tests {
    use super::parse_duration;

    #[test]
    fn parses_units() {
        assert_eq!(parse_duration("60").unwrap(), 60);
        assert_eq!(parse_duration("60s").unwrap(), 60);
        assert_eq!(parse_duration("5m").unwrap(), 300);
        assert_eq!(parse_duration("1h").unwrap(), 3600);
        assert_eq!(parse_duration("1d").unwrap(), 86400);
        assert_eq!(parse_duration("1w").unwrap(), 604800);
        assert_eq!(parse_duration("1y").unwrap(), 31_536_000);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("1z").is_err());
        assert!(parse_duration("0d").is_err());
    }
}
