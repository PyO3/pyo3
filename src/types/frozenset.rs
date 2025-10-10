use crate::types::PyIterator;
use crate::{
    err::{self, PyErr, PyResult},
    ffi,
    ffi_ptr_ext::FfiPtrExt,
    py_result_ext::PyResultExt,
    Bound, PyAny, Python,
};
use crate::{Borrowed, BoundObject, IntoPyObject, IntoPyObjectExt};
use std::ptr;

/// Allows building a Python `frozenset` one item at a time
pub struct PyFrozenSetBuilder<'py> {
    py_frozen_set: Bound<'py, PyFrozenSet>,
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
        K: IntoPyObject<'py>,
    {
        fn inner(frozenset: &Bound<'_, PyFrozenSet>, key: Borrowed<'_, '_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(frozenset.py(), unsafe {
                ffi::PySet_Add(frozenset.as_ptr(), key.as_ptr())
            })
        }

        inner(
            &self.py_frozen_set,
            key.into_pyobject(self.py_frozen_set.py())
                .map_err(Into::into)?
                .into_any()
                .as_borrowed(),
        )
    }

    /// Finish building the set and take ownership of its current value
    pub fn finalize(self) -> Bound<'py, PyFrozenSet> {
        self.py_frozen_set
    }
}

/// Represents a  Python `frozenset`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFrozenSet>`][crate::Py] or [`Bound<'py, PyFrozenSet>`][Bound].
///
/// For APIs available on `frozenset` objects, see the [`PyFrozenSetMethods`] trait which is implemented for
/// [`Bound<'py, PyFrozenSet>`][Bound].
#[repr(transparent)]
pub struct PyFrozenSet(PyAny);

#[cfg(not(any(PyPy, GraalPy)))]
pyobject_subclassable_native_type!(PyFrozenSet, crate::ffi::PySetObject);
#[cfg(not(any(PyPy, GraalPy)))]
pyobject_native_type!(
    PyFrozenSet,
    ffi::PySetObject,
    pyobject_native_static_type_object!(ffi::PyFrozenSet_Type),
    #checkfunction=ffi::PyFrozenSet_Check
);

#[cfg(any(PyPy, GraalPy))]
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
    pub fn new<'py, T>(
        py: Python<'py>,
        elements: impl IntoIterator<Item = T>,
    ) -> PyResult<Bound<'py, PyFrozenSet>>
    where
        T: IntoPyObject<'py>,
    {
        try_new_from_iter(py, elements)
    }

    /// Creates a new empty frozen set
    pub fn empty(py: Python<'_>) -> PyResult<Bound<'_, PyFrozenSet>> {
        unsafe {
            ffi::PyFrozenSet_New(ptr::null_mut())
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }
}

/// Implementation of functionality for [`PyFrozenSet`].
///
/// These methods are defined for the `Bound<'py, PyFrozenSet>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyFrozenSet")]
pub trait PyFrozenSetMethods<'py>: crate::sealed::Sealed {
    /// Returns the number of items in the set.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    fn len(&self) -> usize;

    /// Checks if set is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Determines if the set contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: IntoPyObject<'py>;

    /// Returns an iterator of values in this set.
    fn iter(&self) -> BoundFrozenSetIterator<'py>;
}

impl<'py> PyFrozenSetMethods<'py> for Bound<'py, PyFrozenSet> {
    #[inline]
    fn len(&self) -> usize {
        unsafe { ffi::PySet_Size(self.as_ptr()) as usize }
    }

    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: IntoPyObject<'py>,
    {
        fn inner(
            frozenset: &Bound<'_, PyFrozenSet>,
            key: Borrowed<'_, '_, PyAny>,
        ) -> PyResult<bool> {
            match unsafe { ffi::PySet_Contains(frozenset.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(frozenset.py())),
            }
        }

        let py = self.py();
        inner(
            self,
            key.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn iter(&self) -> BoundFrozenSetIterator<'py> {
        BoundFrozenSetIterator::new(self.clone())
    }
}

impl<'py> IntoIterator for Bound<'py, PyFrozenSet> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundFrozenSetIterator<'py>;

    /// Returns an iterator of values in this set.
    fn into_iter(self) -> Self::IntoIter {
        BoundFrozenSetIterator::new(self)
    }
}

impl<'py> IntoIterator for &Bound<'py, PyFrozenSet> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundFrozenSetIterator<'py>;

    /// Returns an iterator of values in this set.
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// PyO3 implementation of an iterator for a Python `frozenset` object.
pub struct BoundFrozenSetIterator<'p> {
    it: Bound<'p, PyIterator>,
    // Remaining elements in the frozenset
    remaining: usize,
}

impl<'py> BoundFrozenSetIterator<'py> {
    pub(super) fn new(set: Bound<'py, PyFrozenSet>) -> Self {
        Self {
            it: PyIterator::from_object(&set).unwrap(),
            remaining: set.len(),
        }
    }
}

impl<'py> Iterator for BoundFrozenSetIterator<'py> {
    type Item = Bound<'py, super::PyAny>;

    /// Advances the iterator and returns the next value.
    fn next(&mut self) -> Option<Self::Item> {
        self.remaining = self.remaining.saturating_sub(1);
        self.it.next().map(Result::unwrap)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
}

impl ExactSizeIterator for BoundFrozenSetIterator<'_> {
    fn len(&self) -> usize {
        self.remaining
    }
}

#[inline]
pub(crate) fn try_new_from_iter<'py, T>(
    py: Python<'py>,
    elements: impl IntoIterator<Item = T>,
) -> PyResult<Bound<'py, PyFrozenSet>>
where
    T: IntoPyObject<'py>,
{
    let set = unsafe {
        // We create the  `Py` pointer because its Drop cleans up the set if user code panics.
        ffi::PyFrozenSet_New(std::ptr::null_mut())
            .assume_owned_or_err(py)?
            .cast_into_unchecked()
    };
    let ptr = set.as_ptr();

    for e in elements {
        let obj = e.into_pyobject_or_pyerr(py)?;
        err::error_on_minusone(py, unsafe { ffi::PySet_Add(ptr, obj.as_ptr()) })?;
    }

    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyAnyMethods as _;

    #[test]
    fn test_frozenset_new_and_len() {
        Python::attach(|py| {
            let set = PyFrozenSet::new(py, [1]).unwrap();
            assert_eq!(1, set.len());

            let v = vec![1];
            assert!(PyFrozenSet::new(py, &[v]).is_err());
        });
    }

    #[test]
    fn test_frozenset_empty() {
        Python::attach(|py| {
            let set = PyFrozenSet::empty(py).unwrap();
            assert_eq!(0, set.len());
            assert!(set.is_empty());
        });
    }

    #[test]
    fn test_frozenset_contains() {
        Python::attach(|py| {
            let set = PyFrozenSet::new(py, [1]).unwrap();
            assert!(set.contains(1).unwrap());
        });
    }

    #[test]
    fn test_frozenset_iter() {
        Python::attach(|py| {
            let set = PyFrozenSet::new(py, [1]).unwrap();

            for el in set {
                assert_eq!(1i32, el.extract::<i32>().unwrap());
            }
        });
    }

    #[test]
    fn test_frozenset_iter_bound() {
        Python::attach(|py| {
            let set = PyFrozenSet::new(py, [1]).unwrap();

            for el in &set {
                assert_eq!(1i32, el.extract::<i32>().unwrap());
            }
        });
    }

    #[test]
    fn test_frozenset_iter_size_hint() {
        Python::attach(|py| {
            let set = PyFrozenSet::new(py, [1]).unwrap();
            let mut iter = set.iter();

            // Exact size
            assert_eq!(iter.len(), 1);
            assert_eq!(iter.size_hint(), (1, Some(1)));
            iter.next();
            assert_eq!(iter.len(), 0);
            assert_eq!(iter.size_hint(), (0, Some(0)));
        });
    }

    #[test]
    fn test_frozenset_builder() {
        use super::PyFrozenSetBuilder;

        Python::attach(|py| {
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

    #[test]
    fn test_iter_count() {
        Python::attach(|py| {
            let set = PyFrozenSet::new(py, vec![1, 2, 3]).unwrap();
            assert_eq!(set.iter().count(), 3);
        })
    }
}
