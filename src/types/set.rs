// Copyright (c) 2017-present PyO3 Project and Contributors
//

use crate::err::{self, PyErr, PyResult};
#[cfg(Py_LIMITED_API)]
use crate::types::PyIterator;
use crate::{ffi, AsPyPointer, FromPyObject, IntoPy, PyAny, PyObject, Python, ToPyObject};
use std::cmp;
use std::collections::{BTreeSet, HashSet};
use std::{collections, hash, ptr};

/// Represents a Python `set`
#[repr(transparent)]
pub struct PySet(PyAny);

pyobject_native_type!(PySet, ffi::PySetObject, ffi::PySet_Type, #checkfunction=ffi::PySet_Check);

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
        unsafe {
            match ffi::PySet_Contains(self.as_ptr(), key.to_object(self.py()).as_ptr()) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.py())),
            }
        }
    }

    /// Removes the element from the set if it is present.
    pub fn discard<K>(&self, key: K)
    where
        K: ToPyObject,
    {
        unsafe {
            ffi::PySet_Discard(self.as_ptr(), key.to_object(self.py()).as_ptr());
        }
    }

    /// Adds an element to the set.
    pub fn add<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PySet_Add(self.as_ptr(), key.to_object(self.py()).as_ptr()),
            )
        }
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
    /// # Panics
    ///
    /// If PyO3 detects that the set is mutated during iteration, it will panic.
    pub fn iter(&self) -> PySetIterator<'_> {
        IntoIterator::into_iter(self)
    }
}

#[cfg(Py_LIMITED_API)]
mod impl_ {
    use super::*;

    impl<'a> std::iter::IntoIterator for &'a PySet {
        type Item = &'a PyAny;
        type IntoIter = PySetIterator<'a>;

        /// Returns an iterator of values in this set.
        ///
        /// # Panics
        ///
        /// If PyO3 detects that the set is mutated during iteration, it will panic.
        fn into_iter(self) -> Self::IntoIter {
            PySetIterator {
                it: PyIterator::from_object(self.py(), self).unwrap(),
            }
        }
    }

    pub struct PySetIterator<'p> {
        it: &'p PyIterator,
    }

    impl<'py> Iterator for PySetIterator<'py> {
        type Item = &'py super::PyAny;

        /// Advances the iterator and returns the next value.
        ///
        /// # Panics
        ///
        /// If PyO3 detects that the set is mutated during iteration, it will panic.
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.it.next().map(Result::unwrap)
        }
    }
}

#[cfg(not(Py_LIMITED_API))]
mod impl_ {
    use super::*;
    pub struct PySetIterator<'py> {
        set: &'py super::PyAny,
        pos: ffi::Py_ssize_t,
        used: ffi::Py_ssize_t,
    }

    impl<'a> std::iter::IntoIterator for &'a PySet {
        type Item = &'a PyAny;
        type IntoIter = PySetIterator<'a>;
        /// Returns an iterator of values in this set.
        ///
        /// # Panics
        ///
        /// If PyO3 detects that the set is mutated during iteration, it will panic.
        fn into_iter(self) -> Self::IntoIter {
            PySetIterator {
                set: self,
                pos: 0,
                used: unsafe { ffi::PySet_Size(self.as_ptr()) },
            }
        }
    }

    impl<'py> Iterator for PySetIterator<'py> {
        type Item = &'py super::PyAny;

        /// Advances the iterator and returns the next value.
        ///
        /// # Panics
        ///
        /// If PyO3 detects that the set is mutated during iteration, it will panic.
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            unsafe {
                let len = ffi::PySet_Size(self.set.as_ptr());
                assert_eq!(self.used, len, "Set changed size during iteration");

                let mut key: *mut ffi::PyObject = std::ptr::null_mut();
                let mut hash: ffi::Py_hash_t = 0;
                if ffi::_PySet_NextEntry(self.set.as_ptr(), &mut self.pos, &mut key, &mut hash) != 0
                {
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
}

pub use impl_::*;

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

impl<'source, K> FromPyObject<'source> for BTreeSet<K>
where
    K: FromPyObject<'source> + cmp::Ord,
{
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let set: &PySet = ob.downcast()?;
        set.iter().map(K::extract).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::PySet;
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
    #[should_panic]
    fn test_set_iter_mutation() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1, 2, 3, 4, 5]).unwrap();

            for _ in set {
                let _ = set.add(42);
            }
        });
    }

    #[test]
    #[should_panic]
    fn test_set_iter_mutation_same_len() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1, 2, 3, 4, 5]).unwrap();

            for item in set {
                let item: i32 = item.extract().unwrap();
                let _ = set.del_item(item);
                let _ = set.add(item + 10);
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
