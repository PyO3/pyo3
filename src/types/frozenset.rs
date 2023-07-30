#[cfg(Py_LIMITED_API)]
use crate::types::PyIterator;
use crate::{
    err::{self, PyErr, PyResult},
    Py, PyObject,
};
use crate::{ffi, PyAny, Python, ToPyObject};

use std::ptr;

/// Allows building a Python `frozenset` one item at a time
pub struct PyFrozenSetBuilder<'py> {
    py_frozen_set: &'py PyFrozenSet,
}

impl<'py> PyFrozenSetBuilder<'py> {
    /// Create a new `FrozenSetBuilder`.
    /// Since this allocates a `PyFrozenSet` internally it may
    /// panic when running out of memory.
    pub fn new(py: Python<'py>) -> PyResult<PyFrozenSetBuilder<'py>> {
        Ok(PyFrozenSetBuilder {
            py_frozen_set: PyFrozenSet::empty(py)?,
        })
    }

    /// Adds an element to the set.
    pub fn add<K>(&mut self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        fn inner(frozenset: &PyFrozenSet, key: PyObject) -> PyResult<()> {
            err::error_on_minusone(frozenset.py(), unsafe {
                ffi::PySet_Add(frozenset.as_ptr(), key.as_ptr())
            })
        }

        inner(self.py_frozen_set, key.to_object(self.py_frozen_set.py()))
    }

    /// Finish building the set and take ownership of its current value
    pub fn finalize(self) -> &'py PyFrozenSet {
        self.py_frozen_set
    }
}

/// Represents a  Python `frozenset`
#[repr(transparent)]
pub struct PyFrozenSet(PyAny);

#[cfg(not(PyPy))]
pyobject_native_type!(
    PyFrozenSet,
    ffi::PySetObject,
    pyobject_native_static_type_object!(ffi::PyFrozenSet_Type),
    #checkfunction=ffi::PyFrozenSet_Check
);

#[cfg(PyPy)]
pyobject_native_type_core!(
    PyFrozenSet,
    pyobject_native_static_type_object!(ffi::PyFrozenSet_Type),
    #checkfunction=ffi::PyFrozenSet_Check
);

impl PyFrozenSet {
    /// Creates a new frozenset.
    ///
    /// May panic when running out of memory.
    #[inline]
    pub fn new<'a, 'p, T: ToPyObject + 'a>(
        py: Python<'p>,
        elements: impl IntoIterator<Item = &'a T>,
    ) -> PyResult<&'p PyFrozenSet> {
        new_from_iter(py, elements).map(|set| set.into_ref(py))
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
        fn inner(frozenset: &PyFrozenSet, key: PyObject) -> PyResult<bool> {
            match unsafe { ffi::PySet_Contains(frozenset.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(frozenset.py())),
            }
        }

        inner(self, key.to_object(self.py()))
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

    /// PyO3 implementation of an iterator for a Python `frozenset` object.
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

    /// PyO3 implementation of an iterator for a Python `frozenset` object.
    pub struct PyFrozenSetIterator<'py> {
        set: &'py PyFrozenSet,
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
            let len = self.len();
            (len, Some(len))
        }
    }

    impl<'py> ExactSizeIterator for PyFrozenSetIterator<'py> {
        fn len(&self) -> usize {
            self.set.len().saturating_sub(self.pos as usize)
        }
    }
}

pub use impl_::*;

#[inline]
pub(crate) fn new_from_iter<T: ToPyObject>(
    py: Python<'_>,
    elements: impl IntoIterator<Item = T>,
) -> PyResult<Py<PyFrozenSet>> {
    fn inner(
        py: Python<'_>,
        elements: &mut dyn Iterator<Item = PyObject>,
    ) -> PyResult<Py<PyFrozenSet>> {
        let set: Py<PyFrozenSet> = unsafe {
            // We create the  `Py` pointer because its Drop cleans up the set if user code panics.
            Py::from_owned_ptr_or_err(py, ffi::PyFrozenSet_New(std::ptr::null_mut()))?
        };
        let ptr = set.as_ptr();

        for obj in elements {
            err::error_on_minusone(py, unsafe { ffi::PySet_Add(ptr, obj.as_ptr()) })?;
        }

        Ok(set)
    }

    let mut iter = elements.into_iter().map(|e| e.to_object(py));
    inner(py, &mut iter)
}

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
            for el in set {
                assert_eq!(1i32, el.extract::<i32>().unwrap());
            }

            // intoiterator iteration
            for el in set {
                assert_eq!(1i32, el.extract::<i32>().unwrap());
            }
        });
    }

    #[test]
    fn test_frozenset_builder() {
        use super::PyFrozenSetBuilder;

        Python::with_gil(|py| {
            let mut builder = PyFrozenSetBuilder::new(py).unwrap();

            // add an item
            builder.add(1).unwrap();
            builder.add(2).unwrap();
            builder.add(2).unwrap();

            // finalize it
            let set = builder.finalize();

            assert!(set.contains(1).unwrap());
            assert!(set.contains(2).unwrap());
            assert!(!set.contains(3).unwrap());
        });
    }
}
