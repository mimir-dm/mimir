//! Partial updates (PATCH bodies). A field that is absent stays as it is.
//! For a field that can be empty, `null` clears it: such a field is
//! `Option<Option<T>>` with [`double`] (absent = `None`, `null` =
//! `Some(None)`, a value = `Some(Some(v))`).

use serde::{Deserialize, Deserializer};

/// Deserialize `Option<Option<T>>` so that `null` is `Some(None)`.
pub fn double<'de, D, T>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d).map(Some)
}
