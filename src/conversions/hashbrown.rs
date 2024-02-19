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
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"hashbrown\"] }")]
//! ```
//!
//! Note that you must use compatible versions of hashbrown and PyO3.
//! The required hashbrown version may vary based on the version of PyO3.
use crate::{
    types::any::PyAnyMethods,
    types::dict::PyDictMethods,
    types::frozenset::PyFrozenSetMethods,
    types::set::{new_from_iter, PySetMethods},
    types::{IntoPyDict, PyDict, PyFrozenSet, PySet},
    Bound, FromPyObject, IntoPy, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject,
};
use std::{cmp, hash};

impl<K, V, H> ToPyObject for hashbrown::HashMap<K, V, H>
where
    K: hash::Hash + cmp::Eq + ToPyObject,
    V: ToPyObject,
    H: hash::BuildHasher,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        IntoPyDict::into_py_dict_bound(self, py).into()
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
        IntoPyDict::into_py_dict_bound(iter, py).into()
    }
}

impl<'py, K, V, S> FromPyObject<'py> for hashbrown::HashMap<K, V, S>
where
    K: FromPyObject<'py> + cmp::Eq + hash::Hash,
    V: FromPyObject<'py>,
    S: hash::BuildHasher + Default,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> Result<Self, PyErr> {
        let dict = ob.downcast::<PyDict>()?;
        let mut ret = hashbrown::HashMap::with_capacity_and_hasher(dict.len(), S::default());
        for (k, v) in dict.iter() {
            ret.insert(k.extract()?, v.extract()?);
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

impl<'py, K, S> FromPyObject<'py> for hashbrown::HashSet<K, S>
where
    K: FromPyObject<'py> + cmp::Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        match ob.downcast::<PySet>() {
            Ok(set) => set.iter().map(|any| any.extract()).collect(),
            Err(err) => {
                if let Ok(frozen_set) = ob.downcast::<PyFrozenSet>() {
                    frozen_set.iter().map(|any| any.extract()).collect()
                } else {
                    Err(PyErr::from(err))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::any::PyAnyMethods;

    #[test]
    fn test_hashbrown_hashmap_to_python() {
        Python::with_gil(|py| {
            let mut map = hashbrown::HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let m = map.to_object(py);
            let py_map = m.downcast_bound::<PyDict>(py).unwrap();

            assert!(py_map.len() == 1);
            assert!(
                py_map
                    .get_item(1)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
                    == 1
            );
            assert_eq!(map, py_map.extract().unwrap());
        });
    }
    #[test]
    fn test_hashbrown_hashmap_into_python() {
        Python::with_gil(|py| {
            let mut map = hashbrown::HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let m: PyObject = map.into_py(py);
            let py_map = m.downcast_bound::<PyDict>(py).unwrap();

            assert!(py_map.len() == 1);
            assert!(
                py_map
                    .get_item(1)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
                    == 1
            );
        });
    }

    #[test]
    fn test_hashbrown_hashmap_into_dict() {
        Python::with_gil(|py| {
            let mut map = hashbrown::HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict_bound(py);

            assert_eq!(py_map.len(), 1);
            assert_eq!(
                py_map
                    .get_item(1)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
        });
    }

    #[test]
    fn test_extract_hashbrown_hashset() {
        Python::with_gil(|py| {
            let set = PySet::new_bound(py, &[1, 2, 3, 4, 5]).unwrap();
            let hash_set: hashbrown::HashSet<usize> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());

            let set = PyFrozenSet::new_bound(py, &[1, 2, 3, 4, 5]).unwrap();
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
