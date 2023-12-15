#![cfg(feature = "serde")]

//! Enables (de)serialization of [`PyDetached`]`<T>` objects via [serde](https://docs.rs/serde).
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"serde\"] }")]
//! serde = "1.0"
//! ```

use crate::{PyAny, PyClass, PyDetached, Python};
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};

impl<T> Serialize for PyDetached<T>
where
    T: Serialize + PyClass,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        Python::with_gil(|py| {
            self.try_borrow(py)
                .map_err(|e| ser::Error::custom(e.to_string()))?
                .serialize(serializer)
        })
    }
}

impl<'de, T> Deserialize<'de> for PyDetached<T>
where
    T: PyClass<BaseType = PyAny> + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<PyDetached<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserialized = T::deserialize(deserializer)?;

        Python::with_gil(|py| {
            PyDetached::new(py, deserialized).map_err(|e| de::Error::custom(e.to_string()))
        })
    }
}
