// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::types::PyBytes;
use crate::{
    ffi, AsPyPointer, FromPy, FromPyObject, IntoPy, Py, PyErr, PyNativeType, PyObject, PyResult,
    Python, ToPyObject,
};
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::str;

/// Represents a Python `string` (a Unicode string object).
///
/// This type is immutable.
#[repr(transparent)]
pub struct PyString(PyObject);

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

    pub fn from_object<'p>(
        src: &'p PyObject,
        encoding: &str,
        errors: &str,
    ) -> PyResult<&'p PyString> {
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
    pub fn as_bytes(&self) -> PyResult<&[u8]> {
        unsafe {
            let mut size: ffi::Py_ssize_t = 0;
            let data = ffi::PyUnicode_AsUTF8AndSize(self.as_ptr(), &mut size) as *const u8;
            if data.is_null() {
                Err(PyErr::fetch(self.py()))
            } else {
                Ok(std::slice::from_raw_parts(data, size as usize))
            }
        }
    }

    /// Converts the `PyString` into a Rust string.
    pub fn to_string(&self) -> PyResult<Cow<str>> {
        let bytes = self.as_bytes()?;
        let string = std::str::from_utf8(bytes)?;
        Ok(Cow::Borrowed(string))
    }

    /// Converts the `PyString` into a Rust string.
    ///
    /// Unpaired surrogates invalid UTF-8 sequences are
    /// replaced with `U+FFFD REPLACEMENT CHARACTER`.
    pub fn to_string_lossy(&self) -> Cow<str> {
        match self.to_string() {
            Ok(s) => s,
            Err(_) => {
                let bytes = unsafe {
                    self.py()
                        .from_owned_ptr::<PyBytes>(ffi::PyUnicode_AsEncodedString(
                            self.as_ptr(),
                            CStr::from_bytes_with_nul(b"utf-8\0").unwrap().as_ptr(),
                            CStr::from_bytes_with_nul(b"surrogatepass\0")
                                .unwrap()
                                .as_ptr(),
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
    fn to_object<'p>(&self, py: Python<'p>) -> &'p PyObject {
        PyString::new(py, self).into()
    }
}

impl<'a> IntoPy<Py<PyObject>> for &'a str {
    #[inline]
    fn into_py(self, py: Python) -> Py<PyObject> {
        PyString::new(py, self).into_py(py)
    }
}

/// Converts a Rust `Cow<str>` to a Python object.
/// See `PyString::new` for details on the conversion.
impl<'a> ToPyObject for Cow<'a, str> {
    #[inline]
    fn to_object<'p>(&self, py: Python<'p>) -> &'p PyObject {
        PyString::new(py, self).into()
    }
}

/// Converts a Rust `String` to a Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_object<'p>(&self, py: Python<'p>) -> &'p PyObject {
        PyString::new(py, self).into()
    }
}

impl FromPy<String> for Py<PyObject> {
    fn from_py(other: String, py: Python) -> Self {
        PyString::new(py, &other).into_py(py)
    }
}

impl<'a> IntoPy<Py<PyObject>> for &'a String {
    #[inline]
    fn into_py(self, py: Python) -> Py<PyObject> {
        PyString::new(py, self).into_py(py)
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'source> crate::FromPyObject<'source> for Cow<'source, str> {
    fn extract(ob: &'source PyObject) -> PyResult<Self> {
        ob.downcast::<PyString>()?.to_string()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'a> crate::FromPyObject<'a> for &'a str {
    fn extract(ob: &'a PyObject) -> PyResult<Self> {
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
    fn extract(obj: &'source PyObject) -> PyResult<Self> {
        obj.downcast::<PyString>()?.to_string().map(Cow::into_owned)
    }
}

#[cfg(test)]
mod test {
    use super::PyString;
    use crate::Python;
    use crate::{FromPyObject, PyTryFrom, ToPyObject};
    use std::borrow::Cow;

    #[test]
    fn test_non_bmp() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "\u{1F30F}";
        let py_string = s.to_object(py);
        assert_eq!(s, py_string.extract::<String>().unwrap());
    }

    #[test]
    fn test_extract_str() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "Hello Python";
        let py_string = s.to_object(py);

        let s2: &str = FromPyObject::extract(py_string).unwrap();
        assert_eq!(s, s2);
    }

    #[test]
    fn test_as_bytes() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "ascii 🐈";
        let py_string = PyString::new(py, s);
        assert_eq!(s.as_bytes(), py_string.as_bytes().unwrap());
    }

    #[test]
    fn test_as_bytes_surrogate() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = py.eval(r#"'\ud800'"#, None, None).unwrap();
        let py_string: &PyString = obj.downcast().unwrap();
        assert!(py_string.as_bytes().is_err());
    }

    #[test]
    fn test_to_string_ascii() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "ascii";
        let py_string = PyString::new(py, s);
        assert!(py_string.to_string().is_ok());
        assert_eq!(Cow::Borrowed(s), py_string.to_string().unwrap());
    }

    #[test]
    fn test_to_string_unicode() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "哈哈🐈";
        let py_string = PyString::new(py, s);
        assert!(py_string.to_string().is_ok());
        assert_eq!(Cow::Borrowed(s), py_string.to_string().unwrap());
    }

    #[test]
    fn test_to_string_lossy() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = py.eval(r#"'🐈 Hello \ud800World'"#, None, None).unwrap();
        let py_string: &PyString = obj.downcast().unwrap();
        assert_eq!(py_string.to_string_lossy(), "🐈 Hello ���World");
    }

    #[test]
    fn test_debug_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        let s = <PyString as PyTryFrom>::try_from(v).unwrap();
        assert_eq!(format!("{:?}", s), "'Hello\\n'");
    }

    #[test]
    fn test_display_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        let s = <PyString as PyTryFrom>::try_from(v).unwrap();
        assert_eq!(format!("{}", s), "Hello\n");
    }
}
