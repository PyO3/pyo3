use super::PyMapping;
use crate::err::{self, PyErr, PyResult};
use crate::ffi::Py_ssize_t;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::{Borrowed, Bound};
use crate::py_result_ext::PyResultExt;
use crate::types::any::PyAnyMethods;
use crate::types::{PyAny, PyList};
#[cfg(feature = "gil-refs")]
use crate::PyNativeType;
use crate::{ffi, Python, ToPyObject};

/// Represents a Python `dict`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyDict>`][crate::Py] or [`Bound<'py, PyDict>`][Bound].
///
/// For APIs available on `dict` objects, see the [`PyDictMethods`] trait which is implemented for
/// [`Bound<'py, PyDict>`][Bound].
#[repr(transparent)]
pub struct PyDict(PyAny);

pyobject_native_type!(
    PyDict,
    ffi::PyDictObject,
    pyobject_native_static_type_object!(ffi::PyDict_Type),
    #checkfunction=ffi::PyDict_Check
);

/// Represents a Python `dict_keys`.
#[cfg(not(any(PyPy, GraalPy)))]
#[repr(transparent)]
pub struct PyDictKeys(PyAny);

#[cfg(not(any(PyPy, GraalPy)))]
pyobject_native_type_core!(
    PyDictKeys,
    pyobject_native_static_type_object!(ffi::PyDictKeys_Type),
    #checkfunction=ffi::PyDictKeys_Check
);

/// Represents a Python `dict_values`.
#[cfg(not(any(PyPy, GraalPy)))]
#[repr(transparent)]
pub struct PyDictValues(PyAny);

#[cfg(not(any(PyPy, GraalPy)))]
pyobject_native_type_core!(
    PyDictValues,
    pyobject_native_static_type_object!(ffi::PyDictValues_Type),
    #checkfunction=ffi::PyDictValues_Check
);

/// Represents a Python `dict_items`.
#[cfg(not(any(PyPy, GraalPy)))]
#[repr(transparent)]
pub struct PyDictItems(PyAny);

#[cfg(not(any(PyPy, GraalPy)))]
pyobject_native_type_core!(
    PyDictItems,
    pyobject_native_static_type_object!(ffi::PyDictItems_Type),
    #checkfunction=ffi::PyDictItems_Check
);

impl PyDict {
    /// Creates a new empty dictionary.
    pub fn new_bound(py: Python<'_>) -> Bound<'_, PyDict> {
        unsafe { ffi::PyDict_New().assume_owned(py).downcast_into_unchecked() }
    }

    /// Creates a new dictionary from the sequence given.
    ///
    /// The sequence must consist of `(PyObject, PyObject)`. This is
    /// equivalent to `dict([("a", 1), ("b", 2)])`.
    ///
    /// Returns an error on invalid input. In the case of key collisions,
    /// this keeps the last entry seen.
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn from_sequence_bound<'py>(seq: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
        let py = seq.py();
        let dict = Self::new_bound(py);
        err::error_on_minusone(py, unsafe {
            ffi::PyDict_MergeFromSeq2(dict.as_ptr(), seq.as_ptr(), 1)
        })?;
        Ok(dict)
    }
}

#[cfg(feature = "gil-refs")]
impl PyDict {
    /// Deprecated form of [`new_bound`][PyDict::new_bound].
    #[deprecated(
        since = "0.21.0",
        note = "`PyDict::new` will be replaced by `PyDict::new_bound` in a future PyO3 version"
    )]
    #[inline]
    pub fn new(py: Python<'_>) -> &PyDict {
        Self::new_bound(py).into_gil_ref()
    }

    /// Deprecated form of [`from_sequence_bound`][PyDict::from_sequence_bound].
    #[deprecated(
        since = "0.21.0",
        note = "`PyDict::from_sequence` will be replaced by `PyDict::from_sequence_bound` in a future PyO3 version"
    )]
    #[inline]
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn from_sequence(seq: &PyAny) -> PyResult<&PyDict> {
        Self::from_sequence_bound(&seq.as_borrowed()).map(Bound::into_gil_ref)
    }

    /// Returns a new dictionary that contains the same key-value pairs as self.
    ///
    /// This is equivalent to the Python expression `self.copy()`.
    pub fn copy(&self) -> PyResult<&PyDict> {
        self.as_borrowed().copy().map(Bound::into_gil_ref)
    }

    /// Empties an existing dictionary of all key-value pairs.
    pub fn clear(&self) {
        self.as_borrowed().clear()
    }

    /// Return the number of items in the dictionary.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    pub fn len(&self) -> usize {
        self.as_borrowed().len()
    }

    /// Checks if the dict is empty, i.e. `len(self) == 0`.
    pub fn is_empty(&self) -> bool {
        self.as_borrowed().is_empty()
    }

    /// Determines if the dictionary contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        self.as_borrowed().contains(key)
    }

    /// Gets an item from the dictionary.
    ///
    /// Returns `Ok(None)` if the item is not present. To get a `KeyError` for
    /// non-existing keys, use [`PyAny::get_item`].
    ///
    /// Returns `Err(PyErr)` if Python magic methods `__hash__` or `__eq__` used in dictionary
    /// lookup raise an exception, for example if the key `K` is not hashable. Usually it is
    /// best to bubble this error up to the caller using the `?` operator.
    ///
    /// # Examples
    ///
    /// The following example calls `get_item` for the dictionary `{"a": 1}` with various
    /// keys.
    /// - `get_item("a")` returns `Ok(Some(...))`, with the `PyAny` being a reference to the Python
    ///   int `1`.
    /// - `get_item("b")` returns `Ok(None)`, because "b" is not in the dictionary.
    /// - `get_item(dict)` returns an `Err(PyErr)`. The error will be a `TypeError` because a dict is not
    ///   hashable.
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{IntoPyDict};
    /// use pyo3::exceptions::{PyTypeError, PyKeyError};
    ///
    /// # fn main() {
    /// # let _ =
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let dict = &[("a", 1)].into_py_dict_bound(py);
    ///     // `a` is in the dictionary, with value 1
    ///     assert!(dict.get_item("a")?.map_or(Ok(false), |x| x.eq(1))?);
    ///     // `b` is not in the dictionary
    ///     assert!(dict.get_item("b")?.is_none());
    ///     // `dict` is not hashable, so this returns an error
    ///     assert!(dict.get_item(dict).unwrap_err().is_instance_of::<PyTypeError>(py));
    ///
    ///     // `PyAny::get_item("b")` will raise a `KeyError` instead of returning `None`
    ///     let any = dict.as_any();
    ///     assert!(any.get_item("b").unwrap_err().is_instance_of::<PyKeyError>(py));
    ///     Ok(())
    /// });
    /// # }
    /// ```
    pub fn get_item<K>(&self, key: K) -> PyResult<Option<&PyAny>>
    where
        K: ToPyObject,
    {
        match self.as_borrowed().get_item(key) {
            Ok(Some(item)) => Ok(Some(item.into_gil_ref())),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Sets an item value.
    ///
    /// This is equivalent to the Python statement `self[key] = value`.
    pub fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject,
    {
        self.as_borrowed().set_item(key, value)
    }

    /// Deletes an item.
    ///
    /// This is equivalent to the Python statement `del self[key]`.
    pub fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        self.as_borrowed().del_item(key)
    }

    /// Returns a list of dict keys.
    ///
    /// This is equivalent to the Python expression `list(dict.keys())`.
    pub fn keys(&self) -> &PyList {
        self.as_borrowed().keys().into_gil_ref()
    }

    /// Returns a list of dict values.
    ///
    /// This is equivalent to the Python expression `list(dict.values())`.
    pub fn values(&self) -> &PyList {
        self.as_borrowed().values().into_gil_ref()
    }

    /// Returns a list of dict items.
    ///
    /// This is equivalent to the Python expression `list(dict.items())`.
    pub fn items(&self) -> &PyList {
        self.as_borrowed().items().into_gil_ref()
    }

    /// Returns an iterator of `(key, value)` pairs in this dictionary.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the dictionary is mutated during iteration, it will panic.
    /// It is allowed to modify values as you iterate over the dictionary, but only
    /// so long as the set of keys does not change.
    pub fn iter(&self) -> PyDictIterator<'_> {
        PyDictIterator(self.as_borrowed().iter())
    }

    /// Returns `self` cast as a `PyMapping`.
    pub fn as_mapping(&self) -> &PyMapping {
        unsafe { self.downcast_unchecked() }
    }

    /// Update this dictionary with the key/value pairs from another.
    ///
    /// This is equivalent to the Python expression `self.update(other)`. If `other` is a `PyDict`, you may want
    /// to use `self.update(other.as_mapping())`, note: `PyDict::as_mapping` is a zero-cost conversion.
    pub fn update(&self, other: &PyMapping) -> PyResult<()> {
        self.as_borrowed().update(&other.as_borrowed())
    }

    /// Add key/value pairs from another dictionary to this one only when they do not exist in this.
    ///
    /// This is equivalent to the Python expression `self.update({k: v for k, v in other.items() if k not in self})`.
    /// If `other` is a `PyDict`, you may want to use `self.update_if_missing(other.as_mapping())`,
    /// note: `PyDict::as_mapping` is a zero-cost conversion.
    ///
    /// This method uses [`PyDict_Merge`](https://docs.python.org/3/c-api/dict.html#c.PyDict_Merge) internally,
    /// so should have the same performance as `update`.
    pub fn update_if_missing(&self, other: &PyMapping) -> PyResult<()> {
        self.as_borrowed().update_if_missing(&other.as_borrowed())
    }
}

/// Implementation of functionality for [`PyDict`].
///
/// These methods are defined for the `Bound<'py, PyDict>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyDict")]
pub trait PyDictMethods<'py>: crate::sealed::Sealed {
    /// Returns a new dictionary that contains the same key-value pairs as self.
    ///
    /// This is equivalent to the Python expression `self.copy()`.
    fn copy(&self) -> PyResult<Bound<'py, PyDict>>;

    /// Empties an existing dictionary of all key-value pairs.
    fn clear(&self);

    /// Return the number of items in the dictionary.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    fn len(&self) -> usize;

    /// Checks if the dict is empty, i.e. `len(self) == 0`.
    fn is_empty(&self) -> bool;

    /// Determines if the dictionary contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject;

    /// Gets an item from the dictionary.
    ///
    /// Returns `None` if the item is not present, or if an error occurs.
    ///
    /// To get a `KeyError` for non-existing keys, use `PyAny::get_item`.
    fn get_item<K>(&self, key: K) -> PyResult<Option<Bound<'py, PyAny>>>
    where
        K: ToPyObject;

    /// Sets an item value.
    ///
    /// This is equivalent to the Python statement `self[key] = value`.
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject;

    /// Deletes an item.
    ///
    /// This is equivalent to the Python statement `del self[key]`.
    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject;

    /// Returns a list of dict keys.
    ///
    /// This is equivalent to the Python expression `list(dict.keys())`.
    fn keys(&self) -> Bound<'py, PyList>;

    /// Returns a list of dict values.
    ///
    /// This is equivalent to the Python expression `list(dict.values())`.
    fn values(&self) -> Bound<'py, PyList>;

    /// Returns a list of dict items.
    ///
    /// This is equivalent to the Python expression `list(dict.items())`.
    fn items(&self) -> Bound<'py, PyList>;

    /// Returns an iterator of `(key, value)` pairs in this dictionary.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the dictionary is mutated during iteration, it will panic.
    /// It is allowed to modify values as you iterate over the dictionary, but only
    /// so long as the set of keys does not change.
    fn iter(&self) -> BoundDictIterator<'py>;

    /// Returns `self` cast as a `PyMapping`.
    fn as_mapping(&self) -> &Bound<'py, PyMapping>;

    /// Returns `self` cast as a `PyMapping`.
    fn into_mapping(self) -> Bound<'py, PyMapping>;

    /// Update this dictionary with the key/value pairs from another.
    ///
    /// This is equivalent to the Python expression `self.update(other)`. If `other` is a `PyDict`, you may want
    /// to use `self.update(other.as_mapping())`, note: `PyDict::as_mapping` is a zero-cost conversion.
    fn update(&self, other: &Bound<'_, PyMapping>) -> PyResult<()>;

    /// Add key/value pairs from another dictionary to this one only when they do not exist in this.
    ///
    /// This is equivalent to the Python expression `self.update({k: v for k, v in other.items() if k not in self})`.
    /// If `other` is a `PyDict`, you may want to use `self.update_if_missing(other.as_mapping())`,
    /// note: `PyDict::as_mapping` is a zero-cost conversion.
    ///
    /// This method uses [`PyDict_Merge`](https://docs.python.org/3/c-api/dict.html#c.PyDict_Merge) internally,
    /// so should have the same performance as `update`.
    fn update_if_missing(&self, other: &Bound<'_, PyMapping>) -> PyResult<()>;
}

impl<'py> PyDictMethods<'py> for Bound<'py, PyDict> {
    fn copy(&self) -> PyResult<Bound<'py, PyDict>> {
        unsafe {
            ffi::PyDict_Copy(self.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
    }

    fn clear(&self) {
        unsafe { ffi::PyDict_Clear(self.as_ptr()) }
    }

    fn len(&self) -> usize {
        dict_len(self) as usize
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        fn inner(dict: &Bound<'_, PyDict>, key: Bound<'_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PyDict_Contains(dict.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(dict.py())),
            }
        }

        let py = self.py();
        inner(self, key.to_object(py).into_bound(py))
    }

    fn get_item<K>(&self, key: K) -> PyResult<Option<Bound<'py, PyAny>>>
    where
        K: ToPyObject,
    {
        fn inner<'py>(
            dict: &Bound<'py, PyDict>,
            key: Bound<'_, PyAny>,
        ) -> PyResult<Option<Bound<'py, PyAny>>> {
            let py = dict.py();
            match unsafe {
                ffi::PyDict_GetItemWithError(dict.as_ptr(), key.as_ptr())
                    .assume_borrowed_or_opt(py)
                    .map(Borrowed::to_owned)
            } {
                some @ Some(_) => Ok(some),
                None => PyErr::take(py).map(Err).transpose(),
            }
        }

        let py = self.py();
        inner(self, key.to_object(py).into_bound(py))
    }

    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject,
    {
        fn inner(
            dict: &Bound<'_, PyDict>,
            key: Bound<'_, PyAny>,
            value: Bound<'_, PyAny>,
        ) -> PyResult<()> {
            err::error_on_minusone(dict.py(), unsafe {
                ffi::PyDict_SetItem(dict.as_ptr(), key.as_ptr(), value.as_ptr())
            })
        }

        let py = self.py();
        inner(
            self,
            key.to_object(py).into_bound(py),
            value.to_object(py).into_bound(py),
        )
    }

    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        fn inner(dict: &Bound<'_, PyDict>, key: Bound<'_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(dict.py(), unsafe {
                ffi::PyDict_DelItem(dict.as_ptr(), key.as_ptr())
            })
        }

        let py = self.py();
        inner(self, key.to_object(py).into_bound(py))
    }

    fn keys(&self) -> Bound<'py, PyList> {
        unsafe {
            ffi::PyDict_Keys(self.as_ptr())
                .assume_owned(self.py())
                .downcast_into_unchecked()
        }
    }

    fn values(&self) -> Bound<'py, PyList> {
        unsafe {
            ffi::PyDict_Values(self.as_ptr())
                .assume_owned(self.py())
                .downcast_into_unchecked()
        }
    }

    fn items(&self) -> Bound<'py, PyList> {
        unsafe {
            ffi::PyDict_Items(self.as_ptr())
                .assume_owned(self.py())
                .downcast_into_unchecked()
        }
    }

    fn iter(&self) -> BoundDictIterator<'py> {
        BoundDictIterator::new(self.clone())
    }

    fn as_mapping(&self) -> &Bound<'py, PyMapping> {
        unsafe { self.downcast_unchecked() }
    }

    fn into_mapping(self) -> Bound<'py, PyMapping> {
        unsafe { self.into_any().downcast_into_unchecked() }
    }

    fn update(&self, other: &Bound<'_, PyMapping>) -> PyResult<()> {
        err::error_on_minusone(self.py(), unsafe {
            ffi::PyDict_Update(self.as_ptr(), other.as_ptr())
        })
    }

    fn update_if_missing(&self, other: &Bound<'_, PyMapping>) -> PyResult<()> {
        err::error_on_minusone(self.py(), unsafe {
            ffi::PyDict_Merge(self.as_ptr(), other.as_ptr(), 0)
        })
    }
}

impl<'a, 'py> Borrowed<'a, 'py, PyDict> {
    /// Iterates over the contents of this dictionary without incrementing reference counts.
    ///
    /// # Safety
    /// It must be known that this dictionary will not be modified during iteration.
    pub(crate) unsafe fn iter_borrowed(self) -> BorrowedDictIter<'a, 'py> {
        BorrowedDictIter::new(self)
    }
}

fn dict_len(dict: &Bound<'_, PyDict>) -> Py_ssize_t {
    #[cfg(any(not(Py_3_8), PyPy, GraalPy, Py_LIMITED_API))]
    unsafe {
        ffi::PyDict_Size(dict.as_ptr())
    }

    #[cfg(all(Py_3_8, not(PyPy), not(GraalPy), not(Py_LIMITED_API)))]
    unsafe {
        (*dict.as_ptr().cast::<ffi::PyDictObject>()).ma_used
    }
}

/// PyO3 implementation of an iterator for a Python `dict` object.
#[cfg(feature = "gil-refs")]
pub struct PyDictIterator<'py>(BoundDictIterator<'py>);

#[cfg(feature = "gil-refs")]
impl<'py> Iterator for PyDictIterator<'py> {
    type Item = (&'py PyAny, &'py PyAny);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (key, value) = self.0.next()?;
        Some((key.into_gil_ref(), value.into_gil_ref()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[cfg(feature = "gil-refs")]
impl<'py> ExactSizeIterator for PyDictIterator<'py> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(feature = "gil-refs")]
impl<'a> IntoIterator for &'a PyDict {
    type Item = (&'a PyAny, &'a PyAny);
    type IntoIter = PyDictIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// PyO3 implementation of an iterator for a Python `dict` object.
pub struct BoundDictIterator<'py> {
    dict: Bound<'py, PyDict>,
    ppos: ffi::Py_ssize_t,
    di_used: ffi::Py_ssize_t,
    len: ffi::Py_ssize_t,
}

impl<'py> Iterator for BoundDictIterator<'py> {
    type Item = (Bound<'py, PyAny>, Bound<'py, PyAny>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ma_used = dict_len(&self.dict);

        // These checks are similar to what CPython does.
        //
        // If the dimension of the dict changes e.g. key-value pairs are removed
        // or added during iteration, this will panic next time when `next` is called
        if self.di_used != ma_used {
            self.di_used = -1;
            panic!("dictionary changed size during iteration");
        };

        // If the dict is changed in such a way that the length remains constant
        // then this will panic at the end of iteration - similar to this:
        //
        // d = {"a":1, "b":2, "c": 3}
        //
        // for k, v in d.items():
        //     d[f"{k}_"] = 4
        //     del d[k]
        //     print(k)
        //
        if self.len == -1 {
            self.di_used = -1;
            panic!("dictionary keys changed during iteration");
        };

        let mut key: *mut ffi::PyObject = std::ptr::null_mut();
        let mut value: *mut ffi::PyObject = std::ptr::null_mut();

        if unsafe { ffi::PyDict_Next(self.dict.as_ptr(), &mut self.ppos, &mut key, &mut value) }
            != 0
        {
            self.len -= 1;
            let py = self.dict.py();
            // Safety:
            // - PyDict_Next returns borrowed values
            // - we have already checked that `PyDict_Next` succeeded, so we can assume these to be non-null
            Some((
                unsafe { key.assume_borrowed_unchecked(py) }.to_owned(),
                unsafe { value.assume_borrowed_unchecked(py) }.to_owned(),
            ))
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'py> ExactSizeIterator for BoundDictIterator<'py> {
    fn len(&self) -> usize {
        self.len as usize
    }
}

impl<'py> BoundDictIterator<'py> {
    fn new(dict: Bound<'py, PyDict>) -> Self {
        let len = dict_len(&dict);
        BoundDictIterator {
            dict,
            ppos: 0,
            di_used: len,
            len,
        }
    }
}

impl<'py> IntoIterator for Bound<'py, PyDict> {
    type Item = (Bound<'py, PyAny>, Bound<'py, PyAny>);
    type IntoIter = BoundDictIterator<'py>;

    fn into_iter(self) -> Self::IntoIter {
        BoundDictIterator::new(self)
    }
}

impl<'py> IntoIterator for &Bound<'py, PyDict> {
    type Item = (Bound<'py, PyAny>, Bound<'py, PyAny>);
    type IntoIter = BoundDictIterator<'py>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

mod borrowed_iter {
    use super::*;

    /// Variant of the above which is used to iterate the items of the dictionary
    /// without incrementing reference counts. This is only safe if it's known
    /// that the dictionary will not be modified during iteration.
    pub struct BorrowedDictIter<'a, 'py> {
        dict: Borrowed<'a, 'py, PyDict>,
        ppos: ffi::Py_ssize_t,
        len: ffi::Py_ssize_t,
    }

    impl<'a, 'py> Iterator for BorrowedDictIter<'a, 'py> {
        type Item = (Borrowed<'a, 'py, PyAny>, Borrowed<'a, 'py, PyAny>);

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            let mut key: *mut ffi::PyObject = std::ptr::null_mut();
            let mut value: *mut ffi::PyObject = std::ptr::null_mut();

            // Safety: self.dict lives sufficiently long that the pointer is not dangling
            if unsafe { ffi::PyDict_Next(self.dict.as_ptr(), &mut self.ppos, &mut key, &mut value) }
                != 0
            {
                let py = self.dict.py();
                self.len -= 1;
                // Safety:
                // - PyDict_Next returns borrowed values
                // - we have already checked that `PyDict_Next` succeeded, so we can assume these to be non-null
                Some(unsafe { (key.assume_borrowed(py), value.assume_borrowed(py)) })
            } else {
                None
            }
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = self.len();
            (len, Some(len))
        }
    }

    impl ExactSizeIterator for BorrowedDictIter<'_, '_> {
        fn len(&self) -> usize {
            self.len as usize
        }
    }

    impl<'a, 'py> BorrowedDictIter<'a, 'py> {
        pub(super) fn new(dict: Borrowed<'a, 'py, PyDict>) -> Self {
            let len = dict_len(&dict);
            BorrowedDictIter { dict, ppos: 0, len }
        }
    }
}

pub(crate) use borrowed_iter::BorrowedDictIter;

/// Conversion trait that allows a sequence of tuples to be converted into `PyDict`
/// Primary use case for this trait is `call` and `call_method` methods as keywords argument.
pub trait IntoPyDict: Sized {
    /// Converts self into a `PyDict` object pointer. Whether pointer owned or borrowed
    /// depends on implementation.
    #[cfg(feature = "gil-refs")]
    #[deprecated(
        since = "0.21.0",
        note = "`IntoPyDict::into_py_dict` will be replaced by `IntoPyDict::into_py_dict_bound` in a future PyO3 version"
    )]
    fn into_py_dict(self, py: Python<'_>) -> &PyDict {
        Self::into_py_dict_bound(self, py).into_gil_ref()
    }

    /// Converts self into a `PyDict` object pointer. Whether pointer owned or borrowed
    /// depends on implementation.
    fn into_py_dict_bound(self, py: Python<'_>) -> Bound<'_, PyDict>;
}

impl<T, I> IntoPyDict for I
where
    T: PyDictItem,
    I: IntoIterator<Item = T>,
{
    fn into_py_dict_bound(self, py: Python<'_>) -> Bound<'_, PyDict> {
        let dict = PyDict::new_bound(py);
        for item in self {
            dict.set_item(item.key(), item.value())
                .expect("Failed to set_item on dict");
        }
        dict
    }
}

/// Represents a tuple which can be used as a PyDict item.
pub trait PyDictItem {
    type K: ToPyObject;
    type V: ToPyObject;
    fn key(&self) -> &Self::K;
    fn value(&self) -> &Self::V;
}

impl<K, V> PyDictItem for (K, V)
where
    K: ToPyObject,
    V: ToPyObject,
{
    type K = K;
    type V = V;
    fn key(&self) -> &Self::K {
        &self.0
    }
    fn value(&self) -> &Self::V {
        &self.1
    }
}

impl<K, V> PyDictItem for &(K, V)
where
    K: ToPyObject,
    V: ToPyObject,
{
    type K = K;
    type V = V;
    fn key(&self) -> &Self::K {
        &self.0
    }
    fn value(&self) -> &Self::V {
        &self.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyTuple;
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn test_new() {
        Python::with_gil(|py| {
            let dict = [(7, 32)].into_py_dict_bound(py);
            assert_eq!(
                32,
                dict.get_item(7i32)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert!(dict.get_item(8i32).unwrap().is_none());
            let map: HashMap<i32, i32> = [(7, 32)].iter().cloned().collect();
            assert_eq!(map, dict.extract().unwrap());
            let map: BTreeMap<i32, i32> = [(7, 32)].iter().cloned().collect();
            assert_eq!(map, dict.extract().unwrap());
        });
    }

    #[test]
    #[cfg(not(any(PyPy, GraalPy)))]
    fn test_from_sequence() {
        Python::with_gil(|py| {
            let items = PyList::new_bound(py, vec![("a", 1), ("b", 2)]);
            let dict = PyDict::from_sequence_bound(&items).unwrap();
            assert_eq!(
                1,
                dict.get_item("a")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert_eq!(
                2,
                dict.get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            let map: HashMap<String, i32> =
                [("a".into(), 1), ("b".into(), 2)].into_iter().collect();
            assert_eq!(map, dict.extract().unwrap());
            let map: BTreeMap<String, i32> =
                [("a".into(), 1), ("b".into(), 2)].into_iter().collect();
            assert_eq!(map, dict.extract().unwrap());
        });
    }

    #[test]
    #[cfg(not(any(PyPy, GraalPy)))]
    fn test_from_sequence_err() {
        Python::with_gil(|py| {
            let items = PyList::new_bound(py, vec!["a", "b"]);
            assert!(PyDict::from_sequence_bound(&items).is_err());
        });
    }

    #[test]
    fn test_copy() {
        Python::with_gil(|py| {
            let dict = [(7, 32)].into_py_dict_bound(py);

            let ndict = dict.copy().unwrap();
            assert_eq!(
                32,
                ndict
                    .get_item(7i32)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert!(ndict.get_item(8i32).unwrap().is_none());
        });
    }

    #[test]
    fn test_len() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            assert_eq!(0, dict.len());
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict2 = ob.downcast_bound::<PyDict>(py).unwrap();
            assert_eq!(1, dict2.len());
        });
    }

    #[test]
    fn test_contains() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            assert!(dict.contains(7i32).unwrap());
            assert!(!dict.contains(8i32).unwrap());
        });
    }

    #[test]
    fn test_get_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            assert_eq!(
                32,
                dict.get_item(7i32)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert!(dict.get_item(8i32).unwrap().is_none());
        });
    }

    #[test]
    fn test_set_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            assert!(dict.set_item(7i32, 42i32).is_ok()); // change
            assert!(dict.set_item(8i32, 123i32).is_ok()); // insert
            assert_eq!(
                42i32,
                dict.get_item(7i32)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert_eq!(
                123i32,
                dict.get_item(8i32)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
        });
    }

    #[test]
    fn test_set_item_refcnt() {
        Python::with_gil(|py| {
            let cnt;
            let obj = py.eval_bound("object()", None, None).unwrap();
            {
                cnt = obj.get_refcnt();
                let _dict = [(10, &obj)].into_py_dict_bound(py);
            }
            {
                assert_eq!(cnt, obj.get_refcnt());
            }
        });
    }

    #[test]
    fn test_set_item_does_not_update_original_object() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            assert!(dict.set_item(7i32, 42i32).is_ok()); // change
            assert!(dict.set_item(8i32, 123i32).is_ok()); // insert
            assert_eq!(32i32, v[&7i32]); // not updated!
            assert_eq!(None, v.get(&8i32));
        });
    }

    #[test]
    fn test_del_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            assert!(dict.del_item(7i32).is_ok());
            assert_eq!(0, dict.len());
            assert!(dict.get_item(7i32).unwrap().is_none());
        });
    }

    #[test]
    fn test_del_item_does_not_update_original_object() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            assert!(dict.del_item(7i32).is_ok()); // change
            assert_eq!(32i32, *v.get(&7i32).unwrap()); // not updated!
        });
    }

    #[test]
    fn test_items() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            let mut value_sum = 0;
            for el in dict.items() {
                let tuple = el.downcast::<PyTuple>().unwrap();
                key_sum += tuple.get_item(0).unwrap().extract::<i32>().unwrap();
                value_sum += tuple.get_item(1).unwrap().extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_keys() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            for el in dict.keys() {
                key_sum += el.extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
        });
    }

    #[test]
    fn test_values() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut values_sum = 0;
            for el in dict.values() {
                values_sum += el.extract::<i32>().unwrap();
            }
            assert_eq!(32 + 42 + 123, values_sum);
        });
    }

    #[test]
    fn test_iter() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            let mut key_sum = 0;
            let mut value_sum = 0;
            for (key, value) in dict {
                key_sum += key.extract::<i32>().unwrap();
                value_sum += value.extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_iter_bound() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.to_object(py);
            let dict: &Bound<'_, PyDict> = ob.downcast_bound(py).unwrap();
            let mut key_sum = 0;
            let mut value_sum = 0;
            for (key, value) in dict {
                key_sum += key.extract::<i32>().unwrap();
                value_sum += value.extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_iter_value_mutated() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);

            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();

            for (key, value) in dict {
                dict.set_item(key, value.extract::<i32>().unwrap() + 7)
                    .unwrap();
            }
        });
    }

    #[test]
    #[should_panic]
    fn test_iter_key_mutated() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            for i in 0..10 {
                v.insert(i * 2, i * 2);
            }
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();

            for (i, (key, value)) in dict.iter().enumerate() {
                let key = key.extract::<i32>().unwrap();
                let value = value.extract::<i32>().unwrap();

                dict.set_item(key + 1, value + 1).unwrap();

                if i > 1000 {
                    // avoid this test just running out of memory if it fails
                    break;
                };
            }
        });
    }

    #[test]
    #[should_panic]
    fn test_iter_key_mutated_constant_len() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            for i in 0..10 {
                v.insert(i * 2, i * 2);
            }
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();

            for (i, (key, value)) in dict.iter().enumerate() {
                let key = key.extract::<i32>().unwrap();
                let value = value.extract::<i32>().unwrap();
                dict.del_item(key).unwrap();
                dict.set_item(key + 1, value + 1).unwrap();

                if i > 1000 {
                    // avoid this test just running out of memory if it fails
                    break;
                };
            }
        });
    }

    #[test]
    fn test_iter_size_hint() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();

            let mut iter = dict.iter();
            assert_eq!(iter.size_hint(), (v.len(), Some(v.len())));
            iter.next();
            assert_eq!(iter.size_hint(), (v.len() - 1, Some(v.len() - 1)));

            // Exhaust iterator.
            for _ in &mut iter {}

            assert_eq!(iter.size_hint(), (0, Some(0)));

            assert!(iter.next().is_none());

            assert_eq!(iter.size_hint(), (0, Some(0)));
        });
    }

    #[test]
    fn test_into_iter() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.to_object(py);
            let dict = ob.downcast_bound::<PyDict>(py).unwrap();
            let mut key_sum = 0;
            let mut value_sum = 0;
            for (key, value) in dict {
                key_sum += key.extract::<i32>().unwrap();
                value_sum += value.extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_hashmap_into_dict() {
        Python::with_gil(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict_bound(py);

            assert_eq!(py_map.len(), 1);
            assert_eq!(
                py_map
                    .get_item(1)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
        });
    }

    #[test]
    fn test_btreemap_into_dict() {
        Python::with_gil(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict_bound(py);

            assert_eq!(py_map.len(), 1);
            assert_eq!(
                py_map
                    .get_item(1)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
        });
    }

    #[test]
    fn test_vec_into_dict() {
        Python::with_gil(|py| {
            let vec = vec![("a", 1), ("b", 2), ("c", 3)];
            let py_map = vec.into_py_dict_bound(py);

            assert_eq!(py_map.len(), 3);
            assert_eq!(
                py_map
                    .get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                2
            );
        });
    }

    #[test]
    fn test_slice_into_dict() {
        Python::with_gil(|py| {
            let arr = [("a", 1), ("b", 2), ("c", 3)];
            let py_map = arr.into_py_dict_bound(py);

            assert_eq!(py_map.len(), 3);
            assert_eq!(
                py_map
                    .get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                2
            );
        });
    }

    #[test]
    fn dict_as_mapping() {
        Python::with_gil(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict_bound(py);

            assert_eq!(py_map.as_mapping().len().unwrap(), 1);
            assert_eq!(
                py_map
                    .as_mapping()
                    .get_item(1)
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
        });
    }

    #[test]
    fn dict_into_mapping() {
        Python::with_gil(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict_bound(py);

            let py_mapping = py_map.into_mapping();
            assert_eq!(py_mapping.len().unwrap(), 1);
            assert_eq!(py_mapping.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[cfg(not(any(PyPy, GraalPy)))]
    fn abc_dict(py: Python<'_>) -> Bound<'_, PyDict> {
        let mut map = HashMap::<&'static str, i32>::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);
        map.into_py_dict_bound(py)
    }

    #[test]
    #[cfg(not(any(PyPy, GraalPy)))]
    fn dict_keys_view() {
        Python::with_gil(|py| {
            let dict = abc_dict(py);
            let keys = dict.call_method0("keys").unwrap();
            assert!(keys
                .is_instance(&py.get_type_bound::<PyDictKeys>().as_borrowed())
                .unwrap());
        })
    }

    #[test]
    #[cfg(not(any(PyPy, GraalPy)))]
    fn dict_values_view() {
        Python::with_gil(|py| {
            let dict = abc_dict(py);
            let values = dict.call_method0("values").unwrap();
            assert!(values
                .is_instance(&py.get_type_bound::<PyDictValues>().as_borrowed())
                .unwrap());
        })
    }

    #[test]
    #[cfg(not(any(PyPy, GraalPy)))]
    fn dict_items_view() {
        Python::with_gil(|py| {
            let dict = abc_dict(py);
            let items = dict.call_method0("items").unwrap();
            assert!(items
                .is_instance(&py.get_type_bound::<PyDictItems>().as_borrowed())
                .unwrap());
        })
    }

    #[test]
    fn dict_update() {
        Python::with_gil(|py| {
            let dict = [("a", 1), ("b", 2), ("c", 3)].into_py_dict_bound(py);
            let other = [("b", 4), ("c", 5), ("d", 6)].into_py_dict_bound(py);
            dict.update(other.as_mapping()).unwrap();
            assert_eq!(dict.len(), 4);
            assert_eq!(
                dict.get_item("a")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
            assert_eq!(
                dict.get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                4
            );
            assert_eq!(
                dict.get_item("c")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                5
            );
            assert_eq!(
                dict.get_item("d")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                6
            );

            assert_eq!(other.len(), 3);
            assert_eq!(
                other
                    .get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                4
            );
            assert_eq!(
                other
                    .get_item("c")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                5
            );
            assert_eq!(
                other
                    .get_item("d")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                6
            );
        })
    }

    #[test]
    fn dict_update_if_missing() {
        Python::with_gil(|py| {
            let dict = [("a", 1), ("b", 2), ("c", 3)].into_py_dict_bound(py);
            let other = [("b", 4), ("c", 5), ("d", 6)].into_py_dict_bound(py);
            dict.update_if_missing(other.as_mapping()).unwrap();
            assert_eq!(dict.len(), 4);
            assert_eq!(
                dict.get_item("a")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
            assert_eq!(
                dict.get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                2
            );
            assert_eq!(
                dict.get_item("c")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                3
            );
            assert_eq!(
                dict.get_item("d")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                6
            );

            assert_eq!(other.len(), 3);
            assert_eq!(
                other
                    .get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                4
            );
            assert_eq!(
                other
                    .get_item("c")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                5
            );
            assert_eq!(
                other
                    .get_item("d")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                6
            );
        })
    }
}
