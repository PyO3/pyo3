// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::types::PyBytes;
use crate::{
    ffi, AsPyPointer, FromPy, FromPyObject, IntoPy, PyAny, PyErr, PyNativeType, PyObject, PyResult,
    PyTryFrom, Python, ToPyObject,
};
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

/// Converts a Rust `String` to a Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

impl FromPy<String> for PyObject {
    fn from_py(other: String, py: Python) -> Self {
        PyString::new(py, &other).into()
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
impl<'source> crate::FromPyObject<'source> for &'source str {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        <PyString as PyTryFrom>::try_from(ob)?.to_str()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'source> FromPyObject<'source> for String {
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        <PyString as PyTryFrom>::try_from(obj)?
            .to_str()
            .map(ToOwned::to_owned)
    }
}

#[cfg(test)]
mod test {
    use super::PyString;
    use crate::instance::AsPyRef;
    use crate::object::PyObject;
    use crate::Python;
    use crate::{FromPyObject, PyTryFrom, ToPyObject};

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

        let s2: &str = FromPyObject::extract(py_string.as_ref(py)).unwrap();
        assert_eq!(s, s2);
    }

    #[test]
    fn test_to_str_ascii() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "ascii 🐈";
        let obj: PyObject = PyString::new(py, s).into();
        let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
        assert_eq!(s, py_string.to_str().unwrap());
    }

    #[test]
    fn test_to_str_surrogate() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(
            !crate::PyErr::occurred(py),
            "test must begin without exceptions"
        );
        let obj: PyObject = py.eval(r#"'\ud800'"#, None, None).unwrap().into();
        let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
        assert!(py_string.to_str().is_err());
    }

    #[test]
    fn test_to_str_unicode() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "哈哈🐈";
        let obj: PyObject = PyString::new(py, s).into();
        let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
        assert_eq!(s, py_string.to_str().unwrap());
    }

    #[test]
    fn test_to_string_lossy() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(
            !crate::PyErr::occurred(py),
            "test must begin without exceptions"
        );
        let obj: PyObject = py
            .eval(r#"'🐈 Hello \ud800World'"#, None, None)
            .unwrap()
            .into();
        let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
        assert_eq!(py_string.to_string_lossy(), "🐈 Hello ���World");
    }

    #[test]
    fn test_debug_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        let s = <PyString as PyTryFrom>::try_from(v.as_ref(py)).unwrap();
        assert_eq!(format!("{:?}", s), "'Hello\\n'");
    }

    #[test]
    fn test_display_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        let s = <PyString as PyTryFrom>::try_from(v.as_ref(py)).unwrap();
        assert_eq!(format!("{}", s), "Hello\n");
    }
}
