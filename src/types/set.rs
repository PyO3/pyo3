#[cfg(Py_LIMITED_API)]
use crate::types::PyIterator;
use crate::{
    err::{self, PyErr, PyResult},
    ffi_ptr_ext::FfiPtrExt,
    instance::Bound,
    Py, PyNativeType,
};
use crate::{ffi, PyAny, PyObject, Python, ToPyObject};
use std::ptr;

/// Represents a Python `set`
#[repr(transparent)]
pub struct PySet(PyAny);

#[cfg(not(PyPy))]
pyobject_native_type!(
    PySet,
    ffi::PySetObject,
    pyobject_native_static_type_object!(ffi::PySet_Type),
    #checkfunction=ffi::PySet_Check
);

#[cfg(PyPy)]
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
    pub fn new<'a, 'p, T: ToPyObject + 'a>(
        py: Python<'p>,
        elements: impl IntoIterator<Item = &'a T>,
    ) -> PyResult<&'p PySet> {
        new_from_iter(py, elements).map(|set| set.into_ref(py))
    }

    /// Creates a new empty set.
    pub fn empty(py: Python<'_>) -> PyResult<&PySet> {
        unsafe { py.from_owned_ptr_or_err(ffi::PySet_New(ptr::null_mut())) }
    }

    /// Removes all elements from the set.
    #[inline]
    pub fn clear(&self) {
        self.as_borrowed().clear()
    }

    /// Returns the number of items in the set.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    #[inline]
    pub fn len(&self) -> usize {
        self.as_borrowed().len()
    }

    /// Checks if set is empty.
    pub fn is_empty(&self) -> bool {
        self.as_borrowed().is_empty()
    }

    /// Determines if the set contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        self.as_borrowed().contains(key)
    }

    /// Removes the element from the set if it is present.
    ///
    /// Returns `true` if the element was present in the set.
    pub fn discard<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        self.as_borrowed().discard(key)
    }

    /// Adds an element to the set.
    pub fn add<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        self.as_borrowed().add(key)
    }

    /// Removes and returns an arbitrary element from the set.
    pub fn pop(&self) -> Option<PyObject> {
        self.as_borrowed().pop().map(Py::from)
    }

    /// Returns an iterator of values in this set.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the set is mutated during iteration, it will panic.
    pub fn iter(&self) -> PySetIterator<'_> {
        PySetIterator(BoundSetIterator::new(self.as_borrowed().to_owned()))
    }
}

/// Implementation of functionality for [`PySet`].
///
/// These methods are defined for the `Bound<'py, PySet>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PySet")]
pub trait PySetMethods<'py> {
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
        K: ToPyObject;

    /// Removes the element from the set if it is present.
    ///
    /// Returns `true` if the element was present in the set.
    fn discard<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject;

    /// Adds an element to the set.
    fn add<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject;

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
        K: ToPyObject,
    {
        fn inner(set: &Bound<'_, PySet>, key: Bound<'_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PySet_Contains(set.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(set.py())),
            }
        }

        let py = self.py();
        inner(self, key.to_object(py).into_bound(py))
    }

    fn discard<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        fn inner(set: &Bound<'_, PySet>, key: Bound<'_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PySet_Discard(set.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(set.py())),
            }
        }

        let py = self.py();
        inner(self, key.to_object(py).into_bound(py))
    }

    fn add<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        fn inner(set: &Bound<'_, PySet>, key: Bound<'_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(set.py(), unsafe {
                ffi::PySet_Add(set.as_ptr(), key.as_ptr())
            })
        }

        let py = self.py();
        inner(self, key.to_object(py).into_bound(py))
    }

    fn pop(&self) -> Option<Bound<'py, PyAny>> {
        let element = unsafe { ffi::PySet_Pop(self.as_ptr()).assume_owned_or_err(self.py()) };
        match element {
            Ok(e) => Some(e),
            Err(_) => None,
        }
    }

    fn iter(&self) -> BoundSetIterator<'py> {
        BoundSetIterator::new(self.clone())
    }
}

/// PyO3 implementation of an iterator for a Python `set` object.
pub struct PySetIterator<'py>(BoundSetIterator<'py>);

impl<'py> Iterator for PySetIterator<'py> {
    type Item = &'py super::PyAny;

    /// Advances the iterator and returns the next value.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the set is mutated during iteration, it will panic.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Bound::into_gil_ref)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'py> IntoIterator for &'py PySet {
    type Item = &'py PyAny;
    type IntoIter = PySetIterator<'py>;
    /// Returns an iterator of values in this set.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the set is mutated during iteration, it will panic.
    fn into_iter(self) -> Self::IntoIter {
        PySetIterator(BoundSetIterator::new(self.as_borrowed().to_owned()))
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

impl<'py> IntoIterator for &'_ Bound<'py, PySet> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundSetIterator<'py>;

    /// Returns an iterator of values in this set.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the set is mutated during iteration, it will panic.
    fn into_iter(self) -> Self::IntoIter {
        BoundSetIterator::new(self.clone())
    }
}

#[cfg(Py_LIMITED_API)]
mod impl_ {
    use super::*;

    /// PyO3 implementation of an iterator for a Python `set` object.
    pub struct BoundSetIterator<'p> {
        it: Bound<'p, PyIterator>,
    }

    impl<'py> BoundSetIterator<'py> {
        pub(super) fn new(set: Bound<'py, PySet>) -> Self {
            Self {
                it: PyIterator::from_object2(&set).unwrap(),
            }
        }
    }

    impl<'py> Iterator for BoundSetIterator<'py> {
        type Item = Bound<'py, super::PyAny>;

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

    /// PyO3 implementation of an iterator for a Python `set` object.
    pub struct BoundSetIterator<'py> {
        set: Bound<'py, super::PySet>,
        pos: ffi::Py_ssize_t,
        used: ffi::Py_ssize_t,
    }

    impl<'py> BoundSetIterator<'py> {
        pub(super) fn new(set: Bound<'py, PySet>) -> Self {
            let used = unsafe { ffi::PySet_Size(set.as_ptr()) };
            BoundSetIterator { set, pos: 0, used }
        }
    }

    impl<'py> Iterator for BoundSetIterator<'py> {
        type Item = Bound<'py, super::PyAny>;

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
                    Some(key.assume_borrowed(self.set.py()).to_owned())
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

    impl<'py> ExactSizeIterator for BoundSetIterator<'py> {
        fn len(&self) -> usize {
            self.set.len().saturating_sub(self.pos as usize)
        }
    }

    impl<'py> ExactSizeIterator for PySetIterator<'py> {
        fn len(&self) -> usize {
            self.0.len()
        }
    }
}

pub use impl_::*;

#[inline]
pub(crate) fn new_from_iter<T: ToPyObject>(
    py: Python<'_>,
    elements: impl IntoIterator<Item = T>,
) -> PyResult<Py<PySet>> {
    fn inner(py: Python<'_>, elements: &mut dyn Iterator<Item = PyObject>) -> PyResult<Py<PySet>> {
        let set: Py<PySet> = unsafe {
            // We create the  `Py` pointer because its Drop cleans up the set if user code panics.
            Py::from_owned_ptr_or_err(py, ffi::PySet_New(std::ptr::null_mut()))?
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
    use super::PySet;
    use crate::{Python, ToPyObject};
    use std::collections::HashSet;

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
            assert!(set.is_empty());
        });
    }

    #[test]
    fn test_set_len() {
        Python::with_gil(|py| {
            let mut v = HashSet::new();
            let ob = v.to_object(py);
            let set: &PySet = ob.downcast(py).unwrap();
            assert_eq!(0, set.len());
            v.insert(7);
            let ob = v.to_object(py);
            let set2: &PySet = ob.downcast(py).unwrap();
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
            for el in set {
                assert_eq!(1i32, el.extract::<'_, i32>().unwrap());
            }

            // intoiterator iteration
            for el in set {
                assert_eq!(1i32, el.extract::<'_, i32>().unwrap());
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
    #[cfg(not(Py_LIMITED_API))]
    fn test_set_iter_size_hint() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1]).unwrap();
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
    #[cfg(Py_LIMITED_API)]
    fn test_set_iter_size_hint() {
        Python::with_gil(|py| {
            let set = PySet::new(py, &[1]).unwrap();
            let iter = set.iter();

            // No known bounds
            assert_eq!(iter.size_hint(), (0, None));
        });
    }
}
