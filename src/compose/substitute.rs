//! Compose-style variable substitution: `$VAR`, `${VAR}`, `${VAR:-default}`,
//! `${VAR-default}`. `$$` escapes to a literal `$`. Undefined vars without
//! a default raise an error rather than silently emitting empty string —
//! that footgun is the whole reason this module exists.

use std::collections::HashMap;

#[derive(Debug)]
pub(super) struct SubstituteError {
    pub var: String,
    pub message: String,
}

impl std::fmt::Display for SubstituteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${{{}}}: {}", self.var, self.message)
    }
}

pub(super) fn substitute(input: &str, env: &HashMap<String, String>) -> Result<String, SubstituteError> {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'$' {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        }
        let consumed = consume_dollar(bytes, i, env, &mut out)?;
        i = consumed;
    }
    Ok(out)
}

fn consume_dollar(
    bytes: &[u8],
    start: usize,
    env: &HashMap<String, String>,
    out: &mut String,
) -> Result<usize, SubstituteError> {
    if start + 1 >= bytes.len() {
        out.push('$');
        return Ok(start + 1);
    }
    match bytes[start + 1] {
        b'$' => {
            out.push('$');
            Ok(start + 2)
        }
        b'{' => consume_braced(bytes, start, env, out),
        _ => consume_bare(bytes, start, env, out),
    }
}

fn consume_braced(
    bytes: &[u8],
    start: usize,
    env: &HashMap<String, String>,
    out: &mut String,
) -> Result<usize, SubstituteError> {
    let body_start = start + 2;
    let end = bytes[body_start..].iter().position(|&b| b == b'}').map(|p| body_start + p).ok_or_else(|| {
        SubstituteError {
            var: String::from_utf8_lossy(&bytes[start..]).into_owned(),
            message: "missing closing brace".into(),
        }
    })?;
    let inner = std::str::from_utf8(&bytes[body_start..end]).unwrap_or("");
    let value = resolve_braced(inner, env)?;
    out.push_str(&value);
    Ok(end + 1)
}

fn consume_bare(
    bytes: &[u8],
    start: usize,
    env: &HashMap<String, String>,
    out: &mut String,
) -> Result<usize, SubstituteError> {
    let name_start = start + 1;
    let end = identifier_end(bytes, name_start);
    if end == name_start {
        out.push('$');
        return Ok(start + 1);
    }
    let name = std::str::from_utf8(&bytes[name_start..end]).unwrap_or("");
    match env.get(name) {
        Some(v) => out.push_str(v),
        None => return Err(SubstituteError { var: name.into(), message: "is not set".into() }),
    }
    Ok(end)
}

fn identifier_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    i
}

fn resolve_braced(inner: &str, env: &HashMap<String, String>) -> Result<String, SubstituteError> {
    if let Some(idx) = inner.find(":-") {
        let (name, default) = (&inner[..idx], &inner[idx + 2..]);
        return Ok(env.get(name).cloned().unwrap_or_else(|| default.into()));
    }
    if let Some(idx) = inner.find('-') {
        let (name, default) = (&inner[..idx], &inner[idx + 1..]);
        return Ok(env.get(name).cloned().unwrap_or_else(|| default.into()));
    }
    env.get(inner).cloned().ok_or_else(|| SubstituteError {
        var: inner.into(),
        message: "is not set".into(),
    })
}
