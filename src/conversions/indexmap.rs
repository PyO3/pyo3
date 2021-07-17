#[cfg(feature = "indexmap")]
#[cfg_attr(docsrs, doc(cfg(feature = "indexmap")))]
mod indexmap_indexmap_conversion {

    use crate::types::*;
    use crate::{FromPyObject, IntoPy, PyErr, PyObject, PyTryFrom, Python, ToPyObject};
    use std::{cmp, hash};

    impl<K, V, H> ToPyObject for indexmap::IndexMap<K, V, H>
    where
        K: hash::Hash + cmp::Eq + ToPyObject,
        V: ToPyObject,
        H: hash::BuildHasher,
    {
        fn to_object(&self, py: Python) -> PyObject {
            IntoPyDict::into_py_dict(self, py).into()
        }
    }

    impl<K, V, H> IntoPy<PyObject> for indexmap::IndexMap<K, V, H>
    where
        K: hash::Hash + cmp::Eq + IntoPy<PyObject>,
        V: IntoPy<PyObject>,
        H: hash::BuildHasher,
    {
        fn into_py(self, py: Python) -> PyObject {
            let iter = self
                .into_iter()
                .map(|(k, v)| (k.into_py(py), v.into_py(py)));
            IntoPyDict::into_py_dict(iter, py).into()
        }
    }

    impl<'source, K, V, S> FromPyObject<'source> for indexmap::IndexMap<K, V, S>
    where
        K: FromPyObject<'source> + cmp::Eq + hash::Hash,
        V: FromPyObject<'source>,
        S: hash::BuildHasher + Default,
    {
        fn extract(ob: &'source PyAny) -> Result<Self, PyErr> {
            let dict = <PyDict as PyTryFrom>::try_from(ob)?;
            let mut ret = indexmap::IndexMap::with_capacity_and_hasher(dict.len(), S::default());
            for (k, v) in dict.iter() {
                ret.insert(K::extract(k)?, V::extract(v)?);
            }
            Ok(ret)
        }
    }
}

#[cfg(feature = "indexmap")]
#[cfg(test)]
mod test_indexmap {

    use crate::types::*;
    use crate::{IntoPy, PyObject, PyTryFrom, Python, ToPyObject};

    #[test]
    fn test_indexmap_indexmap_to_python() {
        Python::with_gil(|py| {
            let mut map = indexmap::IndexMap::<i32, i32>::new();
            map.insert(1, 1);

            let m = map.to_object(py);
            let py_map = <PyDict as PyTryFrom>::try_from(m.as_ref(py)).unwrap();

            assert!(py_map.len() == 1);
            assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
            assert_eq!(
                map,
                py_map.extract::<indexmap::IndexMap::<i32, i32>>().unwrap()
            );
        });
    }

    #[test]
    fn test_indexmap_indexmap_into_python() {
        Python::with_gil(|py| {
            let mut map = indexmap::IndexMap::<i32, i32>::new();
            map.insert(1, 1);

            let m: PyObject = map.into_py(py);
            let py_map = <PyDict as PyTryFrom>::try_from(m.as_ref(py)).unwrap();

            assert!(py_map.len() == 1);
            assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
        });
    }

    #[test]
    fn test_indexmap_indexmap_into_dict() {
        Python::with_gil(|py| {
            let mut map = indexmap::IndexMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict(py);

            assert_eq!(py_map.len(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }
}
