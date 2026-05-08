//! Secrets — age-encrypted blobs in a flat directory, one file per
//! secret, plus a hidden per-host X25519 identity. One file per Op
//! variant. The encryption layer (`crypto`), filesystem layer
//! (`store`), and deploy-time tmpfs runtime are siblings.

pub mod delete;
pub mod get;
pub mod list;
pub mod put;
pub mod runtime;

mod crypto;
mod store;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        TempDir::new().expect("tempdir")
    }

    #[tokio::test]
    async fn put_list_get_roundtrip() {
        let dir = tmp();
        put::run(dir.path(), "db_password", "hunter2").await.unwrap();
        let listing = list::run(dir.path()).await.unwrap();
        assert_eq!(listing.len(), 1);
        assert_eq!(listing[0].name, "db_password");
        assert!(listing[0].size > 0);

        let got = get::run(dir.path(), "db_password").await.unwrap();
        assert_eq!(got.name, "db_password");
        assert_eq!(got.value, "hunter2");
    }

    #[tokio::test]
    async fn put_overwrites() {
        let dir = tmp();
        put::run(dir.path(), "k", "first").await.unwrap();
        put::run(dir.path(), "k", "second").await.unwrap();
        let got = get::run(dir.path(), "k").await.unwrap();
        assert_eq!(got.value, "second");
    }

    #[tokio::test]
    async fn delete_removes() {
        let dir = tmp();
        put::run(dir.path(), "ephem", "x").await.unwrap();
        delete::run(dir.path(), "ephem").await.unwrap();
        let listing = list::run(dir.path()).await.unwrap();
        assert!(listing.is_empty());
    }

    #[tokio::test]
    async fn delete_missing_is_ok() {
        let dir = tmp();
        delete::run(dir.path(), "never_existed").await.unwrap();
    }

    #[tokio::test]
    async fn list_skips_identity_file() {
        let dir = tmp();
        put::run(dir.path(), "a", "v").await.unwrap();
        // .identity should now exist but never appear in listings.
        assert!(dir.path().join(".identity").exists());
        let listing = list::run(dir.path()).await.unwrap();
        assert_eq!(listing.len(), 1);
        assert_eq!(listing[0].name, "a");
    }

    #[tokio::test]
    async fn rejects_bad_names() {
        let dir = tmp();
        assert!(put::run(dir.path(), "../etc", "x").await.is_err());
        assert!(put::run(dir.path(), ".hidden", "x").await.is_err());
        assert!(put::run(dir.path(), "with space", "x").await.is_err());
        assert!(put::run(dir.path(), "", "x").await.is_err());
    }
}
