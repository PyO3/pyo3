#![cfg(feature = "hashbrown")]

//!  Conversions to and from [hashbrown](https://docs.rs/hashbrown/)’s
//! `HashMap` and `HashSet`.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! # change * to the latest versions
//! hashbrown = "*"
// workaround for `extended_key_value_attributes`: https://github.com/rust-lang/rust/issues/82768#issuecomment-803935643
#![cfg_attr(docsrs, cfg_attr(docsrs, doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"hashbrown\"] }")))]
#![cfg_attr(
    not(docsrs),
    doc = "pyo3 = { version = \"*\", features = [\"hashbrown\"] }"
)]
//! ```
//!
//! Note that you must use compatible versions of hashbrown and PyO3.
//! The required hashbrown version may vary based on the version of PyO3.
use crate::{
    types::set::new_from_iter,
    types::{IntoPyDict, PyDict, PySet},
    FromPyObject, IntoPy, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject,
};
use std::{cmp, hash};

impl<K, V, H> ToPyObject for hashbrown::HashMap<K, V, H>
where
    K: hash::Hash + cmp::Eq + ToPyObject,
    V: ToPyObject,
    H: hash::BuildHasher,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        IntoPyDict::into_py_dict(self, py).into()
    }
}

impl<K, V, H> IntoPy<PyObject> for hashbrown::HashMap<K, V, H>
where
    K: hash::Hash + cmp::Eq + IntoPy<PyObject>,
    V: IntoPy<PyObject>,
    H: hash::BuildHasher,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        let iter = self
            .into_iter()
            .map(|(k, v)| (k.into_py(py), v.into_py(py)));
        IntoPyDict::into_py_dict(iter, py).into()
    }
}

impl<'source, K, V, S> FromPyObject<'source> for hashbrown::HashMap<K, V, S>
where
    K: FromPyObject<'source> + cmp::Eq + hash::Hash,
    V: FromPyObject<'source>,
    S: hash::BuildHasher + Default,
{
    fn extract(ob: &'source PyAny) -> Result<Self, PyErr> {
        let dict: &PyDict = ob.downcast()?;
        let mut ret = hashbrown::HashMap::with_capacity_and_hasher(dict.len(), S::default());
        for (k, v) in dict.iter() {
            ret.insert(K::extract(k)?, V::extract(v)?);
        }
        Ok(ret)
    }
}

impl<T> ToPyObject for hashbrown::HashSet<T>
where
    T: hash::Hash + Eq + ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        new_from_iter(py, self)
            .expect("Failed to create Python set from hashbrown::HashSet")
            .into()
    }
}

impl<K, S> IntoPy<PyObject> for hashbrown::HashSet<K, S>
where
    K: IntoPy<PyObject> + Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        new_from_iter(py, self.into_iter().map(|item| item.into_py(py)))
            .expect("Failed to create Python set from hashbrown::HashSet")
            .into()
    }
}

impl<'source, K, S> FromPyObject<'source> for hashbrown::HashSet<K, S>
where
    K: FromPyObject<'source> + cmp::Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let set: &PySet = ob.downcast()?;
        set.iter().map(K::extract).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashbrown_hashmap_to_python() {
        Python::with_gil(|py| {
            let mut map = hashbrown::HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let m = map.to_object(py);
            let py_map: &PyDict = m.downcast(py).unwrap();

            assert!(py_map.len() == 1);
            assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
            assert_eq!(map, py_map.extract().unwrap());
        });
    }
    #[test]
    fn test_hashbrown_hashmap_into_python() {
        Python::with_gil(|py| {
            let mut map = hashbrown::HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let m: PyObject = map.into_py(py);
            let py_map: &PyDict = m.downcast(py).unwrap();

            assert!(py_map.len() == 1);
            assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
        });
    }

    #[test]
    fn test_hashbrown_hashmap_into_dict() {
        Python::with_gil(|py| {
            let mut map = hashbrown::HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict(py);

            assert_eq!(py_map.len(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_extract_hashbrown_hashset() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1, 2, 3, 4, 5]).unwrap();
            let hash_set: hashbrown::HashSet<usize> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());
        });
    }

    #[test]
    fn test_hashbrown_hashset_into_py() {
        Python::with_gil(|py| {
            let hs: hashbrown::HashSet<u64> = [1, 2, 3, 4, 5].iter().cloned().collect();

            let hso: PyObject = hs.clone().into_py(py);

            assert_eq!(hs, hso.extract(py).unwrap());
        });
    }
}
