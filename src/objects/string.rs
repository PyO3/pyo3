use std;
use std::{char, str};
use std::ascii::AsciiExt;
use std::borrow::Cow;
use libc::c_char;
use ffi;
use python::{Python, PythonObject, ToPythonPointer};
use super::{exc, PyObject};
use err::{self, PyResult, PyErr};
use conversion::{FromPyObject, ToPyObject};

//pyobject_newtype!(PyBytes, PyBytes_Check, PyBytes_Type);

pyobject_newtype!(PyString, PyString_Check, PyString_Type);

pyobject_newtype!(PyUnicode, PyUnicode_Check, PyUnicode_Type);

impl <'p> PyString<'p> {
    /// Creates a new python string object from the Rust string.
    ///
    /// Note: on Python 2, this function always creates a `str` object,
    /// never a `unicode` object.
    /// Use `str::to_py_object()` instead to create `unicode` objects for non-ascii strings.
    pub fn new(py: Python<'p>, s: &str) -> PyString<'p> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            err::cast_from_owned_ptr_or_panic(py,
                ffi::PyString_FromStringAndSize(ptr, len))
        }
    }

    /// Gets the python string data as byte slice.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let buffer = ffi::PyString_AS_STRING(self.as_ptr()) as *const u8;
            let length = ffi::PyString_GET_SIZE(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length)
        }
    }

    /// Gets the python string data as `&str`.
    pub fn as_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self.as_slice())
    }
}

impl <'p> PyUnicode<'p> {
    /// Creates a new unicode string object from the Rust string.
    pub fn new(py: Python<'p>, s: &str) -> PyUnicode<'p> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            err::cast_from_owned_ptr_or_panic(py,
                ffi::PyUnicode_FromStringAndSize(ptr, len))
        }
    }

    pub fn as_slice(&self) -> &[ffi::Py_UNICODE] {
        unsafe {
            let buffer = ffi::PyUnicode_AS_UNICODE(self.as_ptr()) as *const _;
            let length = ffi::PyUnicode_GET_SIZE(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length)
        }
    }
}

// When converting strings to/from python, we need to copy the string data.
// This means we can implement ToPyObject for str, but FromPyObject only for (Cow)String.

/// Converts rust `str` to python object:
/// ASCII-only strings are converted to python `str` objects;
/// other strings are converted to python `unicode` objects.
impl <'p> ToPyObject<'p> for str {
    type ObjectType = PyObject<'p>;

    fn to_py_object(&self, py : Python<'p>) -> PyResult<'p, PyObject<'p>> {
        let ptr = self.as_ptr() as *const c_char;
        let len = self.len() as ffi::Py_ssize_t;
        unsafe {
            let obj = if self.is_ascii() {
                ffi::PyString_FromStringAndSize(ptr, len)
            } else {
                ffi::PyUnicode_FromStringAndSize(ptr, len)
            };
            err::result_from_owned_ptr(py, obj)
        }
    }
}

/// Converts rust `&str` to python object:
/// ASCII-only strings are converted to python `str` objects;
/// other strings are converted to python `unicode` objects.
impl <'p, 'a> ToPyObject<'p> for &'a str {
    type ObjectType = PyObject<'p>;

    fn to_py_object(&self, py : Python<'p>) -> PyResult<'p, PyObject<'p>> {
        (**self).to_py_object(py)
    }
}

fn u32_as_bytes(input: &[u32]) -> &[u8] {
    unsafe { std::mem::transmute(input) }
}

impl <'p, 's> FromPyObject<'p, 's> for Cow<'s, str> {
    fn from_py_object(o: &'s PyObject<'p>) -> PyResult<'p, Cow<'s, str>> {
        let py = o.python();
        if let Ok(s) = o.cast_as::<PyString>() {
            match s.as_str() {
                Ok(s) => Ok(Cow::Borrowed(s)),
                Err(e) => Err(PyErr::new(try!(exc::UnicodeDecodeError::new_utf8(py, s.as_slice(), e))))
            }
        } else if let Ok(u) = o.cast_as::<PyUnicode>() {
            let u = u.as_slice();
            let mut s = String::with_capacity(u.len());
            for (i, &c) in u.iter().enumerate() {
                match char::from_u32(c) {
                    Some(c) => s.push(c),
                    None => {
                        let e = try!(exc::UnicodeDecodeError::new(
                            py, cstr!("utf-32"), u32_as_bytes(u), i .. i+1, cstr!("invalid code point")));
                        return Err(PyErr::new(e));
                    }
                }
            }
            Ok(Cow::Owned(s))
        } else {
            Err(PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None))
        }
    }
}

impl <'p, 's> FromPyObject<'p, 's> for String {
    fn from_py_object(o: &'s PyObject<'p>) -> PyResult<'p, String> {
        Ok(try!(o.extract::<Cow<str>>()).into_owned())
    }
}

#[test]
fn test_non_bmp() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let s = "\u{1F30F}";
    let py_string = s.to_py_object(py).unwrap();
    assert_eq!(s, py_string.extract::<Cow<str>>().unwrap());
}

