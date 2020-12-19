// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::types::PyBytes;
use crate::{ffi, AsPyPointer, FromPyObject, PyAny, PyErr, PyNativeType, PyResult, Python};
use std::borrow::Cow;
use std::os::raw::c_char;
use std::str;

/// Represents a Python `string` (a Unicode string object).
///
/// This type is immutable.
#[repr(transparent)]
pub struct PyString(PyAny);

pyobject_native_var_type!(PyString, ffi::PyUnicode_Type, ffi::PyUnicode_Check);

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

/// Convenience for calling Python APIs with a temporary string
/// object created from a given Rust string.
pub(crate) fn with_tmp_string<F, R>(py: Python, s: &str, f: F) -> PyResult<R>
where
    F: FnOnce(*mut ffi::PyObject) -> PyResult<R>,
{
    unsafe {
        let s_obj =
            ffi::PyUnicode_FromStringAndSize(s.as_ptr() as *const _, s.len() as ffi::Py_ssize_t);
        if s_obj.is_null() {
            return Err(PyErr::fetch(py));
        }
        let ret = f(s_obj);
        ffi::Py_DECREF(s_obj);
        ret
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'source> FromPyObject<'source> for &'source str {
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        <Self as crate::objects::FromPyObject>::extract(crate::objects::PyAny::from_type_any(&obj))
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl FromPyObject<'_> for String {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        <Self as crate::objects::FromPyObject>::extract(crate::objects::PyAny::from_type_any(&obj))
    }
}

impl FromPyObject<'_> for char {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        <Self as crate::objects::FromPyObject>::extract(crate::objects::PyAny::from_type_any(&obj))
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
