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
    conversion::IntoPyObject,
    types::{
        any::PyAnyMethods,
        dict::PyDictMethods,
        frozenset::PyFrozenSetMethods,
        set::{try_new_from_iter, PySetMethods},
        PyDict, PyFrozenSet, PySet,
    },
    Bound, FromPyObject, PyAny, PyErr, PyResult, Python,
};
use std::{cmp, hash};

impl<'py, K, V, H> IntoPyObject<'py> for hashbrown::HashMap<K, V, H>
where
    K: IntoPyObject<'py> + cmp::Eq + hash::Hash,
    V: IntoPyObject<'py>,
    H: hash::BuildHasher,
{
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        for (k, v) in self {
            dict.set_item(k, v)?;
        }
        Ok(dict)
    }
}

impl<'a, 'py, K, V, H> IntoPyObject<'py> for &'a hashbrown::HashMap<K, V, H>
where
    &'a K: IntoPyObject<'py> + cmp::Eq + hash::Hash,
    &'a V: IntoPyObject<'py>,
    H: hash::BuildHasher,
{
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        for (k, v) in self {
            dict.set_item(k, v)?;
        }
        Ok(dict)
    }
}

impl<'py, K, V, S> FromPyObject<'py> for hashbrown::HashMap<K, V, S>
where
    K: FromPyObject<'py> + cmp::Eq + hash::Hash,
    V: FromPyObject<'py>,
    S: hash::BuildHasher + Default,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> Result<Self, PyErr> {
        let dict = ob.cast::<PyDict>()?;
        let mut ret = hashbrown::HashMap::with_capacity_and_hasher(dict.len(), S::default());
        for (k, v) in dict {
            ret.insert(k.extract()?, v.extract()?);
        }
        Ok(ret)
    }
}

impl<'py, K, H> IntoPyObject<'py> for hashbrown::HashSet<K, H>
where
    K: IntoPyObject<'py> + cmp::Eq + hash::Hash,
    H: hash::BuildHasher,
{
    type Target = PySet;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        try_new_from_iter(py, self)
    }
}

impl<'a, 'py, K, H> IntoPyObject<'py> for &'a hashbrown::HashSet<K, H>
where
    &'a K: IntoPyObject<'py> + cmp::Eq + hash::Hash,
    H: hash::BuildHasher,
{
    type Target = PySet;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        try_new_from_iter(py, self)
    }
}

impl<'py, K, S> FromPyObject<'py> for hashbrown::HashSet<K, S>
where
    K: FromPyObject<'py> + cmp::Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        match ob.cast::<PySet>() {
            Ok(set) => set.iter().map(|any| any.extract()).collect(),
            Err(err) => {
                if let Ok(frozen_set) = ob.cast::<PyFrozenSet>() {
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
    use crate::types::IntoPyDict;
    use std::collections::hash_map::RandomState;

    #[test]
    fn test_hashbrown_hashmap_into_pyobject() {
        Python::attach(|py| {
            let mut map =
                hashbrown::HashMap::<i32, i32, RandomState>::with_hasher(RandomState::new());
            map.insert(1, 1);

            let py_map = (&map).into_pyobject(py).unwrap();

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
    fn test_hashbrown_hashmap_into_dict() {
        Python::attach(|py| {
            let mut map =
                hashbrown::HashMap::<i32, i32, RandomState>::with_hasher(RandomState::new());
            map.insert(1, 1);

            let py_map = map.into_py_dict(py).unwrap();

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
        Python::attach(|py| {
            let set = PySet::new(py, [1, 2, 3, 4, 5]).unwrap();
            let hash_set: hashbrown::HashSet<usize, RandomState> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());

            let set = PyFrozenSet::new(py, [1, 2, 3, 4, 5]).unwrap();
            let hash_set: hashbrown::HashSet<usize, RandomState> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());
        });
    }

    #[test]
    fn test_hashbrown_hashset_into_pyobject() {
        Python::attach(|py| {
            let hs: hashbrown::HashSet<u64, RandomState> =
                [1, 2, 3, 4, 5].iter().cloned().collect();

            let hso = hs.clone().into_pyobject(py).unwrap();

            assert_eq!(hs, hso.extract().unwrap());
        });
    }
}
