use std::{cmp, collections, hash};

#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::{type_hint_subscript, PyStaticExpr};
#[cfg(feature = "experimental-inspect")]
use crate::type_object::PyTypeInfo;
use crate::{
    conversion::{FromPyObjectOwned, IntoPyObject},
    types::{
        any::PyAnyMethods,
        frozenset::PyFrozenSetMethods,
        set::{try_new_from_iter, PySetMethods},
        PyFrozenSet, PySet,
    },
    Borrowed, Bound, FromPyObject, PyAny, PyErr, Python,
};

impl<'py, K, S> IntoPyObject<'py> for collections::HashSet<K, S>
where
    K: IntoPyObject<'py> + Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    type Target = PySet;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = type_hint_subscript!(PySet::TYPE_HINT, K::OUTPUT_TYPE);

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
    H: hash::BuildHasher,
{
    type Target = PySet;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;
    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = type_hint_subscript!(PySet::TYPE_HINT, <&K>::OUTPUT_TYPE);

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        try_new_from_iter(py, self.iter())
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::set_of(<&K>::type_output())
    }
}

impl<'py, K, S> FromPyObject<'_, 'py> for collections::HashSet<K, S>
where
    K: FromPyObjectOwned<'py> + cmp::Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = type_hint_subscript!(PySet::TYPE_HINT, K::INPUT_TYPE);

    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        match ob.cast::<PySet>() {
            Ok(set) => set
                .iter()
                .map(|any| any.extract().map_err(Into::into))
                .collect(),
            Err(err) => {
                if let Ok(frozen_set) = ob.cast::<PyFrozenSet>() {
                    frozen_set
                        .iter()
                        .map(|any| any.extract().map_err(Into::into))
                        .collect()
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

impl<'py, K> IntoPyObject<'py> for collections::BTreeSet<K>
where
    K: IntoPyObject<'py> + cmp::Ord,
{
    type Target = PySet;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = type_hint_subscript!(PySet::TYPE_HINT, K::OUTPUT_TYPE);

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = type_hint_subscript!(PySet::TYPE_HINT, <&K>::OUTPUT_TYPE);

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        try_new_from_iter(py, self.iter())
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::set_of(<&K>::type_output())
    }
}

impl<'py, K> FromPyObject<'_, 'py> for collections::BTreeSet<K>
where
    K: FromPyObjectOwned<'py> + cmp::Ord,
{
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = type_hint_subscript!(PySet::TYPE_HINT, K::INPUT_TYPE);

    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        match ob.cast::<PySet>() {
            Ok(set) => set
                .iter()
                .map(|any| any.extract().map_err(Into::into))
                .collect(),
            Err(err) => {
                if let Ok(frozen_set) = ob.cast::<PyFrozenSet>() {
                    frozen_set
                        .iter()
                        .map(|any| any.extract().map_err(Into::into))
                        .collect()
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
    use crate::{IntoPyObject, Python};
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn test_extract_hashset() {
        Python::attach(|py| {
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
        Python::attach(|py| {
            let set = PySet::new(py, [1, 2, 3, 4, 5]).unwrap();
            let hash_set: BTreeSet<usize> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());

            let set = PyFrozenSet::new(py, [1, 2, 3, 4, 5]).unwrap();
            let hash_set: BTreeSet<usize> = set.extract().unwrap();
            assert_eq!(hash_set, [1, 2, 3, 4, 5].iter().copied().collect());
        });
    }

    #[test]
    fn test_set_into_pyobject() {
        Python::attach(|py| {
            let bt: BTreeSet<u64> = [1, 2, 3, 4, 5].iter().cloned().collect();
            let hs: HashSet<u64> = [1, 2, 3, 4, 5].iter().cloned().collect();

            let bto = (&bt).into_pyobject(py).unwrap();
            let hso = (&hs).into_pyobject(py).unwrap();

            assert_eq!(bt, bto.extract().unwrap());
            assert_eq!(hs, hso.extract().unwrap());
        });
    }
}
