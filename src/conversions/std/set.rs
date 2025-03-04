use std::{cmp, collections, hash};

#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    conversion::IntoPyObject,
    instance::Bound,
    types::{
        any::PyAnyMethods,
        frozenset::PyFrozenSetMethods,
        set::{new_from_iter, try_new_from_iter, PySetMethods},
        PyFrozenSet, PySet,
    },
    FromPyObject, PyAny, PyErr, PyObject, PyResult, Python,
};
#[allow(deprecated)]
use crate::{IntoPy, ToPyObject};

#[allow(deprecated)]
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

#[allow(deprecated)]
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

#[allow(deprecated)]
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
}

impl<'py, K, S> IntoPyObject<'py> for collections::HashSet<K, S>
where
    K: IntoPyObject<'py> + Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    type Target = PySet;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        try_new_from_iter(py, self)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::set_of(K::type_output())
    }
}

impl<'a, 'py, K, H> IntoPyObject<'py> for &'a collections::HashSet<K, H>
where
    &'a K: IntoPyObject<'py> + Eq + hash::Hash,
    K: 'a, // MSRV
    H: hash::BuildHasher,
{
    type Target = PySet;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        try_new_from_iter(py, self.iter())
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::set_of(<&K>::type_output())
    }
}

impl<'py, K, S> FromPyObject<'py> for collections::HashSet<K, S>
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

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::set_of(K::type_input())
    }
}

#[allow(deprecated)]
impl<K> IntoPy<PyObject> for collections::BTreeSet<K>
where
    K: IntoPy<PyObject> + cmp::Ord,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        new_from_iter(py, self.into_iter().map(|item| item.into_py(py)))
            .expect("Failed to create Python set from BTreeSet")
            .into()
    }
}

impl<'py, K> IntoPyObject<'py> for collections::BTreeSet<K>
where
    K: IntoPyObject<'py> + cmp::Ord,
{
    type Target = PySet;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        try_new_from_iter(py, self)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::set_of(K::type_output())
    }
}

impl<'a, 'py, K> IntoPyObject<'py> for &'a collections::BTreeSet<K>
where
    &'a K: IntoPyObject<'py> + cmp::Ord,
    K: 'a,
{
    type Target = PySet;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        try_new_from_iter(py, self.iter())
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::set_of(<&K>::type_output())
    }
}

impl<'py, K> FromPyObject<'py> for collections::BTreeSet<K>
where
    K: FromPyObject<'py> + cmp::Ord,
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

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::set_of(K::type_input())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{any::PyAnyMethods, PyFrozenSet, PySet};
    use crate::{IntoPyObject, PyObject, Python};
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn test_extract_hashset() {
        Python::with_gil(|py| {
            let set = PySet::new(py, [1, 2, 3, 4, 5]).unwrap();
            let hash_set: HashSet<usize> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());

            let set = PyFrozenSet::new(py, [1, 2, 3, 4, 5]).unwrap();
            let hash_set: HashSet<usize> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());
        });
    }

    #[test]
    fn test_extract_btreeset() {
        Python::with_gil(|py| {
            let set = PySet::new(py, [1, 2, 3, 4, 5]).unwrap();
            let hash_set: BTreeSet<usize> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());

            let set = PyFrozenSet::new(py, [1, 2, 3, 4, 5]).unwrap();
            let hash_set: BTreeSet<usize> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());
        });
    }

    #[test]
    #[allow(deprecated)]
    fn test_set_into_py() {
        use crate::IntoPy;
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
    fn test_set_into_pyobject() {
        Python::with_gil(|py| {
            let bt: BTreeSet<u64> = [1, 2, 3, 4, 5].iter().cloned().collect();
            let hs: HashSet<u64> = [1, 2, 3, 4, 5].iter().cloned().collect();

            let bto = (&bt).into_pyobject(py).unwrap();
            let hso = (&hs).into_pyobject(py).unwrap();

            assert_eq!(bt, bto.extract().unwrap());
            assert_eq!(hs, hso.extract().unwrap());
        });
    }
}
