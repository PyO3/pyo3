use std::{cmp, collections, hash};

#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    types::set::new_from_iter, types::PySet, FromPyObject, IntoPy, PyAny, PyObject, PyResult,
    Python, ToPyObject,
};

impl<T, S> ToPyObject for collections::HashSet<T, S>
where
    T: hash::Hash + Eq + ToPyObject,
    S: hash::BuildHasher + Default,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        new_from_iter(py, self)
            .expect("Failed to create Python set from HashSet")
            .into()
    }
}

impl<T> ToPyObject for collections::BTreeSet<T>
where
    T: hash::Hash + Eq + ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        new_from_iter(py, self)
            .expect("Failed to create Python set from BTreeSet")
            .into()
    }
}

impl<K, S> IntoPy<PyObject> for collections::HashSet<K, S>
where
    K: IntoPy<PyObject> + Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        new_from_iter(py, self.into_iter().map(|item| item.into_py(py)))
            .expect("Failed to create Python set from HashSet")
            .into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::set_of(K::type_output())
    }
}

impl<'source, K, S> FromPyObject<'source> for collections::HashSet<K, S>
where
    K: FromPyObject<'source> + cmp::Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let set: &PySet = ob.downcast()?;
        set.iter().map(K::extract).collect()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::set_of(K::type_input())
    }
}

impl<K> IntoPy<PyObject> for collections::BTreeSet<K>
where
    K: IntoPy<PyObject> + cmp::Ord,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        new_from_iter(py, self.into_iter().map(|item| item.into_py(py)))
            .expect("Failed to create Python set from BTreeSet")
            .into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::set_of(K::type_output())
    }
}

impl<'source, K> FromPyObject<'source> for collections::BTreeSet<K>
where
    K: FromPyObject<'source> + cmp::Ord,
{
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let set: &PySet = ob.downcast()?;
        set.iter().map(K::extract).collect()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::set_of(K::type_input())
    }
}

#[cfg(test)]
mod tests {
    use super::PySet;
    use crate::{IntoPy, PyObject, Python, ToPyObject};
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn test_extract_hashset() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1, 2, 3, 4, 5]).unwrap();
            let hash_set: HashSet<usize> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());
        });
    }

    #[test]
    fn test_extract_btreeset() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1, 2, 3, 4, 5]).unwrap();
            let hash_set: BTreeSet<usize> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());
        });
    }

    #[test]
    fn test_set_into_py() {
        Python::with_gil(|py| {
            let bt: BTreeSet<u64> = [1, 2, 3, 4, 5].iter().cloned().collect();
            let hs: HashSet<u64> = [1, 2, 3, 4, 5].iter().cloned().collect();

            let bto: PyObject = bt.clone().into_py(py);
            let hso: PyObject = hs.clone().into_py(py);

            assert_eq!(bt, bto.extract(py).unwrap());
            assert_eq!(hs, hso.extract(py).unwrap());
        });
    }

    #[test]
    fn test_set_to_object() {
        Python::with_gil(|py| {
            let bt: BTreeSet<u64> = [1, 2, 3, 4, 5].iter().cloned().collect();
            let hs: HashSet<u64> = [1, 2, 3, 4, 5].iter().cloned().collect();

            let bto: PyObject = bt.to_object(py);
            let hso: PyObject = hs.to_object(py);

            assert_eq!(bt, bto.extract(py).unwrap());
            assert_eq!(hs, hso.extract(py).unwrap());
        });
    }
}
