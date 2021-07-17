// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::types::PyBytes;
use crate::{
    ffi, AsPyPointer, FromPyObject, IntoPy, PyAny, PyNativeType, PyObject, PyResult, PyTryFrom,
    Python, ToPyObject,
};
use std::borrow::Cow;
use std::os::raw::c_char;
use std::str;

/// Represents a Python `string` (a Unicode string object).
///
/// This type is immutable.
#[repr(transparent)]
pub struct PyString(PyAny);

pyobject_native_type_core!(PyString, ffi::PyUnicode_Type, #checkfunction=ffi::PyUnicode_Check);

impl PyString {
    /// Creates a new Python string object.
    ///
    /// Panics if out of memory.
    pub fn new<'p>(py: Python<'p>, s: &str) -> &'p PyString {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe { py.from_owned_ptr(ffi::PyUnicode_FromStringAndSize(ptr, len)) }
    }

    /// Attempts to create a Python string from a Python [bytes-like object].
    ///
    /// [bytes-like object]: (https://docs.python.org/3/glossary.html#term-bytes-like-object).
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

    /// Gets the Python string as a byte slice.
    ///
    /// Returns a `UnicodeEncodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    #[inline]
    pub fn to_str(&self) -> PyResult<&str> {
        let utf8_slice = {
            cfg_if::cfg_if! {
                if #[cfg(any(not(Py_LIMITED_API), Py_3_10))] {
                    // PyUnicode_AsUTF8AndSize only available on limited API from Python 3.10 and up.
                    let mut size: ffi::Py_ssize_t = 0;
                    let data = unsafe { ffi::PyUnicode_AsUTF8AndSize(self.as_ptr(), &mut size) };
                    if data.is_null() {
                        return Err(crate::PyErr::api_call_failed(self.py()));
                    } else {
                        unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) }
                    }
                } else {
                    let bytes = unsafe {
                        self.py().from_owned_ptr_or_err::<PyBytes>(ffi::PyUnicode_AsUTF8String(self.as_ptr()))?
                    };
                    bytes.as_bytes()
                }
            }
        };
        Ok(unsafe { std::str::from_utf8_unchecked(utf8_slice) })
    }

    /// Converts the `PyString` into a Rust string.
    ///
    /// Unpaired surrogates invalid UTF-8 sequences are
    /// replaced with `U+FFFD REPLACEMENT CHARACTER`.
    pub fn to_string_lossy(&self) -> Cow<str> {
        match self.to_str() {
            Ok(s) => Cow::Borrowed(s),
            Err(_) => {
                let bytes = unsafe {
                    self.py()
                        .from_owned_ptr::<PyBytes>(ffi::PyUnicode_AsEncodedString(
                            self.as_ptr(),
                            b"utf-8\0" as *const _ as _,
                            b"surrogatepass\0" as *const _ as _,
                        ))
                };
                String::from_utf8_lossy(bytes.as_bytes())
            }
        }
    }
}

/// Converts a Rust `str` to a Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for str {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

impl<'a> IntoPy<PyObject> for &'a str {
    #[inline]
    fn into_py(self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

/// Converts a Rust `Cow<str>` to a Python object.
/// See `PyString::new` for details on the conversion.
impl<'a> ToPyObject for Cow<'a, str> {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

impl IntoPy<PyObject> for Cow<'_, str> {
    #[inline]
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

/// Converts a Rust `String` to a Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
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
        PyString::new(py, self.encode_utf8(&mut bytes)).into()
    }
}

impl IntoPy<PyObject> for String {
    fn into_py(self, py: Python) -> PyObject {
        PyString::new(py, &self).into()
    }
}

impl<'a> IntoPy<PyObject> for &'a String {
    #[inline]
    fn into_py(self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'source> FromPyObject<'source> for &'source str {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        <PyString as PyTryFrom>::try_from(ob)?.to_str()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl FromPyObject<'_> for String {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        <PyString as PyTryFrom>::try_from(obj)?
            .to_str()
            .map(ToOwned::to_owned)
    }
}

impl FromPyObject<'_> for char {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        let s = PyString::try_from(obj)?.to_str()?;
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
    use super::PyString;
    use crate::Python;
    use crate::{FromPyObject, PyObject, PyTryFrom, ToPyObject};

    #[test]
    fn test_non_bmp() {
        Python::with_gil(|py| {
            let s = "\u{1F30F}";
            let py_string = s.to_object(py);
            assert_eq!(s, py_string.extract::<String>(py).unwrap());
        })
    }

    #[test]
    fn test_extract_str() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = s.to_object(py);

            let s2: &str = FromPyObject::extract(py_string.as_ref(py)).unwrap();
            assert_eq!(s, s2);
        })
    }

    #[test]
    fn test_extract_char() {
        Python::with_gil(|py| {
            let ch = '😃';
            let py_string = ch.to_object(py);
            let ch2: char = FromPyObject::extract(py_string.as_ref(py)).unwrap();
            assert_eq!(ch, ch2);
        })
    }

    #[test]
    fn test_extract_char_err() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = s.to_object(py);
            let err: crate::PyResult<char> = FromPyObject::extract(py_string.as_ref(py));
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
            let obj: PyObject = PyString::new(py, s).into();
            let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
            assert_eq!(s, py_string.to_str().unwrap());
        })
    }

    #[test]
    fn test_to_str_surrogate() {
        Python::with_gil(|py| {
            let obj: PyObject = py.eval(r#"'\ud800'"#, None, None).unwrap().into();
            let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
            assert!(py_string.to_str().is_err());
        })
    }

    #[test]
    fn test_to_str_unicode() {
        Python::with_gil(|py| {
            let s = "哈哈🐈";
            let obj: PyObject = PyString::new(py, s).into();
            let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
            assert_eq!(s, py_string.to_str().unwrap());
        })
    }

    #[test]
    fn test_to_string_lossy() {
        Python::with_gil(|py| {
            let obj: PyObject = py
                .eval(r#"'🐈 Hello \ud800World'"#, None, None)
                .unwrap()
                .into();
            let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
            assert_eq!(py_string.to_string_lossy(), "🐈 Hello ���World");
        })
    }

    #[test]
    fn test_debug_string() {
        Python::with_gil(|py| {
            let v = "Hello\n".to_object(py);
            let s = <PyString as PyTryFrom>::try_from(v.as_ref(py)).unwrap();
            assert_eq!(format!("{:?}", s), "'Hello\\n'");
        })
    }

    #[test]
    fn test_display_string() {
        Python::with_gil(|py| {
            let v = "Hello\n".to_object(py);
            let s = <PyString as PyTryFrom>::try_from(v.as_ref(py)).unwrap();
            assert_eq!(format!("{}", s), "Hello\n");
        })
    }
}
