use crate::types::PyIterator;
use crate::{
    err::{self, PyErr, PyResult},
    ffi_ptr_ext::FfiPtrExt,
    instance::Bound,
    py_result_ext::PyResultExt,
};
use crate::{ffi, Borrowed, BoundObject, IntoPyObject, IntoPyObjectExt, PyAny, Python};
use std::ptr;

/// Represents a Python `set`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PySet>`][crate::Py] or [`Bound<'py, PySet>`][Bound].
///
/// For APIs available on `set` objects, see the [`PySetMethods`] trait which is implemented for
/// [`Bound<'py, PySet>`][Bound].
#[repr(transparent)]
pub struct PySet(PyAny);

#[cfg(not(any(PyPy, GraalPy)))]
pyobject_subclassable_native_type!(PySet, crate::ffi::PySetObject);

#[cfg(not(any(PyPy, GraalPy)))]
pyobject_native_type!(
    PySet,
    ffi::PySetObject,
    pyobject_native_static_type_object!(ffi::PySet_Type),
    #checkfunction=ffi::PySet_Check
);

#[cfg(any(PyPy, GraalPy))]
pyobject_native_type_core!(
    PySet,
    pyobject_native_static_type_object!(ffi::PySet_Type),
    #checkfunction=ffi::PySet_Check
);

impl PySet {
    /// Creates a new set with elements from the given slice.
    ///
    /// Returns an error if some element is not hashable.
    #[inline]
    pub fn new<'py, T>(
        py: Python<'py>,
        elements: impl IntoIterator<Item = T>,
    ) -> PyResult<Bound<'py, PySet>>
    where
        T: IntoPyObject<'py>,
    {
        try_new_from_iter(py, elements)
    }

    /// Creates a new empty set.
    pub fn empty(py: Python<'_>) -> PyResult<Bound<'_, PySet>> {
        unsafe {
            ffi::PySet_New(ptr::null_mut())
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }
}

/// Implementation of functionality for [`PySet`].
///
/// These methods are defined for the `Bound<'py, PySet>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PySet")]
pub trait PySetMethods<'py>: crate::sealed::Sealed {
    /// Removes all elements from the set.
    fn clear(&self);

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

    /// Removes the element from the set if it is present.
    ///
    /// Returns `true` if the element was present in the set.
    fn discard<K>(&self, key: K) -> PyResult<bool>
    where
        K: IntoPyObject<'py>;

    /// Adds an element to the set.
    fn add<K>(&self, key: K) -> PyResult<()>
    where
        K: IntoPyObject<'py>;

    /// Removes and returns an arbitrary element from the set.
    fn pop(&self) -> Option<Bound<'py, PyAny>>;

    /// Returns an iterator of values in this set.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the set is mutated during iteration, it will panic.
    fn iter(&self) -> BoundSetIterator<'py>;
}

impl<'py> PySetMethods<'py> for Bound<'py, PySet> {
    #[inline]
    fn clear(&self) {
        unsafe {
            ffi::PySet_Clear(self.as_ptr());
        }
    }

    #[inline]
    fn len(&self) -> usize {
        unsafe { ffi::PySet_Size(self.as_ptr()) as usize }
    }

    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: IntoPyObject<'py>,
    {
        fn inner(set: &Bound<'_, PySet>, key: Borrowed<'_, '_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PySet_Contains(set.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(set.py())),
            }
        }

        let py = self.py();
        inner(
            self,
            key.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn discard<K>(&self, key: K) -> PyResult<bool>
    where
        K: IntoPyObject<'py>,
    {
        fn inner(set: &Bound<'_, PySet>, key: Borrowed<'_, '_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PySet_Discard(set.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(set.py())),
            }
        }

        let py = self.py();
        inner(
            self,
            key.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn add<K>(&self, key: K) -> PyResult<()>
    where
        K: IntoPyObject<'py>,
    {
        fn inner(set: &Bound<'_, PySet>, key: Borrowed<'_, '_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(set.py(), unsafe {
                ffi::PySet_Add(set.as_ptr(), key.as_ptr())
            })
        }

        let py = self.py();
        inner(
            self,
            key.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn pop(&self) -> Option<Bound<'py, PyAny>> {
        let element = unsafe { ffi::PySet_Pop(self.as_ptr()).assume_owned_or_err(self.py()) };
        element.ok()
    }

    fn iter(&self) -> BoundSetIterator<'py> {
        BoundSetIterator::new(self.clone())
    }
}

impl<'py> IntoIterator for Bound<'py, PySet> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundSetIterator<'py>;

    /// Returns an iterator of values in this set.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the set is mutated during iteration, it will panic.
    fn into_iter(self) -> Self::IntoIter {
        BoundSetIterator::new(self)
    }
}

impl<'py> IntoIterator for &Bound<'py, PySet> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundSetIterator<'py>;

    /// Returns an iterator of values in this set.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the set is mutated during iteration, it will panic.
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// PyO3 implementation of an iterator for a Python `set` object.
pub struct BoundSetIterator<'p> {
    it: Bound<'p, PyIterator>,
    // Remaining elements in the set. This is fine to store because
    // Python will error if the set changes size during iteration.
    remaining: usize,
}

impl<'py> BoundSetIterator<'py> {
    pub(super) fn new(set: Bound<'py, PySet>) -> Self {
        Self {
            it: PyIterator::from_object(&set).unwrap(),
            remaining: set.len(),
        }
    }
}

impl<'py> Iterator for BoundSetIterator<'py> {
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

impl ExactSizeIterator for BoundSetIterator<'_> {
    fn len(&self) -> usize {
        self.remaining
    }
}

#[inline]
pub(crate) fn try_new_from_iter<'py, T>(
    py: Python<'py>,
    elements: impl IntoIterator<Item = T>,
) -> PyResult<Bound<'py, PySet>>
where
    T: IntoPyObject<'py>,
{
    let set = unsafe {
        // We create the `Bound` pointer because its Drop cleans up the set if
        // user code errors or panics.
        ffi::PySet_New(std::ptr::null_mut())
            .assume_owned_or_err(py)?
            .cast_into_unchecked()
    };
    let ptr = set.as_ptr();

    elements.into_iter().try_for_each(|element| {
        let obj = element.into_pyobject_or_pyerr(py)?;
        err::error_on_minusone(py, unsafe { ffi::PySet_Add(ptr, obj.as_ptr()) })
    })?;

    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::PySet;
    use crate::{
        conversion::IntoPyObject,
        ffi,
        types::{PyAnyMethods, PySetMethods},
        Python,
    };
    use std::collections::HashSet;

    #[test]
    fn test_set_new() {
        Python::attach(|py| {
            let set = PySet::new(py, [1]).unwrap();
            assert_eq!(1, set.len());

            let v = vec![1];
            assert!(PySet::new(py, &[v]).is_err());
        });
    }

    #[test]
    fn test_set_empty() {
        Python::attach(|py| {
            let set = PySet::empty(py).unwrap();
            assert_eq!(0, set.len());
            assert!(set.is_empty());
        });
    }

    #[test]
    fn test_set_len() {
        Python::attach(|py| {
            let mut v = HashSet::<i32>::new();
            let ob = (&v).into_pyobject(py).unwrap();
            let set = ob.cast::<PySet>().unwrap();
            assert_eq!(0, set.len());
            v.insert(7);
            let ob = v.into_pyobject(py).unwrap();
            let set2 = ob.cast::<PySet>().unwrap();
            assert_eq!(1, set2.len());
        });
    }

    #[test]
    fn test_set_clear() {
        Python::attach(|py| {
            let set = PySet::new(py, [1]).unwrap();
            assert_eq!(1, set.len());
            set.clear();
            assert_eq!(0, set.len());
        });
    }

    #[test]
    fn test_set_contains() {
        Python::attach(|py| {
            let set = PySet::new(py, [1]).unwrap();
            assert!(set.contains(1).unwrap());
        });
    }

    #[test]
    fn test_set_discard() {
        Python::attach(|py| {
            let set = PySet::new(py, [1]).unwrap();
            assert!(!set.discard(2).unwrap());
            assert_eq!(1, set.len());

            assert!(set.discard(1).unwrap());
            assert_eq!(0, set.len());
            assert!(!set.discard(1).unwrap());

            assert!(set.discard(vec![1, 2]).is_err());
        });
    }

    #[test]
    fn test_set_add() {
        Python::attach(|py| {
            let set = PySet::new(py, [1, 2]).unwrap();
            set.add(1).unwrap(); // Add a dupliated element
            assert!(set.contains(1).unwrap());
        });
    }

    #[test]
    fn test_set_pop() {
        Python::attach(|py| {
            let set = PySet::new(py, [1]).unwrap();
            let val = set.pop();
            assert!(val.is_some());
            let val2 = set.pop();
            assert!(val2.is_none());
            assert!(py
                .eval(
                    ffi::c_str!("print('Exception state should not be set.')"),
                    None,
                    None
                )
                .is_ok());
        });
    }

    #[test]
    fn test_set_iter() {
        Python::attach(|py| {
            let set = PySet::new(py, [1]).unwrap();

            for el in set {
                assert_eq!(1i32, el.extract::<'_, i32>().unwrap());
            }
        });
    }

    #[test]
    fn test_set_iter_bound() {
        use crate::types::any::PyAnyMethods;

        Python::attach(|py| {
            let set = PySet::new(py, [1]).unwrap();

            for el in &set {
                assert_eq!(1i32, el.extract::<i32>().unwrap());
            }
        });
    }

    #[test]
    #[should_panic]
    fn test_set_iter_mutation() {
        Python::attach(|py| {
            let set = PySet::new(py, [1, 2, 3, 4, 5]).unwrap();

            for _ in &set {
                let _ = set.add(42);
            }
        });
    }

    #[test]
    #[should_panic]
    fn test_set_iter_mutation_same_len() {
        Python::attach(|py| {
            let set = PySet::new(py, [1, 2, 3, 4, 5]).unwrap();

            for item in &set {
                let item: i32 = item.extract().unwrap();
                let _ = set.del_item(item);
                let _ = set.add(item + 10);
            }
        });
    }

    #[test]
    fn test_set_iter_size_hint() {
        Python::attach(|py| {
            let set = PySet::new(py, [1]).unwrap();
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
    fn test_iter_count() {
        Python::attach(|py| {
            let set = PySet::new(py, vec![1, 2, 3]).unwrap();
            assert_eq!(set.iter().count(), 3);
        })
    }
}
