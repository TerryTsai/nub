//! `nub token` — token operations. Today: `mint` and `scopes`.

use anyhow::{anyhow, bail, Context, Result};

use crate::auth::{jwt, scope, Issuer};

use super::TokenCmd;

pub fn run(action: TokenCmd) -> Result<()> {
    match action {
        TokenCmd::Mint {
            sub,
            scope,
            preset,
            expires,
            aud,
        } => mint(sub, scope, preset, expires, aud),
        TokenCmd::Scopes => print_scopes(),
    }
}

fn mint(
    sub: String,
    scope_arg: Option<String>,
    preset_arg: Option<String>,
    expires: String,
    audience: Option<String>,
) -> Result<()> {
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
    let scope_str = resolve_scope(scope_arg, preset_arg)?;
    let ttl = parse_duration(&expires)?;
    let now = jwt::current_unix_seconds();
    let claims = jwt::Claims {
        iss: "nub".into(),
        sub,
        aud,
        exp: now + ttl,
        nbf: now,
        iat: now,
        scope: scope_str,
    };
    let token = jwt::encode(&claims, &issuer)?;
    println!("{token}");
    Ok(())
}

/// Resolve `--scope` / `--preset` (mutually exclusive at the CLI layer)
/// into the literal scope string embedded in the JWT.
///
/// Default (neither flag) is the `admin` preset, preserving prior UX
/// where `nub token mint --sub foo` produced a wildcard token.
fn resolve_scope(scope_arg: Option<String>, preset_arg: Option<String>) -> Result<String> {
    if let Some(name) = preset_arg {
        return preset_to_scope_string(&name);
    }
    if let Some(raw) = scope_arg {
        // Accept comma OR whitespace as separators for human convenience;
        // normalize to single spaces in the JWT.
        let normalized: String = raw
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if normalized.is_empty() {
            bail!("--scope was empty");
        }
        if let Err(bad) = scope::validate_string(&normalized) {
            bail!(
                "unknown scope(s): {}. Run `nub token scopes` for the valid set.",
                bad.join(", ")
            );
        }
        return Ok(normalized);
    }
    // Default: admin preset.
    preset_to_scope_string("admin")
}

fn preset_to_scope_string(name: &str) -> Result<String> {
    match name {
        "admin" => Ok(scope::presets::ADMIN_LITERAL.to_string()),
        "phone" => Ok(scope::join_scopes(scope::presets::PHONE)),
        "readonly" | "read-only" => Ok(scope::join_scopes(scope::presets::READONLY)),
        other => bail!("unknown preset `{other}`. Valid: admin, phone, readonly"),
    }
}

fn print_scopes() -> Result<()> {
    println!("Scopes (one per network-exposed op):");
    for s in scope::Scope::ALL {
        println!("  {s}");
    }
    println!();
    println!("Wildcards:");
    println!("  *              all scopes");
    println!("  <resource>:*   all actions on a resource");
    println!();
    println!("Presets:");
    println!("  admin     → *");
    println!(
        "  phone     → {} scopes (everyday operator surface; no secrets:reveal)",
        scope::presets::PHONE.len()
    );
    println!(
        "  readonly  → {} scopes (state-changing ops excluded)",
        scope::presets::READONLY.len()
    );
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
