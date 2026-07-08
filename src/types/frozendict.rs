use crate::conversion;
use crate::err::{self, PyErr, PyResult};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Bound;
use crate::py_result_ext::PyResultExt;
use crate::types::{PyAny, PyList, PyMapping};
use crate::{ffi, Borrowed, BoundObject, IntoPyObject, IntoPyObjectExt, Python};
#[cfg(any(RustPython, Py_LIMITED_API))]
use crate::{
    sync::PyOnceLock,
    types::{PyType, PyTypeMethods},
    Py,
};
use core::ptr;
#[cfg(Py_LIMITED_API)]
use std::ffi::c_int;

/// Represents a Python `frozendict`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFrozenDict>`][crate::Py] or [`Bound<'py, PyFrozenDict>`][Bound].
///
/// For APIs available on `frozendict` objects, see the [`PyFrozenDictMethods`] trait which is implemented for
/// [`Bound<'py, PyFrozenDict>`][Bound].
///
/// This type is only available on Python 3.15+.
#[repr(transparent)]
pub struct PyFrozenDict(PyAny);

#[cfg(all(Py_3_15, not(any(GraalPy, PyPy, RustPython, Py_LIMITED_API))))]
pyobject_native_type_core!(
    PyFrozenDict,
    pyobject_native_static_type_object!(ffi::PyFrozenDict_Type),
    "builtins",
    "frozendict",
    #checkfunction=ffi::PyFrozenDict_Check
);

#[cfg(all(Py_3_15, Py_LIMITED_API))]
fn PyFrozenDict_Check(ptr: *mut ffi::PyObject) -> c_int {
    unsafe { ffi::PyObject_TypeCheck(ptr, PyFrozenDict::as_type_ptr()) }
}

#[cfg(all(Py_3_15, Py_LIMITED_API))]
pyobject_native_type_core!(
    PyFrozenDict,
    |py| {
        static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        TYPE.import(py, "builtins", "frozendict").unwrap().as_type_ptr()
    },
    "builtins",
    "frozendict",
    #checkfunction=PyFrozenDict_Check
);

impl PyFrozenDict {
    /// Creates a new frozendict from an iterable of key-value pairs.
    ///
    /// The iterable can be any Python object that yields (key, value) pairs,
    /// such as another dict, a list of tuples, or any mapping-like object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::types::PyFrozenDict;
    /// # #[cfg(Py_3_15)]
    /// # fn example() -> PyResult<()> {
    /// # Python::try_attach(|py| -> PyResult<()> {
    ///     let fd = PyFrozenDict::new(py, vec![("a", 1), ("b", 2)])?;
    ///     assert_eq!(fd.len(), 2);
    /// #     Ok(())
    /// # })
    /// # }
    /// ```
    pub fn new<'py, T>(py: Python<'py>, iterable: T) -> PyResult<Bound<'py, PyFrozenDict>>
    where
        T: IntoPyObject<'py>,
        err::PyErr: core::convert::From<<T as conversion::IntoPyObject<'py>>::Error>,
    {
        let obj = iterable.into_pyobject(py)?;
        #[cfg(Py_LIMITED_API)]
        {
            PyFrozenDict::type_object(py)
                .call1((obj,))
                .map(|obj| unsafe { obj.cast_into_unchecked() })
        }
        #[cfg(not(Py_LIMITED_API))]
        {
            unsafe {
                ffi::PyFrozenDict_New(obj.as_ptr())
                    .assume_owned_or_err(py)
                    .cast_into_unchecked()
            }
        }
    }

    /// Creates a new empty frozendict.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::types::PyFrozenDict;
    /// # #[cfg(Py_3_15)]
    /// # fn example() -> PyResult<()> {
    /// # Python::try_attach(|py| -> PyResult<()> {
    ///     let fd = PyFrozenDict::empty(py)?;
    ///     assert!(fd.is_empty());
    /// #     Ok(())
    /// # })
    /// # }
    /// ```
    pub fn empty(py: Python<'_>) -> PyResult<Bound<'_, PyFrozenDict>> {
        #[cfg(Py_LIMITED_API)]
        {
            PyFrozenDict::type_object_raw(py)
                .call0()
                .map(|obj| unsafe { obj.cast_into_unchecked() })
        }
        #[cfg(not(Py_LIMITED_API))]
        unsafe {
            ffi::PyFrozenDict_New(ptr::null_mut())
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }
}

/// Implementation of functionality for [`PyFrozenDict`].
///
/// These methods are defined for the `Bound<'py, PyFrozenDict>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyFrozenDict")]
pub trait PyFrozenDictMethods<'py>: crate::sealed::Sealed {
    /// Return the number of items in the frozendict.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    fn len(&self) -> usize;

    /// Checks if the frozendict is empty, i.e. `len(self) == 0`.
    fn is_empty(&self) -> bool;

    /// Determines if the frozendict contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: IntoPyObject<'py>;

    /// Gets an item from the frozendict.
    ///
    /// Returns `None` if the item is not present, or if an error occurs.
    ///
    /// To get a `KeyError` for non-existing keys, use `PyAny::get_item`.
    fn get_item<K>(&self, key: K) -> PyResult<Option<Bound<'py, PyAny>>>
    where
        K: IntoPyObject<'py>;

    /// Returns a list of all keys in the frozendict.
    ///
    /// This is equivalent to the Python expression `list(self.keys())`.
    fn keys(&self) -> Bound<'py, PyList>;

    /// Returns a list of all values in the frozendict.
    ///
    /// This is equivalent to the Python expression `list(self.values())`.
    fn values(&self) -> Bound<'py, PyList>;

    /// Returns a list of all (key, value) tuples in the frozendict.
    ///
    /// This is equivalent to the Python expression `list(self.items())`.
    fn items(&self) -> Bound<'py, PyList>;

    /// Returns an iterator of `(key, value)` tuples in this frozendict.
    ///
    /// Since `frozendict` objects are immutable, iteration does not need the
    /// mutation guards that are required for [`PyDict`].
    fn iter(&self) -> BoundFrozenDictIterator<'py>;

    /// Returns `self` cast as a `PyMapping`.
    ///
    /// This is a zero-cost conversion that allows using the frozendict
    /// with methods that accept a mapping protocol object.
    fn as_mapping(&self) -> &Bound<'py, PyMapping>;

    /// Returns `self` cast as a `PyMapping`.
    ///
    /// This is a zero-cost conversion that allows using the frozendict
    /// with methods that accept a mapping protocol object.
    fn into_mapping(self) -> Bound<'py, PyMapping>;
}

impl<'py> PyFrozenDictMethods<'py> for Bound<'py, PyFrozenDict> {
    #[inline]
    fn len(&self) -> usize {
        unsafe { ffi::PyDict_Size(self.as_ptr()) as usize }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: IntoPyObject<'py>,
    {
        fn inner(fd: &Bound<'_, PyFrozenDict>, key: Borrowed<'_, '_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PyDict_Contains(fd.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(fd.py())),
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
            fd: &Bound<'py, PyFrozenDict>,
            key: Borrowed<'_, '_, PyAny>,
        ) -> PyResult<Option<Bound<'py, PyAny>>> {
            let py = fd.py();
            let mut result: *mut ffi::PyObject = core::ptr::null_mut();
            match unsafe { ffi::compat::PyDict_GetItemRef(fd.as_ptr(), key.as_ptr(), &mut result) }
            {
                core::ffi::c_int::MIN..=-1 => Err(PyErr::fetch(py)),
                0 => Ok(None),
                1..=core::ffi::c_int::MAX => {
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

    fn iter(&self) -> BoundFrozenDictIterator<'py> {
        BoundFrozenDictIterator::new(self.clone())
    }

    fn as_mapping(&self) -> &Bound<'py, PyMapping> {
        unsafe { self.cast_unchecked() }
    }

    fn into_mapping(self) -> Bound<'py, PyMapping> {
        unsafe { self.cast_into_unchecked() }
    }
}

/// An iterator over the items in a frozendict.
///
/// Created by the `iter()` method on `Bound<'py, PyFrozenDict>`.
///
/// Because the underlying mapping cannot be mutated, this iterator simply
/// walks the current contents as a stable snapshot.
pub struct BoundFrozenDictIterator<'py> {
    fd: Bound<'py, PyFrozenDict>,
    ppos: isize,
    remaining: usize,
}

impl<'py> BoundFrozenDictIterator<'py> {
    fn new(fd: Bound<'py, PyFrozenDict>) -> Self {
        let remaining = fd.len();
        BoundFrozenDictIterator {
            fd,
            ppos: 0,
            remaining,
        }
    }
}

impl<'py> Iterator for BoundFrozenDictIterator<'py> {
    type Item = (Bound<'py, PyAny>, Bound<'py, PyAny>);

    fn next(&mut self) -> Option<Self::Item> {
        let ppos: *mut ffi::Py_ssize_t = &mut self.ppos;
        let mut key: *mut ffi::PyObject = core::ptr::null_mut();
        let mut value: *mut ffi::PyObject = core::ptr::null_mut();

        if unsafe { ffi::PyDict_Next(self.fd.as_ptr(), ppos, &mut key, &mut value) != 0 } {
            let py = self.fd.py();
            self.remaining -= 1;
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

impl ExactSizeIterator for BoundFrozenDictIterator<'_> {
    fn len(&self) -> usize {
        self.remaining
    }
}

impl<'py> IntoIterator for Bound<'py, PyFrozenDict> {
    type Item = (Bound<'py, PyAny>, Bound<'py, PyAny>);
    type IntoIter = BoundFrozenDictIterator<'py>;

    /// Returns an iterator over the `(key, value)` pairs in this frozendict.
    fn into_iter(self) -> Self::IntoIter {
        BoundFrozenDictIterator::new(self)
    }
}

impl<'py> IntoIterator for &Bound<'py, PyFrozenDict> {
    type Item = (Bound<'py, PyAny>, Bound<'py, PyAny>);
    type IntoIter = BoundFrozenDictIterator<'py>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(all(Py_3_15, test))]
mod tests {
    use super::*;
    use crate::types::{list::PyListMethods, mapping::PyMappingMethods, PyAnyMethods};

    use std::string::{String, ToString};
    use std::vec::Vec;

    #[test]
    fn test_frozendict_new() {
        Python::attach(|py| {
            let fd = PyFrozenDict::new(py, vec![("a", 1), ("b", 2)]).unwrap();
            assert_eq!(fd.len(), 2);
        })
    }

    #[test]
    fn test_frozendict_empty() {
        Python::attach(|py| {
            let fd = PyFrozenDict::empty(py).unwrap();
            assert!(fd.is_empty());
            assert_eq!(fd.len(), 0);
        })
    }

    #[test]
    fn test_frozendict_contains() {
        Python::attach(|py| {
            let fd = PyFrozenDict::new(py, vec![("a", 1), ("b", 2)]).unwrap();
            assert!(fd.contains("a").unwrap());
            assert!(!fd.contains("c").unwrap());
        })
    }

    #[test]
    fn test_frozendict_get_item() {
        Python::attach(|py| {
            let fd = PyFrozenDict::new(py, vec![("a", 1), ("b", 2)]).unwrap();
            let val = fd.get_item("a").unwrap();
            assert!(val.is_some_and(|v| v.extract::<i32>().unwrap() == 1));
        })
    }

    #[test]
    fn test_frozendict_keys() {
        Python::attach(|py| {
            let fd = PyFrozenDict::new(py, vec![("a", 1), ("b", 2)]).unwrap();
            let keys = fd.keys();
            assert_eq!(keys.len(), 2);
            assert!(keys.contains("a").unwrap());
            assert!(keys.contains("b").unwrap());
        })
    }

    #[test]
    fn test_frozendict_values() {
        Python::attach(|py| {
            let fd = PyFrozenDict::new(py, vec![("a", 1), ("b", 2)]).unwrap();
            let values = fd.values();
            assert_eq!(values.len(), 2);
            assert!(values.contains(1).unwrap());
            assert!(values.contains(2).unwrap());
        })
    }

    #[test]
    fn test_frozendict_items() {
        Python::attach(|py| {
            let fd = PyFrozenDict::new(py, vec![("a", 1), ("b", 2)]).unwrap();
            let items = fd.items();
            assert_eq!(items.len(), 2);
            assert!(items.contains(&("a", 1)).unwrap());
            assert!(items.contains(&("b", 2)).unwrap());
        })
    }

    #[test]
    fn test_frozendict_iter() {
        Python::attach(|py| {
            let fd = PyFrozenDict::new(py, vec![("a", 1), ("b", 2)]).unwrap();
            let mut count = 0;
            for ((k, v), (expected_k, expected_v)) in fd.iter().zip([("a", 1), ("b", 2)].iter()) {
                count += 1;
                assert_eq!(
                    (k.extract::<String>().unwrap(), v.extract::<i32>().unwrap()),
                    (expected_k.to_string(), *expected_v)
                );
            }
            assert_eq!(count, 2);
        })
    }

    #[test]
    fn test_frozendict_iter_size_hint() {
        Python::attach(|py| {
            let fd = PyFrozenDict::new(py, vec![("a", 1), ("b", 2)]).unwrap();

            let mut iter = fd.iter();
            assert_eq!(iter.size_hint(), (2, Some(2)));
            iter.next();
            assert_eq!(iter.size_hint(), (1, Some(1)));

            for _ in &mut iter {}
            assert_eq!(iter.size_hint(), (0, Some(0)));
            assert!(iter.next().is_none());
        })
    }

    #[test]
    fn test_frozendict_into_iter() {
        Python::attach(|py| {
            let fd = PyFrozenDict::new(py, vec![("a", 1), ("b", 2)]).unwrap();
            let mut items = Vec::new();

            for (key, value) in fd {
                items.push((
                    key.extract::<String>().unwrap(),
                    value.extract::<i32>().unwrap(),
                ));
            }

            assert_eq!(items.len(), 2);
            assert!(items.contains(&("a".to_string(), 1)));
            assert!(items.contains(&("b".to_string(), 2)));
        })
    }

    #[test]
    fn test_frozendict_as_mapping() {
        Python::attach(|py| {
            let fd = PyFrozenDict::new(py, vec![("a", 1)]).unwrap();
            let mapping = fd.as_mapping();
            assert!(PyMappingMethods::len(mapping).unwrap() == 1);
        })
    }

    #[test]
    fn test_frozendict_hash() {
        Python::attach(|py| {
            let fd = PyFrozenDict::new(py, vec![("a", 1)]).unwrap();
            let h = fd.hash().unwrap();
            assert!(h != 0);
        })
    }
}
