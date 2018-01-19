// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::os::raw::c_char;
use ffi;
use object::PyObject;
use instance::PyObjectWithToken;
use python::{Python, ToPyPointer};
use err::{PyResult, PyErr};

/// Represents a Python `bytearray`.
pub struct PyByteArray(PyObject);

pyobject_convert!(PyByteArray);
pyobject_nativetype!(PyByteArray, PyByteArray_Type, PyByteArray_Check);

impl PyByteArray {
    /// Creates a new Python bytearray object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new<'p>(py: Python<'p>, src: &[u8]) -> &'p PyByteArray {
        let ptr = src.as_ptr() as *const c_char;
        let len = src.len() as ffi::Py_ssize_t;
        unsafe {
            py.from_owned_ptr::<PyByteArray>(
                ffi::PyByteArray_FromStringAndSize(ptr, len))
        }
    }

    /// Creates a new Python bytearray object
    /// from other PyObject, that implements the buffer protocol.
    pub fn from<'p, I>(py: Python<'p>, src: &'p I) -> PyResult<&'p PyByteArray>
        where I: ToPyPointer
    {
        unsafe {
            py.from_owned_ptr_or_err(
                ffi::PyByteArray_FromObject(src.as_ptr()))
        }
    }

    /// Gets the length of the bytearray.
    #[inline]
    pub fn len(&self) -> usize {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe {
            ffi::PyByteArray_Size(self.0.as_ptr()) as usize
        }
    }

    /// Check if bytearray is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the Python bytearray data as byte slice.
    #[cfg_attr(feature = "cargo-clippy", allow(mut_from_ref))]
    pub fn data(&self) -> &mut [u8] {
        unsafe {
            let buffer = ffi::PyByteArray_AsString(self.0.as_ptr()) as *mut u8;
            let length = ffi::PyByteArray_Size(self.0.as_ptr()) as usize;
            std::slice::from_raw_parts_mut(buffer, length)
        }
    }

    /// Resize bytearray object.
    pub fn resize(&self, len: usize) -> PyResult<()> {
        unsafe {
            let result = ffi::PyByteArray_Resize(self.0.as_ptr(), len as ffi::Py_ssize_t);
            if result == 0 {
                Ok(())
            } else {
                Err(PyErr::fetch(self.py()))
            }
        }
    }
}


#[cfg(test)]
mod test {
    use exc;
    use python::Python;
    use object::PyObject;
    use objects::PyByteArray;

    #[test]
    fn test_bytearray() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let src = b"Hello Python";
        let bytearray = PyByteArray::new(py, src);
        assert_eq!(src.len(), bytearray.len());
        assert_eq!(src, bytearray.data());

        let ba: PyObject = bytearray.into();
        let bytearray = PyByteArray::from(py, &ba).unwrap();

        assert_eq!(src.len(), bytearray.len());
        assert_eq!(src, bytearray.data());

        bytearray.resize(20).unwrap();
        assert_eq!(20, bytearray.len());

        let none = py.None();
        if let Err(err) = PyByteArray::from(py, &none) {
            assert!(err.is_instance::<exc::TypeError>(py));
        } else {
            panic!("error");
        }
        drop(none);
    }
}
