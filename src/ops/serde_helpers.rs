//! Shared serde adapters for engine-shape decoding.

use serde::Deserialize;

/// Deserialize, treating JSON `null` as the type's `Default`. Pair with
/// `#[serde(default)]` so missing fields also default. Use on `HashMap`
/// or `Vec` fields where the engine may emit `null`.
pub(super) fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
