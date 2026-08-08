#![cfg(feature = "serde")]

//! Enables (de)serialization of [`Py`]`<T>` objects via [serde](https://docs.rs/serde).
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

use crate::platform::prelude::*;
use crate::{Py, PyAny, PyClass, Python};
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};

impl<T> Serialize for Py<T>
where
    T: Serialize + PyClass,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        Python::attach(|py| {
            self.try_borrow(py)
                .map_err(|e| ser::Error::custom(e.to_string()))?
                .serialize(serializer)
        })
    }
}

impl<'de, T> Deserialize<'de> for Py<T>
where
    T: PyClass<BaseType = PyAny> + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Py<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserialized = T::deserialize(deserializer)?;

        Python::attach(|py| Py::new(py, deserialized).map_err(|e| de::Error::custom(e.to_string())))
    }
}

#[cfg(all(test, feature = "macros"))]
mod tests {
    use crate::prelude::*;

    use serde::{Deserialize, Serialize};

    #[pyclass(crate = "crate")]
    #[derive(Debug, Serialize, Deserialize)]
    struct Group {
        name: std::string::String,
    }

    #[pyclass(crate = "crate")]
    #[derive(Debug, Serialize, Deserialize)]
    struct User {
        username: std::string::String,
        group: Option<Py<Group>>,
        friends: std::vec::Vec<Py<User>>,
    }

    #[test]
    fn test_serialize() {
        let friend1 = User {
            username: "friend 1".into(),
            group: None,
            friends: vec![],
        };
        let friend2 = User {
            username: "friend 2".into(),
            group: None,
            friends: vec![],
        };

        let user = Python::attach(|py| {
            let py_friend1 = Py::new(py, friend1).expect("failed to create friend 1");
            let py_friend2 = Py::new(py, friend2).expect("failed to create friend 2");

            let friends = vec![py_friend1, py_friend2];
            let py_group = Py::new(
                py,
                Group {
                    name: "group name".into(),
                },
            )
            .unwrap();

            User {
                username: "danya".into(),
                group: Some(py_group),
                friends,
            }
        });

        let serialized = serde_json::to_string(&user).expect("failed to serialize");
        assert_eq!(
            serialized,
            r#"{"username":"danya","group":{"name":"group name"},"friends":[{"username":"friend 1","group":null,"friends":[]},{"username":"friend 2","group":null,"friends":[]}]}"#
        );
    }

    #[test]
    fn test_deserialize() {
        let serialized = r#"{"username": "danya", "friends":
            [{"username": "friend", "group": {"name": "danya's friends"}, "friends": []}]}"#;
        let user: User = serde_json::from_str(serialized).expect("failed to deserialize");

        assert_eq!(user.username, "danya");
        assert!(user.group.is_none());
        assert_eq!(user.friends.len(), 1usize);
        let friend = user.friends.first().unwrap();

        Python::attach(|py| {
            assert_eq!(friend.borrow(py).username, "friend");
            assert_eq!(
                friend.borrow(py).group.as_ref().unwrap().borrow(py).name,
                "danya's friends"
            )
        });
    }
}
