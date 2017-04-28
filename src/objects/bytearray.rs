// Copyright (c) 2017 Nikolay Kim
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std;
use std::{mem, str, char, ptr};
use std::ascii::AsciiExt;
use std::borrow::Cow;
use libc::c_char;
use ffi;
use python::{Python, PythonObject, PyClone, ToPythonPointer, PythonObjectDowncastError};
use super::{exc, PyObject};
use err::{self, PyResult, PyErr};
use conversion::{FromPyObject, RefFromPyObject, ToPyObject};

/// Represents a Python bytearray.
pub struct PyByteArray(PyObject);

pyobject_newtype!(PyByteArray, PyByteArray_Check, PyByteArray_Type);

impl PyByteArray {
    /// Creates a new Python bytearray object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python, src: &[u8]) -> PyByteArray {
        let ptr = src.as_ptr() as *const c_char;
        let len = src.len() as ffi::Py_ssize_t;
        unsafe {
            err::cast_from_owned_ptr_or_panic(py,
                ffi::PyByteArray_FromStringAndSize(ptr, len))
        }
    }

    /// Creates a new Python bytearray object
    /// from other PyObject, that implements the buffer protocol.
    pub fn from(py: Python, src: PyObject) -> PyResult<PyByteArray> {
        unsafe {
            let res = ffi::PyByteArray_FromObject(src.as_ptr());
            if res != ptr::null_mut() {
                Ok(err::cast_from_owned_ptr_or_panic(py, res))
            } else {
                Err(PyErr::fetch(py))
            }
        }
    }

    /// Gets the length of the bytearray.
    #[inline]
    pub fn len(&self, _py: Python) -> usize {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe {
            ffi::PyByteArray_Size(self.0.as_ptr()) as usize
        }
    }

    /// Gets the Python bytearray data as byte slice.
    pub fn data(&self, _py: Python) -> &mut [u8] {
        unsafe {
            let buffer = ffi::PyByteArray_AsString(self.as_ptr()) as *mut u8;
            let length = ffi::PyByteArray_Size(self.as_ptr()) as usize;
            std::slice::from_raw_parts_mut(buffer, length)
        }
    }

    /// Resize bytearray object.
    pub fn resize(&self, py: Python, len: usize) -> PyResult<()> {
        unsafe {
            let result = ffi::PyByteArray_Resize(self.as_ptr(), len as ffi::Py_ssize_t);
            if result == 0 {
                Ok(())
            } else {
                Err(PyErr::fetch(py))
            }
        }
    }
}


#[cfg(test)]
mod test {
    use exc;
    use python::{Python, PythonObject, PythonObjectWithTypeObject};
    use objects::PyByteArray;

    #[test]
    fn test_bytearray() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let src = b"Hello Python";
        let bytearray = PyByteArray::new(py, src);
        assert_eq!(src.len(), bytearray.len(py));
        assert_eq!(src, bytearray.data(py));

        let bytearray = PyByteArray::from(py, bytearray.into_object()).unwrap();
        assert_eq!(src.len(), bytearray.len(py));
        assert_eq!(src, bytearray.data(py));

        bytearray.resize(py, 20).unwrap();
        assert_eq!(20, bytearray.len(py));

        if let Err(mut err) = PyByteArray::from(py, py.None()) {
            assert!(exc::TypeError::type_object(py).is_instance(py, &err.instance(py)))
        } else {
            panic!("error");
        }
    }
}
