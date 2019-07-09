// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::conversion::FromPyObject;
use crate::conversion::{IntoPyObject, PyTryFrom, ToPyObject};
use crate::err::{PyErr, PyResult};
use crate::exceptions;
use crate::ffi;
use crate::instance::PyNativeType;
use crate::object::PyObject;
use crate::types::PyAny;
use crate::AsPyPointer;
use crate::Python;
use std::borrow::Cow;
use std::os::raw::c_char;
use std::str;

/// Represents a Python `string`.
#[repr(transparent)]
pub struct PyString(PyObject);

pyobject_native_type!(PyString, ffi::PyUnicode_Type, ffi::PyUnicode_Check);

/// Represents a Python `byte` string.
#[repr(transparent)]
pub struct PyBytes(PyObject);

pyobject_native_type!(
    PyBytes,
    ffi::PyBytes_Type,
    Some("builtins"),
    ffi::PyBytes_Check
);

impl PyString {
    /// Creates a new Python string object.
    ///
    /// Panics if out of memory.
    pub fn new<'p>(py: Python<'p>, s: &str) -> &'p PyString {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe { py.from_owned_ptr(ffi::PyUnicode_FromStringAndSize(ptr, len)) }
    }

    pub fn from_object<'p>(src: &'p PyAny, encoding: &str, errors: &str) -> PyResult<&'p PyString> {
        unsafe {
            src.py()
                .from_owned_ptr_or_err::<PyString>(ffi::PyUnicode_FromEncodedObject(
                    src.as_ptr(),
                    encoding.as_ptr() as *const c_char,
                    errors.as_ptr() as *const c_char,
                ))
        }
    }

    /// Get the Python string as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let mut size: ffi::Py_ssize_t = 0;
            let data = ffi::PyUnicode_AsUTF8AndSize(self.0.as_ptr(), &mut size) as *const u8;
            // PyUnicode_AsUTF8AndSize would return null if the pointer did not reference a valid
            // unicode object, but because we have a valid PyString, assume success
            debug_assert!(!data.is_null());
            std::slice::from_raw_parts(data, size as usize)
        }
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// Returns a `UnicodeDecodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    pub fn to_string(&self) -> PyResult<Cow<str>> {
        match std::str::from_utf8(self.as_bytes()) {
            Ok(s) => Ok(Cow::Borrowed(s)),
            Err(e) => Err(PyErr::from_instance(
                exceptions::UnicodeDecodeError::new_utf8(self.py(), self.as_bytes(), e)?,
            )),
        }
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// Unpaired surrogates invalid UTF-8 sequences are
    /// replaced with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.as_bytes())
    }
}

impl PyBytes {
    /// Creates a new Python byte string object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new<'p>(py: Python<'p>, s: &[u8]) -> &'p PyBytes {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe { py.from_owned_ptr(ffi::PyBytes_FromStringAndSize(ptr, len)) }
    }

    /// Creates a new Python byte string object from raw pointer.
    ///
    /// Panics if out of memory.
    pub unsafe fn from_ptr(py: Python<'_>, ptr: *const u8, len: usize) -> &PyBytes {
        py.from_owned_ptr(ffi::PyBytes_FromStringAndSize(
            ptr as *const _,
            len as isize,
        ))
    }

    /// Get the Python string as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let buffer = ffi::PyBytes_AsString(self.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(self.as_ptr()) as usize;
            debug_assert!(!buffer.is_null());
            std::slice::from_raw_parts(buffer, length)
        }
    }
}

/// Converts Rust `str` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for str {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

impl<'a> IntoPyObject for &'a str {
    #[inline]
    fn into_object(self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

/// Converts Rust `Cow<str>` to Python object.
/// See `PyString::new` for details on the conversion.
impl<'a> ToPyObject for Cow<'a, str> {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

/// Converts Rust `String` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

impl IntoPyObject for String {
    #[inline]
    fn into_object(self, py: Python) -> PyObject {
        PyString::new(py, &self).into()
    }
}

impl<'a> IntoPyObject for &'a String {
    #[inline]
    fn into_object(self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'source> crate::FromPyObject<'source> for Cow<'source, str> {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        <PyString as PyTryFrom>::try_from(ob)?.to_string()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'a> crate::FromPyObject<'a> for &'a str {
    fn extract(ob: &'a PyAny) -> PyResult<Self> {
        let s: Cow<'a, str> = crate::FromPyObject::extract(ob)?;
        match s {
            Cow::Borrowed(r) => Ok(r),
            Cow::Owned(r) => {
                let r = ob.py().register_any(r);
                Ok(r.as_str())
            }
        }
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'source> FromPyObject<'source> for String {
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        <PyString as PyTryFrom>::try_from(obj)?
            .to_string()
            .map(Cow::into_owned)
    }
}

#[cfg(test)]
mod test {
    use super::PyString;
    use crate::instance::AsPyRef;
    use crate::object::PyObject;
    use crate::Python;
    use crate::{FromPyObject, PyTryFrom, ToPyObject};
    use std::borrow::Cow;

    #[test]
    fn test_non_bmp() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "\u{1F30F}";
        let py_string = s.to_object(py);
        assert_eq!(s, py_string.extract::<String>(py).unwrap());
    }

    #[test]
    fn test_extract_str() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "Hello Python";
        let py_string = s.to_object(py);

        let s2: &str = FromPyObject::extract(py_string.as_ref(py).into()).unwrap();
        assert_eq!(s, s2);
    }

    #[test]
    fn test_as_bytes() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "ascii 🐈";
        let obj: PyObject = PyString::new(py, s).into();
        let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
        assert_eq!(s.as_bytes(), py_string.as_bytes());
    }

    #[test]
    fn test_to_string_ascii() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "ascii";
        let obj: PyObject = PyString::new(py, s).into();
        let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
        assert!(py_string.to_string().is_ok());
        assert_eq!(Cow::Borrowed(s), py_string.to_string().unwrap());
    }

    #[test]
    fn test_to_string_unicode() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "哈哈🐈";
        let obj: PyObject = PyString::new(py, s).into();
        let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
        assert!(py_string.to_string().is_ok());
        assert_eq!(Cow::Borrowed(s), py_string.to_string().unwrap());
    }
}
