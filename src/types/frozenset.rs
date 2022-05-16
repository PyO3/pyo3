// Copyright (c) 2017-present PyO3 Project and Contributors
//

use crate::err::{PyErr, PyResult};
#[cfg(Py_LIMITED_API)]
use crate::types::PyIterator;
use crate::{ffi, AsPyPointer, PyAny, Python, ToPyObject};

use std::ptr;

/// Represents a  Python `frozenset`
#[repr(transparent)]
pub struct PyFrozenSet(PyAny);

pyobject_native_type!(
    PyFrozenSet,
    ffi::PySetObject,
    ffi::PyFrozenSet_Type,
    #checkfunction=ffi::PyFrozenSet_Check
);

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

    /// Returns an iterator of values in this frozen set.
    pub fn iter(&self) -> PyFrozenSetIterator<'_> {
        IntoIterator::into_iter(self)
    }
}

#[cfg(Py_LIMITED_API)]
mod impl_ {
    use super::*;

    impl<'a> std::iter::IntoIterator for &'a PyFrozenSet {
        type Item = &'a PyAny;
        type IntoIter = PyFrozenSetIterator<'a>;

        fn into_iter(self) -> Self::IntoIter {
            PyFrozenSetIterator {
                it: PyIterator::from_object(self.py(), self).unwrap(),
            }
        }
    }

    pub struct PyFrozenSetIterator<'p> {
        it: &'p PyIterator,
    }

    impl<'py> Iterator for PyFrozenSetIterator<'py> {
        type Item = &'py super::PyAny;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.it.next().map(Result::unwrap)
        }
    }
}

#[cfg(not(Py_LIMITED_API))]
mod impl_ {
    use super::*;

    impl<'a> std::iter::IntoIterator for &'a PyFrozenSet {
        type Item = &'a PyAny;
        type IntoIter = PyFrozenSetIterator<'a>;

        fn into_iter(self) -> Self::IntoIter {
            PyFrozenSetIterator { set: self, pos: 0 }
        }
    }

    pub struct PyFrozenSetIterator<'py> {
        set: &'py PyAny,
        pos: ffi::Py_ssize_t,
    }

    impl<'py> Iterator for PyFrozenSetIterator<'py> {
        type Item = &'py PyAny;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            unsafe {
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

use impl_::*;

#[cfg(test)]
mod tests {
    use super::*;

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
}
