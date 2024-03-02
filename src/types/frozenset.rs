use crate::types::PyIterator;
use crate::{
    err::{self, PyErr, PyResult},
    ffi,
    ffi_ptr_ext::FfiPtrExt,
    py_result_ext::PyResultExt,
    types::any::PyAnyMethods,
    Bound, PyAny, PyNativeType, PyObject, Python, ToPyObject,
};
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
            py_frozen_set: PyFrozenSet::empty_bound(py)?,
        })
    }

    /// Adds an element to the set.
    pub fn add<K>(&mut self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        fn inner(frozenset: &Bound<'_, PyFrozenSet>, key: PyObject) -> PyResult<()> {
            err::error_on_minusone(frozenset.py(), unsafe {
                ffi::PySet_Add(frozenset.as_ptr(), key.as_ptr())
            })
        }

        inner(&self.py_frozen_set, key.to_object(self.py_frozen_set.py()))
    }

    /// Deprecated form of [`PyFrozenSetBuilder::finalize_bound`]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyFrozenSetBuilder::finalize` will be replaced by `PyFrozenSetBuilder::finalize_bound` in a future PyO3 version"
        )
    )]
    pub fn finalize(self) -> &'py PyFrozenSet {
        self.finalize_bound().into_gil_ref()
    }

    /// Finish building the set and take ownership of its current value
    pub fn finalize_bound(self) -> Bound<'py, PyFrozenSet> {
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
    /// Deprecated form of [`PyFrozenSet::new_bound`].
    #[inline]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyFrozenSet::new` will be replaced by `PyFrozenSet::new_bound` in a future PyO3 version"
        )
    )]
    pub fn new<'a, 'p, T: ToPyObject + 'a>(
        py: Python<'p>,
        elements: impl IntoIterator<Item = &'a T>,
    ) -> PyResult<&'p PyFrozenSet> {
        Self::new_bound(py, elements).map(Bound::into_gil_ref)
    }

    /// Creates a new frozenset.
    ///
    /// May panic when running out of memory.
    #[inline]
    pub fn new_bound<'a, 'p, T: ToPyObject + 'a>(
        py: Python<'p>,
        elements: impl IntoIterator<Item = &'a T>,
    ) -> PyResult<Bound<'p, PyFrozenSet>> {
        new_from_iter(py, elements)
    }

    /// Deprecated form of [`PyFrozenSet::empty_bound`].
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyFrozenSet::empty` will be replaced by `PyFrozenSet::empty_bound` in a future PyO3 version"
        )
    )]
    pub fn empty(py: Python<'_>) -> PyResult<&'_ PyFrozenSet> {
        Self::empty_bound(py).map(Bound::into_gil_ref)
    }

    /// Creates a new empty frozen set
    pub fn empty_bound(py: Python<'_>) -> PyResult<Bound<'_, PyFrozenSet>> {
        unsafe {
            ffi::PyFrozenSet_New(ptr::null_mut())
                .assume_owned_or_err(py)
                .downcast_into_unchecked()
        }
    }

    /// Return the number of items in the set.
    /// This is equivalent to len(p) on a set.
    #[inline]
    pub fn len(&self) -> usize {
        self.as_borrowed().len()
    }

    /// Check if set is empty.
    pub fn is_empty(&self) -> bool {
        self.as_borrowed().is_empty()
    }

    /// Determine if the set contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        self.as_borrowed().contains(key)
    }

    /// Returns an iterator of values in this frozen set.
    pub fn iter(&self) -> PyFrozenSetIterator<'_> {
        PyFrozenSetIterator(BoundFrozenSetIterator::new(self.as_borrowed().to_owned()))
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
        K: ToPyObject;

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
        K: ToPyObject,
    {
        fn inner(frozenset: &Bound<'_, PyFrozenSet>, key: Bound<'_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PySet_Contains(frozenset.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(frozenset.py())),
            }
        }

        let py = self.py();
        inner(self, key.to_object(py).into_bound(py))
    }

    fn iter(&self) -> BoundFrozenSetIterator<'py> {
        BoundFrozenSetIterator::new(self.clone())
    }
}

/// PyO3 implementation of an iterator for a Python `frozenset` object.
pub struct PyFrozenSetIterator<'py>(BoundFrozenSetIterator<'py>);

impl<'py> Iterator for PyFrozenSetIterator<'py> {
    type Item = &'py super::PyAny;

    /// Advances the iterator and returns the next value.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Bound::into_gil_ref)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for PyFrozenSetIterator<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'py> IntoIterator for &'py PyFrozenSet {
    type Item = &'py PyAny;
    type IntoIter = PyFrozenSetIterator<'py>;
    /// Returns an iterator of values in this set.
    fn into_iter(self) -> Self::IntoIter {
        PyFrozenSetIterator(BoundFrozenSetIterator::new(self.as_borrowed().to_owned()))
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

/// PyO3 implementation of an iterator for a Python `frozenset` object.
pub struct BoundFrozenSetIterator<'p> {
    it: Bound<'p, PyIterator>,
    // Remaining elements in the frozenset
    remaining: usize,
}

impl<'py> BoundFrozenSetIterator<'py> {
    pub(super) fn new(set: Bound<'py, PyFrozenSet>) -> Self {
        Self {
            it: PyIterator::from_bound_object(&set).unwrap(),
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
}

impl<'py> ExactSizeIterator for BoundFrozenSetIterator<'py> {
    fn len(&self) -> usize {
        self.remaining
    }
}

#[inline]
pub(crate) fn new_from_iter<T: ToPyObject>(
    py: Python<'_>,
    elements: impl IntoIterator<Item = T>,
) -> PyResult<Bound<'_, PyFrozenSet>> {
    fn inner<'py>(
        py: Python<'py>,
        elements: &mut dyn Iterator<Item = PyObject>,
    ) -> PyResult<Bound<'py, PyFrozenSet>> {
        let set = unsafe {
            // We create the  `Py` pointer because its Drop cleans up the set if user code panics.
            ffi::PyFrozenSet_New(std::ptr::null_mut())
                .assume_owned_or_err(py)?
                .downcast_into_unchecked()
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
#[cfg_attr(not(feature = "gil-refs"), allow(deprecated))]
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
            assert!(set.is_empty());
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
    fn test_frozenset_iter_size_hint() {
        Python::with_gil(|py| {
            let set = PyFrozenSet::new(py, &[1]).unwrap();
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

        Python::with_gil(|py| {
            let mut builder = PyFrozenSetBuilder::new(py).unwrap();

            // add an item
            builder.add(1).unwrap();
            builder.add(2).unwrap();
            builder.add(2).unwrap();

            // finalize it
            let set = builder.finalize_bound();

            assert!(set.contains(1).unwrap());
            assert!(set.contains(2).unwrap());
            assert!(!set.contains(3).unwrap());
        });
    }
}
