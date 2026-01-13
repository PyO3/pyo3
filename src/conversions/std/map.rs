#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::{type_hint_subscript, PyStaticExpr};
#[cfg(feature = "experimental-inspect")]
use crate::type_object::PyTypeInfo;
use crate::{
    conversion::{FromPyObjectOwned, IntoPyObject},
    instance::Bound,
    types::{any::PyAnyMethods, dict::PyDictMethods, PyDict},
    Borrowed, FromPyObject, PyAny, PyErr, Python,
};
use std::{cmp, collections, hash};

impl<'py, K, V, H> IntoPyObject<'py> for collections::HashMap<K, V, H>
where
    K: IntoPyObject<'py> + cmp::Eq + hash::Hash,
    V: IntoPyObject<'py>,
    H: hash::BuildHasher,
{
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyDict::TYPE_HINT, K::OUTPUT_TYPE, V::OUTPUT_TYPE);

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
    H: hash::BuildHasher,
{
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyDict::TYPE_HINT, <&K>::OUTPUT_TYPE, <&V>::OUTPUT_TYPE);

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyDict::TYPE_HINT, K::OUTPUT_TYPE, V::OUTPUT_TYPE);

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyDict::TYPE_HINT, <&K>::OUTPUT_TYPE, <&V>::OUTPUT_TYPE);

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

impl<'py, K, V, S> FromPyObject<'_, 'py> for collections::HashMap<K, V, S>
where
    K: FromPyObjectOwned<'py> + cmp::Eq + hash::Hash,
    V: FromPyObjectOwned<'py>,
    S: hash::BuildHasher + Default,
{
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(&PyDict::TYPE_HINT, K::INPUT_TYPE, V::INPUT_TYPE);

    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        let dict = ob.cast::<PyDict>()?;
        let mut ret = collections::HashMap::with_capacity_and_hasher(dict.len(), S::default());
        for (k, v) in dict.iter() {
            ret.insert(
                k.extract().map_err(Into::into)?,
                v.extract().map_err(Into::into)?,
            );
        }
        Ok(ret)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::mapping_of(K::type_input(), V::type_input())
    }
}

impl<'py, K, V> FromPyObject<'_, 'py> for collections::BTreeMap<K, V>
where
    K: FromPyObjectOwned<'py> + cmp::Ord,
    V: FromPyObjectOwned<'py>,
{
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyDict::TYPE_HINT, K::INPUT_TYPE, V::INPUT_TYPE);

    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> Result<Self, PyErr> {
        let dict = ob.cast::<PyDict>()?;
        let mut ret = collections::BTreeMap::new();
        for (k, v) in dict.iter() {
            ret.insert(
                k.extract().map_err(Into::into)?,
                v.extract().map_err(Into::into)?,
            );
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

            assert_eq!(py_map.len(), 1);
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

            assert_eq!(py_map.len(), 1);
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

            assert_eq!(py_map.len(), 1);
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

            assert_eq!(py_map.len(), 1);
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
