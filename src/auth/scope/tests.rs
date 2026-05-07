use super::*;
use std::collections::HashSet;

#[test]
fn every_scope_has_unique_string() {
    let mut seen = HashSet::new();
    for &s in Scope::ALL {
        assert!(seen.insert(s.as_str()), "duplicate scope string: {}", s.as_str());
    }
}

#[test]
fn every_scope_is_resource_colon_action() {
    for &s in Scope::ALL {
        let str_form = s.as_str();
        assert!(str_form.contains(':'), "scope `{str_form}` missing `:`");
        assert!(
            !str_form.starts_with(':') && !str_form.ends_with(':'),
            "scope `{str_form}` has empty resource or action"
        );
        let r = s.resource();
        assert!(str_form.starts_with(r));
        assert_eq!(str_form.as_bytes()[r.len()], b':');
    }
}

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
fn from_str_roundtrip() {
    for &s in Scope::ALL {
        let parsed: Scope = s.as_str().parse().unwrap();
        assert_eq!(parsed, s);
    }
}

#[test]
fn from_str_rejects_unknown() {
    assert!("nonsense".parse::<Scope>().is_err());
    assert!("containers:nope".parse::<Scope>().is_err());
    assert!("".parse::<Scope>().is_err());
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
