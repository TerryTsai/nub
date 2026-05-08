//! XDG path resolution and the per-resource defaults nub reads if the
//! TOML config doesn't override them.

use std::path::PathBuf;

/// `$XDG_CONFIG_HOME` if set, else `$HOME/.config`, else `./.config`.
pub fn xdg_config_home() -> PathBuf {
    if let Some(x) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(x);
    }
    if let Some(h) = std::env::var_os("HOME") {
        return PathBuf::from(h).join(".config");
    }
    PathBuf::from(".config")
}

/// `$XDG_DATA_HOME` if set, else `$HOME/.local/share`, else `./.local/share`.
pub fn xdg_data_home() -> PathBuf {
    if let Some(x) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(x);
    }
    if let Some(h) = std::env::var_os("HOME") {
        return PathBuf::from(h).join(".local/share");
    }
    PathBuf::from(".local/share")
}

/// Default dockerfiles directory: `$XDG_DATA_HOME/nub/dockerfiles`.
pub fn default_dockerfiles_dir() -> PathBuf {
    xdg_data_home().join("nub/dockerfiles")
}

/// Default stacks directory: `$XDG_DATA_HOME/nub/stacks`. One subdir
/// per stack with the YAML manifest inside.
pub fn default_stacks_dir() -> PathBuf {
    xdg_data_home().join("nub/stacks")
}

/// Default secrets directory: `$XDG_DATA_HOME/nub/secrets`. Holds the
/// age `.identity` file plus one `<name>.age` blob per secret.
pub fn default_secrets_dir() -> PathBuf {
    xdg_data_home().join("nub/secrets")
}

/// Default issuer key path: `$XDG_DATA_HOME/nub/issuer.key`. PKCS#8
/// binary; written mode 600 by `Issuer::load_or_generate`.
pub fn default_issuer_key() -> PathBuf {
    xdg_data_home().join("nub/issuer.key")
}

/// Default admin JWT path: `$XDG_DATA_HOME/nub/admin.jwt`. Persisted
/// once at first run; re-printed on subsequent starts so paired
/// clients (browsers, scripts, anything holding the connect URL)
/// survive restart.
pub fn default_admin_jwt() -> PathBuf {
    xdg_data_home().join("nub/admin.jwt")
}
