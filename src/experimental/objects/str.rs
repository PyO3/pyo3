// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::types::Str;
use crate::{
    ffi,
    objects::{FromPyObject, PyAny, PyBytes, PyNativeObject},
    AsPyPointer, IntoPy, PyErr, PyObject, PyResult, Python, ToPyObject,
};
use std::borrow::Cow;
use std::os::raw::c_char;
use std::str;

/// Represents a Python `str` (a Unicode string object).
///
/// This type is immutable.
#[repr(transparent)]
pub struct PyStr<'py>(pub(crate) PyAny<'py>);

pyo3_native_object!(PyStr<'py>, Str, 'py);

impl<'py> PyStr<'py> {
    /// Creates a new Python string object.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python<'py>, s: &str) -> Self {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            Self(PyAny::from_raw_or_panic(
                py,
                ffi::PyUnicode_FromStringAndSize(ptr, len),
            ))
        }
    }

    pub fn from_object(src: &PyAny<'py>, encoding: &str, errors: &str) -> Self {
        unsafe {
            Self(PyAny::from_raw_or_panic(
                src.py(),
                ffi::PyUnicode_FromEncodedObject(
                    src.as_ptr(),
                    encoding.as_ptr() as *const c_char,
                    errors.as_ptr() as *const c_char,
                ),
            ))
        }
    }

    /// Gets the Python string as a byte slice.
    ///
    /// Returns a `UnicodeEncodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    #[inline]
    pub fn to_str(&self) -> PyResult<&str> {
        #[cfg(not(Py_LIMITED_API))]
        unsafe {
            let mut size: ffi::Py_ssize_t = 0;
            let data = ffi::PyUnicode_AsUTF8AndSize(self.as_ptr(), &mut size) as *const u8;
            if data.is_null() {
                Err(PyErr::fetch(self.py()))
            } else {
                let slice = std::slice::from_raw_parts(data, size as usize);
                Ok(std::str::from_utf8_unchecked(slice))
            }
        }
        #[cfg(Py_LIMITED_API)]
        unsafe {
            let data = ffi::PyUnicode_AsUTF8String(self.as_ptr());
            if data.is_null() {
                Err(PyErr::fetch(self.py()))
            } else {
                let bytes = self.py().from_owned_ptr::<PyBytes>(data);
                Ok(std::str::from_utf8_unchecked(bytes.as_bytes()))
            }
        }
    }

    /// Converts the `PyStr` into a Rust string.
    ///
    /// Unpaired surrogates invalid UTF-8 sequences are
    /// replaced with `U+FFFD REPLACEMENT CHARACTER`.
    pub fn to_string_lossy(&self) -> Cow<str> {
        match self.to_str() {
            Ok(s) => Cow::Borrowed(s),
            Err(_) => {
                let bytes: PyBytes<'py> = unsafe {
                    PyBytes(PyAny::from_raw_or_panic(
                        self.py(),
                        ffi::PyUnicode_AsEncodedString(
                            self.as_ptr(),
                            b"utf-8\0" as *const _ as _,
                            b"surrogatepass\0" as *const _ as _,
                        ),
                    ))
                };
                Cow::Owned(String::from_utf8_lossy(bytes.as_bytes()).to_string())
            }
        }
    }
}

/// Converts a Rust `str` to a Python object.
/// See `PyStr::new` for details on the conversion.
impl ToPyObject for str {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyStr::new(py, self).into()
    }
}

impl<'a> IntoPy<PyObject> for &'a str {
    #[inline]
    fn into_py(self, py: Python) -> PyObject {
        PyStr::new(py, self).into()
    }
}

/// Converts a Rust `Cow<str>` to a Python object.
/// See `PyStr::new` for details on the conversion.
impl<'a> ToPyObject for Cow<'a, str> {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyStr::new(py, self).into()
    }
}

/// Converts a Rust `String` to a Python object.
/// See `PyStr::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyStr::new(py, self).into()
    }
}

impl ToPyObject for char {
    fn to_object(&self, py: Python) -> PyObject {
        self.into_py(py)
    }
}

impl IntoPy<PyObject> for char {
    fn into_py(self, py: Python) -> PyObject {
        let mut bytes = [0u8; 4];
        PyStr::new(py, self.encode_utf8(&mut bytes)).into()
    }
}

impl IntoPy<PyObject> for String {
    fn into_py(self, py: Python) -> PyObject {
        PyStr::new(py, &self).into()
    }
}

impl<'a> IntoPy<PyObject> for &'a String {
    #[inline]
    fn into_py(self, py: Python) -> PyObject {
        PyStr::new(py, self).into()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` objects.
impl<'a> FromPyObject<'a, '_> for &'a str {
    fn extract(ob: &'a PyAny<'_>) -> PyResult<Self> {
        ob.downcast::<PyStr>()?.to_str()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` objects.
impl FromPyObject<'_, '_> for String {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        obj.downcast::<PyStr>()?.to_str().map(ToOwned::to_owned)
    }
}

impl FromPyObject<'_, '_> for char {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        let s = obj.downcast::<PyStr>()?.to_str()?;
        let mut iter = s.chars();
        if let (Some(ch), None) = (iter.next(), iter.next()) {
            Ok(ch)
        } else {
            Err(crate::exceptions::PyValueError::new_err(
                "expected a string of length 1",
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::experimental::ToPyObject;

    #[test]
    fn test_non_bmp() {
        Python::with_gil(|py| {
            let s = "\u{1F30F}";
            let py_str = s.to_object(py);
            assert_eq!(s, py_str.extract::<String>().unwrap());
        })
    }

    #[test]
    fn test_extract_str() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_str = s.to_object(py);
            let s2: &str = py_str.extract().unwrap();
            assert_eq!(s, s2);
        })
    }

    #[test]
    fn test_extract_char() {
        Python::with_gil(|py| {
            let ch = '😃';
            let py_str = ch.to_object(py);
            let ch2: char = FromPyObject::extract(&py_str).unwrap();
            assert_eq!(ch, ch2);
        })
    }

    #[test]
    fn test_extract_char_err() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_str = s.to_object(py);
            let err: crate::PyResult<char> = FromPyObject::extract(&py_str);
            assert!(err
                .unwrap_err()
                .to_string()
                .contains("expected a string of length 1"));
        })
    }

    #[test]
    fn test_to_str_ascii() {
        Python::with_gil(|py| {
            let s = "ascii 🐈";
            let py_str = PyStr::new(py, s);
            assert_eq!(s, py_str.to_str().unwrap());
        })
    }

    #[test]
    fn test_to_str_surrogate() {
        Python::with_gil(|py| {
            let obj: PyAny = py.eval(r#"'\ud800'"#, None, None).unwrap().to_owned();
            let py_str = obj.downcast::<PyStr>().unwrap();
            assert!(py_str.to_str().is_err());
        })
    }

    #[test]
    fn test_to_str_unicode() {
        Python::with_gil(|py| {
            let s = "哈哈🐈";
            let py_str = PyStr::new(py, s);
            assert_eq!(s, py_str.to_str().unwrap());
        })
    }

    #[test]
    fn test_to_string_lossy() {
        Python::with_gil(|py| {
            let obj: PyObject = py
                .eval(r#"'🐈 Hello \ud800World'"#, None, None)
                .unwrap()
                .into();
            let py_str = obj.as_object::<PyAny>(py).downcast::<PyStr>().unwrap();
            assert_eq!(py_str.to_string_lossy(), "🐈 Hello ���World");
        })
    }

    #[test]
    fn test_debug_string() {
        Python::with_gil(|py| {
            let v = "Hello\n".to_object(py);
            let s = v.downcast::<PyStr>().unwrap();
            assert_eq!(format!("{:?}", s), "'Hello\\n'");
        })
    }

    #[test]
    fn test_display_string() {
        Python::with_gil(|py| {
            let v = "Hello\n".to_object(py);
            let s = v.downcast::<PyStr>().unwrap();
            assert_eq!(format!("{}", s), "Hello\n");
        })
    }
}
