// Copyright (c) 2017-present PyO3 Project and Contributors
//

use crate::err::{self, PyErr, PyResult};
#[cfg(Py_LIMITED_API)]
use crate::types::PyIterator;
use crate::{
    ffi, AsPyPointer, FromPyObject, IntoPy, IntoPyObject, Py, PyAny, PyObject, Python,
    ToBorrowedObject, ToPyObject,
};
use std::cmp;
use std::collections::{BTreeSet, HashSet};
use std::{collections, hash, ptr};

/// Represents a Python `set`
#[repr(transparent)]
pub struct PySet(PyAny);

/// Represents a  Python `frozenset`
#[repr(transparent)]
pub struct PyFrozenSet(PyAny);

pyobject_native_type!(PySet, ffi::PySetObject, ffi::PySet_Type, #checkfunction=ffi::PySet_Check);
pyobject_native_type!(
    PyFrozenSet,
    ffi::PySetObject,
    ffi::PyFrozenSet_Type,
    #checkfunction=ffi::PyFrozenSet_Check
);

impl PySet {
    /// Creates a new set with elements from the given slice.
    ///
    /// Returns an error if some element is not hashable.
    pub fn new<'p, T: ToPyObject>(py: Python<'p>, elements: &[T]) -> PyResult<&'p PySet> {
        let list = elements.to_object(py);
        unsafe { py.from_owned_ptr_or_err(ffi::PySet_New(list.as_ptr())) }
    }

    /// Creates a new empty set.
    pub fn empty(py: Python<'_>) -> PyResult<&PySet> {
        unsafe { py.from_owned_ptr_or_err(ffi::PySet_New(ptr::null_mut())) }
    }

    /// Removes all elements from the set.
    #[inline]
    pub fn clear(&self) {
        unsafe {
            ffi::PySet_Clear(self.as_ptr());
        }
    }

    /// Returns the number of items in the set.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ffi::PySet_Size(self.as_ptr()) as usize }
    }

    /// Checks if set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Determines if the set contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            match ffi::PySet_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.py())),
            }
        })
    }

    /// Removes the element from the set if it is present.
    pub fn discard<K>(&self, key: K)
    where
        K: ToPyObject,
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            ffi::PySet_Discard(self.as_ptr(), key);
        })
    }

    /// Adds an element to the set.
    pub fn add<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        key.with_borrowed_ptr(self.py(), move |key| unsafe {
            err::error_on_minusone(self.py(), ffi::PySet_Add(self.as_ptr(), key))
        })
    }

    /// Removes and returns an arbitrary element from the set.
    pub fn pop(&self) -> Option<PyObject> {
        let element =
            unsafe { PyObject::from_owned_ptr_or_err(self.py(), ffi::PySet_Pop(self.as_ptr())) };
        match element {
            Ok(e) => Some(e),
            Err(_) => None,
        }
    }

    /// Returns an iterator of values in this set.
    ///
    /// Note that it can be unsafe to use when the set might be changed by other code.
    pub fn iter(&self) -> PySetIterator<'_> {
        PySetIterator::new(self)
    }
}

#[cfg(Py_LIMITED_API)]
pub struct PySetIterator<'p> {
    it: &'p PyIterator,
}

#[cfg(Py_LIMITED_API)]
impl PySetIterator<'_> {
    fn new(set: &PyAny) -> PySetIterator<'_> {
        PySetIterator {
            it: PyIterator::from_object(set.py(), set).unwrap(),
        }
    }
}

#[cfg(Py_LIMITED_API)]
impl<'py> Iterator for PySetIterator<'py> {
    type Item = &'py super::PyAny;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|p| p.unwrap())
    }
}

#[cfg(not(Py_LIMITED_API))]
pub struct PySetIterator<'py> {
    set: &'py super::PyAny,
    pos: isize,
}

#[cfg(not(Py_LIMITED_API))]
impl PySetIterator<'_> {
    fn new(set: &PyAny) -> PySetIterator<'_> {
        PySetIterator { set, pos: 0 }
    }
}

#[cfg(not(Py_LIMITED_API))]
impl<'py> Iterator for PySetIterator<'py> {
    type Item = &'py super::PyAny;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut key: *mut ffi::PyObject = std::ptr::null_mut();
            let mut hash: ffi::Py_hash_t = 0;
            if ffi::_PySet_NextEntry(self.set.as_ptr(), &mut self.pos, &mut key, &mut hash) != 0 {
                // _PySet_NextEntry returns borrowed object; for safety must make owned (see #890)
                Some(self.set.py().from_owned_ptr(ffi::_Py_NewRef(key)))
            } else {
                None
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.set.len().unwrap_or_default();
        (
            len.saturating_sub(self.pos as usize),
            Some(len.saturating_sub(self.pos as usize)),
        )
    }
}

impl<'a> std::iter::IntoIterator for &'a PySet {
    type Item = &'a PyAny;
    type IntoIter = PySetIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S> ToPyObject for collections::HashSet<T, S>
where
    T: hash::Hash + Eq + ToPyObject,
    S: hash::BuildHasher + Default,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let set = PySet::new::<T>(py, &[]).expect("Failed to construct empty set");
        {
            for val in self {
                set.add(val).expect("Failed to add to set");
            }
        }
        set.into()
    }
}

impl<T> ToPyObject for collections::BTreeSet<T>
where
    T: hash::Hash + Eq + ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let set = PySet::new::<T>(py, &[]).expect("Failed to construct empty set");
        {
            for val in self {
                set.add(val).expect("Failed to add to set");
            }
        }
        set.into()
    }
}

impl<K, S> IntoPy<PyObject> for HashSet<K, S>
where
    K: IntoPy<PyObject> + Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        let set = PySet::empty(py).expect("Failed to construct empty set");
        {
            for val in self {
                set.add(val.into_py(py)).expect("Failed to add to set");
            }
        }
        set.into()
    }
}

impl<K, S> IntoPyObject for HashSet<K, S>
where
    K: IntoPyObject + Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    type Target = PySet;
    fn into_py(self, py: Python<'_>) -> Py<PySet> {
        let set = PySet::empty(py).expect("Failed to construct empty set");
        {
            for val in self {
                set.add(val.into_py(py)).expect("Failed to add to set");
            }
        }
        set.into()
    }
}

impl<'source, K, S> FromPyObject<'source> for HashSet<K, S>
where
    K: FromPyObject<'source> + cmp::Eq + hash::Hash,
    S: hash::BuildHasher + Default,
{
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let set: &PySet = ob.downcast()?;
        set.iter().map(K::extract).collect()
    }
}

impl<K> IntoPy<PyObject> for BTreeSet<K>
where
    K: IntoPy<PyObject> + cmp::Ord,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        let set = PySet::empty(py).expect("Failed to construct empty set");
        {
            for val in self {
                set.add(val.into_py(py)).expect("Failed to add to set");
            }
        }
        set.into()
    }
}

impl<K> IntoPyObject for BTreeSet<K>
where
    K: IntoPyObject + cmp::Ord,
{
    type Target = PySet;
    fn into_py(self, py: Python<'_>) -> Py<PySet> {
        let set = PySet::empty(py).expect("Failed to construct empty set");
        {
            for val in self {
                set.add(val.into_py(py)).expect("Failed to add to set");
            }
        }
        set.into()
    }
}

impl<'source, K> FromPyObject<'source> for BTreeSet<K>
where
    K: FromPyObject<'source> + cmp::Ord,
{
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let set: &PySet = ob.downcast()?;
        set.iter().map(K::extract).collect()
    }
}

impl PyFrozenSet {
    /// Creates a new frozenset.
    ///
    /// May panic when running out of memory.
    pub fn new<'p, T: ToPyObject>(py: Python<'p>, elements: &[T]) -> PyResult<&'p PyFrozenSet> {
        let list = elements.to_object(py);
        unsafe { py.from_owned_ptr_or_err(ffi::PyFrozenSet_New(list.as_ptr())) }
    }

    /// Creates a new empty frozen set
    pub fn empty(py: Python<'_>) -> PyResult<&PyFrozenSet> {
        unsafe { py.from_owned_ptr_or_err(ffi::PyFrozenSet_New(ptr::null_mut())) }
    }

    /// Return the number of items in the set.
    /// This is equivalent to len(p) on a set.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ffi::PySet_Size(self.as_ptr()) as usize }
    }

    /// Check if set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Determine if the set contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToBorrowedObject,
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            match ffi::PySet_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.py())),
            }
        })
    }

    /// Returns an iterator of values in this frozen set.
    ///
    /// Note that it can be unsafe to use when the set might be changed by other code.
    pub fn iter(&self) -> PySetIterator<'_> {
        PySetIterator::new(self.as_ref())
    }
}

impl<'a> std::iter::IntoIterator for &'a PyFrozenSet {
    type Item = &'a PyAny;
    type IntoIter = PySetIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{PyFrozenSet, PySet};
    use crate::{IntoPy, PyObject, PyTryFrom, Python, ToPyObject};
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn test_set_new() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1]).unwrap();
            assert_eq!(1, set.len());

            let v = vec![1];
            assert!(PySet::new(py, &[v]).is_err());
        });
    }

    #[test]
    fn test_set_empty() {
        Python::with_gil(|py| {
            let set = PySet::empty(py).unwrap();
            assert_eq!(0, set.len());
        });
    }

    #[test]
    fn test_set_len() {
        Python::with_gil(|py| {
            let mut v = HashSet::new();
            let ob = v.to_object(py);
            let set = <PySet as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(0, set.len());
            v.insert(7);
            let ob = v.to_object(py);
            let set2 = <PySet as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(1, set2.len());
        });
    }

    #[test]
    fn test_set_clear() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1]).unwrap();
            assert_eq!(1, set.len());
            set.clear();
            assert_eq!(0, set.len());
        });
    }

    #[test]
    fn test_set_contains() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1]).unwrap();
            assert!(set.contains(1).unwrap());
        });
    }

    #[test]
    fn test_set_discard() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1]).unwrap();
            set.discard(2);
            assert_eq!(1, set.len());
            set.discard(1);
            assert_eq!(0, set.len());
        });
    }

    #[test]
    fn test_set_add() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1, 2]).unwrap();
            set.add(1).unwrap(); // Add a dupliated element
            assert!(set.contains(1).unwrap());
        });
    }

    #[test]
    fn test_set_pop() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1]).unwrap();
            let val = set.pop();
            assert!(val.is_some());
            let val2 = set.pop();
            assert!(val2.is_none());
            assert!(py
                .eval("print('Exception state should not be set.')", None, None)
                .is_ok());
        });
    }

    #[test]
    fn test_set_iter() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1]).unwrap();

            // iter method
            for el in set.iter() {
                assert_eq!(1i32, el.extract().unwrap());
            }

            // intoiterator iteration
            for el in set {
                assert_eq!(1i32, el.extract().unwrap());
            }
        });
    }

    #[test]
    fn test_set_iter_size_hint() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1]).unwrap();

            let mut iter = set.iter();

            if cfg!(Py_LIMITED_API) {
                assert_eq!(iter.size_hint(), (0, None));
            } else {
                assert_eq!(iter.size_hint(), (1, Some(1)));
                iter.next();
                assert_eq!(iter.size_hint(), (0, Some(0)));
            }
        });
    }

    #[test]
    fn test_frozenset_new_and_len() {
        Python::with_gil(|py| {
            let set = PyFrozenSet::new(py, &[1]).unwrap();
            assert_eq!(1, set.len());

            let v = vec![1];
            assert!(PyFrozenSet::new(py, &[v]).is_err());
        });
    }

    #[test]
    fn test_frozenset_empty() {
        Python::with_gil(|py| {
            let set = PyFrozenSet::empty(py).unwrap();
            assert_eq!(0, set.len());
        });
    }

    #[test]
    fn test_frozenset_contains() {
        Python::with_gil(|py| {
            let set = PyFrozenSet::new(py, &[1]).unwrap();
            assert!(set.contains(1).unwrap());
        });
    }

    #[test]
    fn test_frozenset_iter() {
        Python::with_gil(|py| {
            let set = PyFrozenSet::new(py, &[1]).unwrap();

            // iter method
            for el in set.iter() {
                assert_eq!(1i32, el.extract::<i32>().unwrap());
            }

            // intoiterator iteration
            for el in set {
                assert_eq!(1i32, el.extract::<i32>().unwrap());
            }
        });
    }

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
}
