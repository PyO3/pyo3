#![cfg(feature = "indexmap")]

//!  Conversions to and from [indexmap](https://docs.rs/indexmap/)’s
//! `IndexMap`.
//!
//! [`indexmap::IndexMap`] is a hash table that is closely compatible with the standard [`std::collections::HashMap`],
//! with the difference that it preserves the insertion order when iterating over keys. It was inspired
//! by Python's 3.6+ dict implementation.
//!
//! Dictionary order is guaranteed to be insertion order in Python, hence IndexMap is a good candidate
//! for maintaining an equivalent behaviour in Rust.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! # change * to the latest versions
//! indexmap = "*"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"indexmap\"] }")]
//! ```
//!
//! Note that you must use compatible versions of indexmap and PyO3.
//! The required indexmap version may vary based on the version of PyO3.
//!
//! # Examples
//!
//! Using [indexmap](https://docs.rs/indexmap) to return a dictionary with some statistics
//! about a list of numbers. Because of the insertion order guarantees, the Python code will
//! always print the same result, matching users' expectations about Python's dict.
//! ```rust
//! use indexmap::{indexmap, IndexMap};
//! use pyo3::prelude::*;
//!
//! fn median(data: &Vec<i32>) -> f32 {
//!     let sorted_data = data.clone().sort();
//!     let mid = data.len() / 2;
//!     if data.len() % 2 == 0 {
//!         data[mid] as f32
//!     }
//!     else {
//!         (data[mid] + data[mid - 1]) as f32 / 2.0
//!     }
//! }
//!
//! fn mean(data: &Vec<i32>) -> f32 {
//!     data.iter().sum::<i32>() as f32 / data.len() as f32
//! }
//! fn mode(data: &Vec<i32>) -> f32 {
//!     let mut frequency = IndexMap::new(); // we can use IndexMap as any hash table
//!
//!     for &element in data {
//!         *frequency.entry(element).or_insert(0) += 1;
//!     }
//!
//!     frequency
//!         .iter()
//!         .max_by(|a, b| a.1.cmp(&b.1))
//!         .map(|(k, _v)| *k)
//!         .unwrap() as f32
//!   }
//!
//! #[pyfunction]
//! fn calculate_statistics(data: Vec<i32>) -> IndexMap<&'static str, f32> {
//!     indexmap! {
//!        "median" => median(&data),
//!        "mean" => mean(&data),
//!        "mode" => mode(&data),
//!     }
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(calculate_statistics, m)?)?;
//!     Ok(())
//! }
//! ```
//!
//! Python code:
//! ```python
//! from my_module import calculate_statistics
//!
//! data = [1, 1, 1, 3, 4, 5]
//! print(calculate_statistics(data))
//! # always prints {"median": 2.0, "mean": 2.5, "mode": 1.0} in the same order
//! # if another hash table was used, the order could be random
//! ```

use crate::conversion::IntoPyObject;
use crate::types::*;
use crate::{Bound, FromPyObject, PyErr, Python};
use std::{cmp, hash};

impl<'py, K, V, H> IntoPyObject<'py> for indexmap::IndexMap<K, V, H>
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

impl<'a, 'py, K, V, H> IntoPyObject<'py> for &'a indexmap::IndexMap<K, V, H>
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

impl<'py, K, V, S> FromPyObject<'py> for indexmap::IndexMap<K, V, S>
where
    K: FromPyObject<'py> + cmp::Eq + hash::Hash,
    V: FromPyObject<'py>,
    S: hash::BuildHasher + Default,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> Result<Self, PyErr> {
        let dict = ob.cast::<PyDict>()?;
        let mut ret = indexmap::IndexMap::with_capacity_and_hasher(dict.len(), S::default());
        for (k, v) in dict {
            ret.insert(k.extract()?, v.extract()?);
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod test_indexmap {

    use crate::types::*;
    use crate::{IntoPyObject, Python};

    #[test]
    fn test_indexmap_indexmap_into_pyobject() {
        Python::attach(|py| {
            let mut map = indexmap::IndexMap::<i32, i32>::new();
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
            assert_eq!(
                map,
                py_map.extract::<indexmap::IndexMap::<i32, i32>>().unwrap()
            );
        });
    }

    #[test]
    fn test_indexmap_indexmap_into_dict() {
        Python::attach(|py| {
            let mut map = indexmap::IndexMap::<i32, i32>::new();
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
    fn test_indexmap_indexmap_insertion_order_round_trip() {
        Python::attach(|py| {
            let n = 20;
            let mut map = indexmap::IndexMap::<i32, i32>::new();

            for i in 1..=n {
                if i % 2 == 1 {
                    map.insert(i, i);
                } else {
                    map.insert(n - i, i);
                }
            }

            let py_map = (&map).into_py_dict(py).unwrap();

            let trip_map = py_map.extract::<indexmap::IndexMap<i32, i32>>().unwrap();

            for (((k1, v1), (k2, v2)), (k3, v3)) in
                map.iter().zip(py_map.iter()).zip(trip_map.iter())
            {
                let k2 = k2.extract::<i32>().unwrap();
                let v2 = v2.extract::<i32>().unwrap();
                assert_eq!((k1, v1), (&k2, &v2));
                assert_eq!((k1, v1), (k3, v3));
                assert_eq!((&k2, &v2), (k3, v3));
            }
        });
    }
}
