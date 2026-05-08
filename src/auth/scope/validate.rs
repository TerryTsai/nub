//! Free functions on scope strings — token/string validation, the
//! per-token grant check the runtime uses, and the `Vec<Scope>` →
//! `String` joiner used at mint time.

use super::types::{ParseScopeError, Scope};

/// Resources that admit `<resource>:*` wildcards. Kept in sync with
/// `Scope::ALL` by `tests::resources_match_scopes`.
pub const RESOURCES: &[&str] = &[
    "host",
    "auth",
    "containers",
    "images",
    "volumes",
    "networks",
    "dockerfiles",
    "stacks",
    "secrets",
];

/// Validate one scope token (one word from a JWT `scope` claim or
/// `--scope` arg). Accepts `*`, `<resource>:*`, or any concrete scope.
pub fn validate_token(tok: &str) -> Result<(), ParseScopeError> {
    if tok == "*" {
        return Ok(());
    }
    if tok.matches(':').count() != 1 {
        return Err(ParseScopeError(tok.to_string()));
    }
    let (res, action) = tok.split_once(':').unwrap();
    if res.is_empty() || action.is_empty() {
        return Err(ParseScopeError(tok.to_string()));
    }
    if action == "*" {
        if RESOURCES.contains(&res) {
            return Ok(());
        }
        return Err(ParseScopeError(tok.to_string()));
    }
    tok.parse::<Scope>().map(|_| ())
}

/// Validate a whole space- or comma-separated scope string. Returns
/// the list of unknown tokens if any are bad.
pub fn validate_string(s: &str) -> Result<(), Vec<String>> {
    let bad: Vec<String> = s
        .split_ascii_whitespace()
        .filter(|t| validate_token(t).is_err())
        .map(str::to_string)
        .collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(bad)
    }
}

/// Does any token in the granted scope string authorize `needed`?
pub fn granted_allows(granted: &str, needed: Scope) -> bool {
    let needed_str = needed.as_str();
    let needed_resource = needed.resource();
    granted.split_ascii_whitespace().any(|tok| {
        if tok == "*" || tok == needed_str {
            return true;
        }
        match tok.split_once(':') {
            Some((res, "*")) => res == needed_resource,
            _ => false,
        }
    })
}

/// Render a `&[Scope]` as a single space-separated string suitable for
/// embedding in a JWT `scope` claim.
pub fn join_scopes(scopes: &[Scope]) -> String {
    let mut out = String::new();
    for (i, s) in scopes.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(s.as_str());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::presets;
    use super::*;

    #[test]
    fn resources_match_scopes() {
        let mut scope_resources: Vec<&str> = Scope::ALL.iter().map(|s| s.resource()).collect();
        scope_resources.sort();
        scope_resources.dedup();
        let mut listed = RESOURCES.to_vec();
        listed.sort();
        assert_eq!(scope_resources, listed);
    }

    #[test]
    fn validate_token_accepts_wildcards() {
        assert!(validate_token("*").is_ok());
        assert!(validate_token("containers:*").is_ok());
        assert!(validate_token("secrets:*").is_ok());
    }

    #[test]
    fn validate_token_rejects_unknown_resource_wildcard() {
        assert!(validate_token("widgets:*").is_err());
    }

    #[test]
    fn validate_token_rejects_garbage() {
        assert!(validate_token("").is_err());
        assert!(validate_token(":").is_err());
        assert!(validate_token("containers:").is_err());
        assert!(validate_token(":get").is_err());
        assert!(validate_token("containers:list:extra").is_err());
    }

    #[test]
    fn validate_string_collects_bad_tokens() {
        let r = validate_string("containers:list bogus stacks:get also_bogus");
        assert_eq!(r.unwrap_err(), vec!["bogus", "also_bogus"]);
    }

    #[test]
    fn granted_allows_exact() {
        assert!(granted_allows("containers:list", Scope::ContainersList));
        assert!(!granted_allows("containers:list", Scope::ContainersGet));
    }

    #[test]
    fn granted_allows_admin_wildcard() {
        for &s in Scope::ALL {
            assert!(granted_allows("*", s));
        }
    }

    #[test]
    fn granted_allows_resource_wildcard() {
        assert!(granted_allows("containers:*", Scope::ContainersExec));
        assert!(granted_allows("secrets:*", Scope::SecretsReveal));
        assert!(!granted_allows("containers:*", Scope::ImagesList));
    }

    #[test]
    fn granted_allows_multi_token() {
        let granted = "containers:list stacks:get secrets:put";
        assert!(granted_allows(granted, Scope::ContainersList));
        assert!(granted_allows(granted, Scope::StacksGet));
        assert!(granted_allows(granted, Scope::SecretsPut));
        assert!(!granted_allows(granted, Scope::SecretsReveal));
    }

    #[test]
    fn presets_dont_grant_secrets_reveal() {
        assert!(!presets::OPERATOR.contains(&Scope::SecretsReveal));
        assert!(!presets::DEPLOY.contains(&Scope::SecretsReveal));
        assert!(!presets::READONLY.contains(&Scope::SecretsReveal));
    }

    #[test]
    fn join_scopes_format() {
        let s = join_scopes(&[Scope::ContainersList, Scope::StacksGet]);
        assert_eq!(s, "containers:list stacks:get");
        assert_eq!(join_scopes(&[]), "");
    }
}
