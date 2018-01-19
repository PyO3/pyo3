// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::{mem, str};
use std::borrow::Cow;
use std::os::raw::c_char;

use ffi;
use instance::{Py, PyObjectWithToken};
use object::PyObject;
use objects::PyObjectRef;
use python::{ToPyPointer, Python};
use err::{PyResult, PyErr};
use super::PyStringData;

/// Represents a Python `string`.
pub struct PyString(PyObject);

pyobject_convert!(PyString);
pyobject_nativetype!(PyString, PyUnicode_Type, PyUnicode_Check);

/// Represents a Python `unicode string`.
/// Corresponds to `unicode` in Python 2, and `str` in Python 3.
pub use PyString as PyUnicode;

/// Represents a Python `byte` string.
pub struct PyBytes(PyObject);

pyobject_convert!(PyBytes);
pyobject_nativetype!(PyBytes, PyBytes_Type, PyBytes_Check);


impl PyString {

    /// Creates a new Python string object.
    ///
    /// Panics if out of memory.
    pub fn new(_py: Python, s: &str) -> Py<PyString> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            Py::from_owned_ptr_or_panic(ffi::PyUnicode_FromStringAndSize(ptr, len))
        }
    }

    pub fn from_object<'p>(src: &'p PyObjectRef, encoding: &str, errors: &str)
                           -> PyResult<&'p PyString>
    {
        unsafe {
            src.py().from_owned_ptr_or_err::<PyString>(
                ffi::PyUnicode_FromEncodedObject(
                    src.as_ptr(),
                    encoding.as_ptr() as *const c_char,
                    errors.as_ptr() as *const  c_char))
        }
    }

    /// Gets the python string data in its underlying representation.
    pub fn data(&self) -> PyStringData {
        // TODO: return the original representation instead
        // of forcing the UTF-8 representation to be created.
        unsafe {
            let mut size: ffi::Py_ssize_t = mem::uninitialized();
            let data = ffi::PyUnicode_AsUTF8AndSize(self.0.as_ptr(), &mut size) as *const u8;
            if data.is_null() {
                PyErr::fetch(self.py()).print(self.py());
                panic!("PyUnicode_AsUTF8AndSize failed");
            }
            PyStringData::Utf8(std::slice::from_raw_parts(data, size as usize))
        }
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// Returns a `UnicodeDecodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    pub fn to_string(&self) -> PyResult<Cow<str>> {
        self.data().to_string(self.py())
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// Unpaired surrogates invalid UTF-8 sequences are
    /// replaced with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(&self) -> Cow<str> {
        self.data().to_string_lossy()
    }
}


impl PyBytes {
    /// Creates a new Python byte string object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new(_py: Python, s: &[u8]) -> Py<PyBytes> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            Py::from_owned_ptr_or_panic(ffi::PyBytes_FromStringAndSize(ptr, len))
        }
    }

    /// Creates a new Python byte string object from raw pointer.
    ///
    /// Panics if out of memory.
    pub unsafe fn from_ptr(_py: Python, ptr: *const u8, len: usize) -> Py<PyBytes> {
        Py::from_owned_ptr_or_panic(
            ffi::PyBytes_FromStringAndSize(ptr as *const _, len as isize))
    }

    /// Gets the Python string data as byte slice.
    pub fn data(&self) -> &[u8] {
        unsafe {
            let buffer = ffi::PyBytes_AsString(self.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length)
        }
    }
}


#[cfg(test)]
mod test {
    use python::Python;
    use instance::AsPyRef;
    use conversion::{FromPyObject, ToPyObject};

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
}
