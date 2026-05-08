//! Persistent configuration: a flat TOML file plus path helpers for
//! the locations nub reads and writes (XDG config/data dirs, issuer
//! key, admin token, dockerfiles/stacks/secrets roots).

mod discover;
mod file;
mod paths;

pub use discover::locate_path;
pub use file::Config;
pub use paths::{
    default_admin_jwt, default_dockerfiles_dir, default_issuer_key, default_secrets_dir, default_stacks_dir,
    xdg_config_home, xdg_data_home,
};
