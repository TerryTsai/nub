//! Filesystem-name validator shared by Dockerfile and secret names. The
//! stricter stack-name validator lives next to its caller in
//! `ops::stacks::store`.

/// FS-name validator shared by Dockerfile and secret names. Letters,
/// digits, `.`, `_`, `-`. Bans path separators, traversal, leading
/// dot/hyphen, embedded NUL. Capped at 128 chars.
pub(super) fn valid_fs_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 128 {
        return false;
    }
    if name.starts_with('.') || name.starts_with('-') {
        return false;
    }
    if name == "." || name == ".." {
        return false;
    }
    name.as_bytes()
        .iter()
        .all(|&b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_fs_name_accepts_typical_shapes() {
        assert!(valid_fs_name("nginx"));
        assert!(valid_fs_name("nginx.Dockerfile"));
        assert!(valid_fs_name("foo_bar-1"));
        assert!(valid_fs_name("db_password"));
        assert!(valid_fs_name("API_KEY"));
        assert!(valid_fs_name("a.b-c_1"));
        assert!(valid_fs_name("1nginx"));
        assert!(valid_fs_name("9up"));
        assert!(valid_fs_name(&"a".repeat(128)));
    }

    #[test]
    fn valid_fs_name_rejects_traversal_and_specials() {
        assert!(!valid_fs_name(""));
        assert!(!valid_fs_name("."));
        assert!(!valid_fs_name(".."));
        assert!(!valid_fs_name(".hidden"));
        assert!(!valid_fs_name(".identity"));
        assert!(!valid_fs_name("-leading"));
        assert!(!valid_fs_name("a/b"));
        assert!(!valid_fs_name("a\0b"));
        assert!(!valid_fs_name("x y"));
        assert!(!valid_fs_name(&"a".repeat(129)));
    }

    #[test]
    fn valid_fs_name_rejects_unicode() {
        assert!(!valid_fs_name("café"));
        assert!(!valid_fs_name("nginx™"));
    }
}
