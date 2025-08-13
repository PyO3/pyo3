use std::{cmp, collections, hash};

#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    conversion::IntoPyObject,
    instance::Bound,
    types::{any::PyAnyMethods, dict::PyDictMethods, PyDict},
    FromPyObject, PyAny, PyErr, Python,
};

impl<'py, K, V, H> IntoPyObject<'py> for collections::HashMap<K, V, H>
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

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::dict_of(K::type_output(), V::type_output())
    }
}

impl<'a, 'py, K, V, H> IntoPyObject<'py> for &'a collections::HashMap<K, V, H>
where
    &'a K: IntoPyObject<'py> + cmp::Eq + hash::Hash,
    &'a V: IntoPyObject<'py>,
    K: 'a, // MSRV
    V: 'a, // MSRV
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

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::dict_of(<&K>::type_output(), <&V>::type_output())
    }
}

impl<'py, K, V> IntoPyObject<'py> for collections::BTreeMap<K, V>
where
    K: IntoPyObject<'py> + cmp::Eq,
    V: IntoPyObject<'py>,
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

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::dict_of(K::type_output(), V::type_output())
    }
}

impl<'a, 'py, K, V> IntoPyObject<'py> for &'a collections::BTreeMap<K, V>
where
    &'a K: IntoPyObject<'py> + cmp::Eq,
    &'a V: IntoPyObject<'py>,
    K: 'a,
    V: 'a,
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

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::dict_of(<&K>::type_output(), <&V>::type_output())
    }
}

impl<'py, K, V, S> FromPyObject<'py> for collections::HashMap<K, V, S>
where
    K: FromPyObject<'py> + cmp::Eq + hash::Hash,
    V: FromPyObject<'py>,
    S: hash::BuildHasher + Default,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> Result<Self, PyErr> {
        let dict = ob.cast::<PyDict>()?;
        let mut ret = collections::HashMap::with_capacity_and_hasher(dict.len(), S::default());
        for (k, v) in dict {
            ret.insert(k.extract()?, v.extract()?);
        }
        Ok(ret)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::mapping_of(K::type_input(), V::type_input())
    }
}

impl<'py, K, V> FromPyObject<'py> for collections::BTreeMap<K, V>
where
    K: FromPyObject<'py> + cmp::Ord,
    V: FromPyObject<'py>,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> Result<Self, PyErr> {
        let dict = ob.cast::<PyDict>()?;
        let mut ret = collections::BTreeMap::new();
        for (k, v) in dict {
            ret.insert(k.extract()?, v.extract()?);
        }
        Ok(ret)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::mapping_of(K::type_input(), V::type_input())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn test_hashmap_to_python() {
        Python::attach(|py| {
            let mut map = HashMap::<i32, i32>::new();
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
    fn test_btreemap_to_python() {
        Python::attach(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
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
    fn test_hashmap_into_python() {
        Python::attach(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_pyobject(py).unwrap();

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
    fn test_btreemap_into_py() {
        Python::attach(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_pyobject(py).unwrap();

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
}
