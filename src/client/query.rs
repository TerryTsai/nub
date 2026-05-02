//! Tiny URL-encoded query string builder. Hand-rolled to avoid pulling in a
//! URL crate just for `?all=true&follow=true`-shaped strings.

use std::fmt::Write as _;

#[derive(Default)]
pub(crate) struct Query {
    out: String,
}

impl Query {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push(&mut self, key: &str, value: &str) -> &mut Self {
        let sep = if self.out.is_empty() { '?' } else { '&' };
        self.out.push(sep);
        encode_into(&mut self.out, key);
        self.out.push('=');
        encode_into(&mut self.out, value);
        self
    }

    pub(crate) fn push_bool(&mut self, key: &str, value: bool) -> &mut Self {
        self.push(key, if value { "true" } else { "false" })
    }

    pub(crate) fn finish(self) -> String {
        self.out
    }
}

fn encode_into(out: &mut String, s: &str) {
    for b in s.bytes() {
        if matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
}
