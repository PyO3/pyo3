use crate::err::{self, PyErr, PyResult};
use crate::ffi::Py_ssize_t;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::{Borrowed, Bound};
use crate::py_result_ext::PyResultExt;
use crate::types::{PyAny, PyList, PyMapping};
use crate::{ffi, BoundObject, IntoPyObject, IntoPyObjectExt, Python};

/// Represents a Python `dict`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyDict>`][crate::Py] or [`Bound<'py, PyDict>`][Bound].
///
/// For APIs available on `dict` objects, see the [`PyDictMethods`] trait which is implemented for
/// [`Bound<'py, PyDict>`][Bound].
#[repr(transparent)]
pub struct PyDict(PyAny);

pyobject_subclassable_native_type!(PyDict, crate::ffi::PyDictObject);

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
    pub fn new(py: Python<'_>) -> Bound<'_, PyDict> {
        unsafe { ffi::PyDict_New().assume_owned(py).cast_into_unchecked() }
    }

    /// Creates a new dictionary from the sequence given.
    ///
    /// The sequence must consist of `(PyObject, PyObject)`. This is
    /// equivalent to `dict([("a", 1), ("b", 2)])`.
    ///
    /// Returns an error on invalid input. In the case of key collisions,
    /// this keeps the last entry seen.
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn from_sequence<'py>(seq: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
        let py = seq.py();
        let dict = Self::new(py);
        err::error_on_minusone(py, unsafe {
            ffi::PyDict_MergeFromSeq2(dict.as_ptr(), seq.as_ptr(), 1)
        })?;
        Ok(dict)
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
        K: IntoPyObject<'py>;

    /// Gets an item from the dictionary.
    ///
    /// Returns `None` if the item is not present, or if an error occurs.
    ///
    /// To get a `KeyError` for non-existing keys, use `PyAny::get_item`.
    fn get_item<K>(&self, key: K) -> PyResult<Option<Bound<'py, PyAny>>>
    where
        K: IntoPyObject<'py>;

    /// Sets an item value.
    ///
    /// This is equivalent to the Python statement `self[key] = value`.
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: IntoPyObject<'py>,
        V: IntoPyObject<'py>;

    /// Deletes an item.
    ///
    /// This is equivalent to the Python statement `del self[key]`.
    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: IntoPyObject<'py>;

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

    /// Iterates over the contents of this dictionary while holding a critical section on the dict.
    /// This is useful when the GIL is disabled and the dictionary is shared between threads.
    /// It is not guaranteed that the dictionary will not be modified during iteration when the
    /// closure calls arbitrary Python code that releases the critical section held by the
    /// iterator. Otherwise, the dictionary will not be modified during iteration.
    ///
    /// This method is a small performance optimization over `.iter().try_for_each()` when the
    /// nightly feature is not enabled because we cannot implement an optimised version of
    /// `iter().try_fold()` on stable yet. If your iteration is infallible then this method has the
    /// same performance as `.iter().for_each()`.
    fn locked_for_each<F>(&self, closure: F) -> PyResult<()>
    where
        F: Fn(Bound<'py, PyAny>, Bound<'py, PyAny>) -> PyResult<()>;

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
                .cast_into_unchecked()
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
        K: IntoPyObject<'py>,
    {
        fn inner(dict: &Bound<'_, PyDict>, key: Borrowed<'_, '_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PyDict_Contains(dict.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(dict.py())),
            }
        }

        let py = self.py();
        inner(
            self,
            key.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn get_item<K>(&self, key: K) -> PyResult<Option<Bound<'py, PyAny>>>
    where
        K: IntoPyObject<'py>,
    {
        fn inner<'py>(
            dict: &Bound<'py, PyDict>,
            key: Borrowed<'_, '_, PyAny>,
        ) -> PyResult<Option<Bound<'py, PyAny>>> {
            let py = dict.py();
            let mut result: *mut ffi::PyObject = std::ptr::null_mut();
            match unsafe {
                ffi::compat::PyDict_GetItemRef(dict.as_ptr(), key.as_ptr(), &mut result)
            } {
                std::ffi::c_int::MIN..=-1 => Err(PyErr::fetch(py)),
                0 => Ok(None),
                1..=std::ffi::c_int::MAX => {
                    // Safety: PyDict_GetItemRef positive return value means the result is a valid
                    // owned reference
                    Ok(Some(unsafe { result.assume_owned_unchecked(py) }))
                }
            }
        }

        let py = self.py();
        inner(
            self,
            key.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: IntoPyObject<'py>,
        V: IntoPyObject<'py>,
    {
        fn inner(
            dict: &Bound<'_, PyDict>,
            key: Borrowed<'_, '_, PyAny>,
            value: Borrowed<'_, '_, PyAny>,
        ) -> PyResult<()> {
            err::error_on_minusone(dict.py(), unsafe {
                ffi::PyDict_SetItem(dict.as_ptr(), key.as_ptr(), value.as_ptr())
            })
        }

        let py = self.py();
        inner(
            self,
            key.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
            value.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: IntoPyObject<'py>,
    {
        fn inner(dict: &Bound<'_, PyDict>, key: Borrowed<'_, '_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(dict.py(), unsafe {
                ffi::PyDict_DelItem(dict.as_ptr(), key.as_ptr())
            })
        }

        let py = self.py();
        inner(
            self,
            key.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn keys(&self) -> Bound<'py, PyList> {
        unsafe {
            ffi::PyDict_Keys(self.as_ptr())
                .assume_owned(self.py())
                .cast_into_unchecked()
        }
    }

    fn values(&self) -> Bound<'py, PyList> {
        unsafe {
            ffi::PyDict_Values(self.as_ptr())
                .assume_owned(self.py())
                .cast_into_unchecked()
        }
    }

    fn items(&self) -> Bound<'py, PyList> {
        unsafe {
            ffi::PyDict_Items(self.as_ptr())
                .assume_owned(self.py())
                .cast_into_unchecked()
        }
    }

    fn iter(&self) -> BoundDictIterator<'py> {
        BoundDictIterator::new(self.clone())
    }

    fn locked_for_each<F>(&self, f: F) -> PyResult<()>
    where
        F: Fn(Bound<'py, PyAny>, Bound<'py, PyAny>) -> PyResult<()>,
    {
        #[cfg(feature = "nightly")]
        {
            // We don't need a critical section when the nightly feature is enabled because
            // try_for_each is locked by the implementation of try_fold.
            self.iter().try_for_each(|(key, value)| f(key, value))
        }

        #[cfg(not(feature = "nightly"))]
        {
            crate::sync::with_critical_section(self, || {
                self.iter().try_for_each(|(key, value)| f(key, value))
            })
        }
    }

    fn as_mapping(&self) -> &Bound<'py, PyMapping> {
        unsafe { self.cast_unchecked() }
    }

    fn into_mapping(self) -> Bound<'py, PyMapping> {
        unsafe { self.cast_into_unchecked() }
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
    /// It must be known that this dictionary will not be modified during iteration,
    /// for example, when parsing arguments in a keyword arguments dictionary.
    pub(crate) unsafe fn iter_borrowed(self) -> BorrowedDictIter<'a, 'py> {
        BorrowedDictIter::new(self)
    }
}

fn dict_len(dict: &Bound<'_, PyDict>) -> Py_ssize_t {
    #[cfg(any(not(Py_3_8), PyPy, GraalPy, Py_LIMITED_API, Py_GIL_DISABLED))]
    unsafe {
        ffi::PyDict_Size(dict.as_ptr())
    }

    #[cfg(all(
        Py_3_8,
        not(PyPy),
        not(GraalPy),
        not(Py_LIMITED_API),
        not(Py_GIL_DISABLED)
    ))]
    unsafe {
        (*dict.as_ptr().cast::<ffi::PyDictObject>()).ma_used
    }
}

/// PyO3 implementation of an iterator for a Python `dict` object.
pub struct BoundDictIterator<'py> {
    dict: Bound<'py, PyDict>,
    inner: DictIterImpl,
}

enum DictIterImpl {
    DictIter {
        ppos: ffi::Py_ssize_t,
        di_used: ffi::Py_ssize_t,
        remaining: ffi::Py_ssize_t,
    },
}

impl DictIterImpl {
    #[deny(unsafe_op_in_unsafe_fn)]
    #[inline]
    /// Safety: the dict should be locked with a critical section on the free-threaded build
    /// and otherwise not shared between threads in code that releases the GIL.
    unsafe fn next_unchecked<'py>(
        &mut self,
        dict: &Bound<'py, PyDict>,
    ) -> Option<(Bound<'py, PyAny>, Bound<'py, PyAny>)> {
        match self {
            Self::DictIter {
                di_used,
                remaining,
                ppos,
                ..
            } => {
                let ma_used = dict_len(dict);

                // These checks are similar to what CPython does.
                //
                // If the dimension of the dict changes e.g. key-value pairs are removed
                // or added during iteration, this will panic next time when `next` is called
                if *di_used != ma_used {
                    *di_used = -1;
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
                if *remaining == -1 {
                    *di_used = -1;
                    panic!("dictionary keys changed during iteration");
                };

                let mut key: *mut ffi::PyObject = std::ptr::null_mut();
                let mut value: *mut ffi::PyObject = std::ptr::null_mut();

                if unsafe { ffi::PyDict_Next(dict.as_ptr(), ppos, &mut key, &mut value) != 0 } {
                    *remaining -= 1;
                    let py = dict.py();
                    // Safety:
                    // - PyDict_Next returns borrowed values
                    // - we have already checked that `PyDict_Next` succeeded, so we can assume these to be non-null
                    Some((
                        unsafe { key.assume_borrowed_unchecked(py).to_owned() },
                        unsafe { value.assume_borrowed_unchecked(py).to_owned() },
                    ))
                } else {
                    None
                }
            }
        }
    }

    #[cfg(Py_GIL_DISABLED)]
    #[inline]
    fn with_critical_section<F, R>(&mut self, dict: &Bound<'_, PyDict>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        match self {
            Self::DictIter { .. } => crate::sync::with_critical_section(dict, || f(self)),
        }
    }
}

impl<'py> Iterator for BoundDictIterator<'py> {
    type Item = (Bound<'py, PyAny>, Bound<'py, PyAny>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(Py_GIL_DISABLED)]
        {
            self.inner
                .with_critical_section(&self.dict, |inner| unsafe {
                    inner.next_unchecked(&self.dict)
                })
        }
        #[cfg(not(Py_GIL_DISABLED))]
        {
            unsafe { self.inner.next_unchecked(&self.dict) }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    #[inline]
    #[cfg(Py_GIL_DISABLED)]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.with_critical_section(&self.dict, |inner| {
            let mut accum = init;
            while let Some(x) = unsafe { inner.next_unchecked(&self.dict) } {
                accum = f(accum, x);
            }
            accum
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, feature = "nightly"))]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> R,
        R: std::ops::Try<Output = B>,
    {
        self.inner.with_critical_section(&self.dict, |inner| {
            let mut accum = init;
            while let Some(x) = unsafe { inner.next_unchecked(&self.dict) } {
                accum = f(accum, x)?
            }
            R::from_output(accum)
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(feature = "nightly")))]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        self.inner.with_critical_section(&self.dict, |inner| {
            while let Some(x) = unsafe { inner.next_unchecked(&self.dict) } {
                if !f(x) {
                    return false;
                }
            }
            true
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(feature = "nightly")))]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        self.inner.with_critical_section(&self.dict, |inner| {
            while let Some(x) = unsafe { inner.next_unchecked(&self.dict) } {
                if f(x) {
                    return true;
                }
            }
            false
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(feature = "nightly")))]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        self.inner.with_critical_section(&self.dict, |inner| {
            while let Some(x) = unsafe { inner.next_unchecked(&self.dict) } {
                if predicate(&x) {
                    return Some(x);
                }
            }
            None
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(feature = "nightly")))]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        self.inner.with_critical_section(&self.dict, |inner| {
            while let Some(x) = unsafe { inner.next_unchecked(&self.dict) } {
                if let found @ Some(_) = f(x) {
                    return found;
                }
            }
            None
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(feature = "nightly")))]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        self.inner.with_critical_section(&self.dict, |inner| {
            let mut acc = 0;
            while let Some(x) = unsafe { inner.next_unchecked(&self.dict) } {
                if predicate(x) {
                    return Some(acc);
                }
                acc += 1;
            }
            None
        })
    }
}

impl ExactSizeIterator for BoundDictIterator<'_> {
    fn len(&self) -> usize {
        match self.inner {
            DictIterImpl::DictIter { remaining, .. } => remaining as usize,
        }
    }
}

impl<'py> BoundDictIterator<'py> {
    fn new(dict: Bound<'py, PyDict>) -> Self {
        let remaining = dict_len(&dict);

        Self {
            dict,
            inner: DictIterImpl::DictIter {
                ppos: 0,
                di_used: remaining,
                remaining,
            },
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

        #[inline]
        fn count(self) -> usize
        where
            Self: Sized,
        {
            self.len()
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
pub trait IntoPyDict<'py>: Sized {
    /// Converts self into a `PyDict` object pointer. Whether pointer owned or borrowed
    /// depends on implementation.
    fn into_py_dict(self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>>;
}

impl<'py, T, I> IntoPyDict<'py> for I
where
    T: PyDictItem<'py>,
    I: IntoIterator<Item = T>,
{
    fn into_py_dict(self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        self.into_iter().try_for_each(|item| {
            let (key, value) = item.unpack();
            dict.set_item(key, value)
        })?;
        Ok(dict)
    }
}

/// Represents a tuple which can be used as a PyDict item.
trait PyDictItem<'py> {
    type K: IntoPyObject<'py>;
    type V: IntoPyObject<'py>;
    fn unpack(self) -> (Self::K, Self::V);
}

impl<'py, K, V> PyDictItem<'py> for (K, V)
where
    K: IntoPyObject<'py>,
    V: IntoPyObject<'py>,
{
    type K = K;
    type V = V;

    fn unpack(self) -> (Self::K, Self::V) {
        (self.0, self.1)
    }
}

impl<'a, 'py, K, V> PyDictItem<'py> for &'a (K, V)
where
    &'a K: IntoPyObject<'py>,
    &'a V: IntoPyObject<'py>,
{
    type K = &'a K;
    type V = &'a V;

    fn unpack(self) -> (Self::K, Self::V) {
        (&self.0, &self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PyAnyMethods as _, PyTuple};
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn test_new() {
        Python::attach(|py| {
            let dict = [(7, 32)].into_py_dict(py).unwrap();
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
        Python::attach(|py| {
            let items = PyList::new(py, vec![("a", 1), ("b", 2)]).unwrap();
            let dict = PyDict::from_sequence(&items).unwrap();
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
        Python::attach(|py| {
            let items = PyList::new(py, vec!["a", "b"]).unwrap();
            assert!(PyDict::from_sequence(&items).is_err());
        });
    }

    #[test]
    fn test_copy() {
        Python::attach(|py| {
            let dict = [(7, 32)].into_py_dict(py).unwrap();

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
        Python::attach(|py| {
            let mut v = HashMap::<i32, i32>::new();
            let dict = (&v).into_pyobject(py).unwrap();
            assert_eq!(0, dict.len());
            v.insert(7, 32);
            let dict2 = v.into_pyobject(py).unwrap();
            assert_eq!(1, dict2.len());
        });
    }

    #[test]
    fn test_contains() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let dict = v.into_pyobject(py).unwrap();
            assert!(dict.contains(7i32).unwrap());
            assert!(!dict.contains(8i32).unwrap());
        });
    }

    #[test]
    fn test_get_item() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let dict = v.into_pyobject(py).unwrap();
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

    #[cfg(feature = "macros")]
    #[test]
    fn test_get_item_error_path() {
        use crate::exceptions::PyTypeError;

        #[crate::pyclass(crate = "crate")]
        struct HashErrors;

        #[crate::pymethods(crate = "crate")]
        impl HashErrors {
            #[new]
            fn new() -> Self {
                HashErrors {}
            }

            fn __hash__(&self) -> PyResult<isize> {
                Err(PyTypeError::new_err("Error from __hash__"))
            }
        }

        Python::attach(|py| {
            let class = py.get_type::<HashErrors>();
            let instance = class.call0().unwrap();
            let d = PyDict::new(py);
            match d.get_item(instance) {
                Ok(_) => {
                    panic!("this get_item call should always error")
                }
                Err(err) => {
                    assert!(err.is_instance_of::<PyTypeError>(py));
                    assert!(err.value(py).to_string().contains("Error from __hash__"));
                }
            }
        })
    }

    #[test]
    fn test_set_item() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let dict = v.into_pyobject(py).unwrap();
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
        Python::attach(|py| {
            let cnt;
            let obj = py.eval(ffi::c_str!("object()"), None, None).unwrap();
            {
                cnt = obj.get_refcnt();
                let _dict = [(10, &obj)].into_py_dict(py);
            }
            {
                assert_eq!(cnt, obj.get_refcnt());
            }
        });
    }

    #[test]
    fn test_set_item_does_not_update_original_object() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let dict = (&v).into_pyobject(py).unwrap();
            assert!(dict.set_item(7i32, 42i32).is_ok()); // change
            assert!(dict.set_item(8i32, 123i32).is_ok()); // insert
            assert_eq!(32i32, v[&7i32]); // not updated!
            assert_eq!(None, v.get(&8i32));
        });
    }

    #[test]
    fn test_del_item() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let dict = v.into_pyobject(py).unwrap();
            assert!(dict.del_item(7i32).is_ok());
            assert_eq!(0, dict.len());
            assert!(dict.get_item(7i32).unwrap().is_none());
        });
    }

    #[test]
    fn test_del_item_does_not_update_original_object() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let dict = (&v).into_pyobject(py).unwrap();
            assert!(dict.del_item(7i32).is_ok()); // change
            assert_eq!(32i32, *v.get(&7i32).unwrap()); // not updated!
        });
    }

    #[test]
    fn test_items() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let dict = v.into_pyobject(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            let mut value_sum = 0;
            for el in dict.items() {
                let tuple = el.cast::<PyTuple>().unwrap();
                key_sum += tuple.get_item(0).unwrap().extract::<i32>().unwrap();
                value_sum += tuple.get_item(1).unwrap().extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_keys() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let dict = v.into_pyobject(py).unwrap();
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
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let dict = v.into_pyobject(py).unwrap();
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
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let dict = v.into_pyobject(py).unwrap();
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
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let dict = v.into_pyobject(py).unwrap();
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
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);

            let dict = (&v).into_pyobject(py).unwrap();

            for (key, value) in &dict {
                dict.set_item(key, value.extract::<i32>().unwrap() + 7)
                    .unwrap();
            }
        });
    }

    #[test]
    #[should_panic]
    fn test_iter_key_mutated() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            for i in 0..10 {
                v.insert(i * 2, i * 2);
            }
            let dict = v.into_pyobject(py).unwrap();

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
        Python::attach(|py| {
            let mut v = HashMap::new();
            for i in 0..10 {
                v.insert(i * 2, i * 2);
            }
            let dict = v.into_pyobject(py).unwrap();

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
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let dict = (&v).into_pyobject(py).unwrap();

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
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let dict = v.into_pyobject(py).unwrap();
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
        Python::attach(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict(py).unwrap();

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
        Python::attach(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict(py).unwrap();

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
        Python::attach(|py| {
            let vec = vec![("a", 1), ("b", 2), ("c", 3)];
            let py_map = vec.into_py_dict(py).unwrap();

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
        Python::attach(|py| {
            let arr = [("a", 1), ("b", 2), ("c", 3)];
            let py_map = arr.into_py_dict(py).unwrap();

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
        Python::attach(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict(py).unwrap();

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
        Python::attach(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict(py).unwrap();

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
        map.into_py_dict(py).unwrap()
    }

    #[test]
    #[cfg(not(any(PyPy, GraalPy)))]
    fn dict_keys_view() {
        Python::attach(|py| {
            let dict = abc_dict(py);
            let keys = dict.call_method0("keys").unwrap();
            assert!(keys.is_instance(&py.get_type::<PyDictKeys>()).unwrap());
        })
    }

    #[test]
    #[cfg(not(any(PyPy, GraalPy)))]
    fn dict_values_view() {
        Python::attach(|py| {
            let dict = abc_dict(py);
            let values = dict.call_method0("values").unwrap();
            assert!(values.is_instance(&py.get_type::<PyDictValues>()).unwrap());
        })
    }

    #[test]
    #[cfg(not(any(PyPy, GraalPy)))]
    fn dict_items_view() {
        Python::attach(|py| {
            let dict = abc_dict(py);
            let items = dict.call_method0("items").unwrap();
            assert!(items.is_instance(&py.get_type::<PyDictItems>()).unwrap());
        })
    }

    #[test]
    fn dict_update() {
        Python::attach(|py| {
            let dict = [("a", 1), ("b", 2), ("c", 3)].into_py_dict(py).unwrap();
            let other = [("b", 4), ("c", 5), ("d", 6)].into_py_dict(py).unwrap();
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
        Python::attach(|py| {
            let dict = [("a", 1), ("b", 2), ("c", 3)].into_py_dict(py).unwrap();
            let other = [("b", 4), ("c", 5), ("d", 6)].into_py_dict(py).unwrap();
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

    #[test]
    fn test_iter_all() {
        Python::attach(|py| {
            let dict = [(1, true), (2, true), (3, true)].into_py_dict(py).unwrap();
            assert!(dict.iter().all(|(_, v)| v.extract::<bool>().unwrap()));

            let dict = [(1, true), (2, false), (3, true)].into_py_dict(py).unwrap();
            assert!(!dict.iter().all(|(_, v)| v.extract::<bool>().unwrap()));
        });
    }

    #[test]
    fn test_iter_any() {
        Python::attach(|py| {
            let dict = [(1, true), (2, false), (3, false)]
                .into_py_dict(py)
                .unwrap();
            assert!(dict.iter().any(|(_, v)| v.extract::<bool>().unwrap()));

            let dict = [(1, false), (2, false), (3, false)]
                .into_py_dict(py)
                .unwrap();
            assert!(!dict.iter().any(|(_, v)| v.extract::<bool>().unwrap()));
        });
    }

    #[test]
    #[allow(clippy::search_is_some)]
    fn test_iter_find() {
        Python::attach(|py| {
            let dict = [(1, false), (2, true), (3, false)]
                .into_py_dict(py)
                .unwrap();

            assert_eq!(
                Some((2, true)),
                dict.iter()
                    .find(|(_, v)| v.extract::<bool>().unwrap())
                    .map(|(k, v)| (k.extract().unwrap(), v.extract().unwrap()))
            );

            let dict = [(1, false), (2, false), (3, false)]
                .into_py_dict(py)
                .unwrap();

            assert!(dict
                .iter()
                .find(|(_, v)| v.extract::<bool>().unwrap())
                .is_none());
        });
    }

    #[test]
    #[allow(clippy::search_is_some)]
    fn test_iter_position() {
        Python::attach(|py| {
            let dict = [(1, false), (2, false), (3, true)]
                .into_py_dict(py)
                .unwrap();
            assert_eq!(
                Some(2),
                dict.iter().position(|(_, v)| v.extract::<bool>().unwrap())
            );

            let dict = [(1, false), (2, false), (3, false)]
                .into_py_dict(py)
                .unwrap();
            assert!(dict
                .iter()
                .position(|(_, v)| v.extract::<bool>().unwrap())
                .is_none());
        });
    }

    #[test]
    fn test_iter_fold() {
        Python::attach(|py| {
            let dict = [(1, 1), (2, 2), (3, 3)].into_py_dict(py).unwrap();
            let sum = dict
                .iter()
                .fold(0, |acc, (_, v)| acc + v.extract::<i32>().unwrap());
            assert_eq!(sum, 6);
        });
    }

    #[test]
    fn test_iter_try_fold() {
        Python::attach(|py| {
            let dict = [(1, 1), (2, 2), (3, 3)].into_py_dict(py).unwrap();
            let sum = dict
                .iter()
                .try_fold(0, |acc, (_, v)| PyResult::Ok(acc + v.extract::<i32>()?))
                .unwrap();
            assert_eq!(sum, 6);

            let dict = [(1, "foo"), (2, "bar")].into_py_dict(py).unwrap();
            assert!(dict
                .iter()
                .try_fold(0, |acc, (_, v)| PyResult::Ok(acc + v.extract::<i32>()?))
                .is_err());
        });
    }

    #[test]
    fn test_iter_count() {
        Python::attach(|py| {
            let dict = [(1, 1), (2, 2), (3, 3)].into_py_dict(py).unwrap();
            assert_eq!(dict.iter().count(), 3);
        })
    }
}
