// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::err::{PyErr, PyResult};
use crate::ffi;
use crate::instance::PyNativeType;
use crate::object::PyObject;
use crate::AsPyPointer;
use crate::Python;
use std::os::raw::c_char;
use std::slice;

/// Represents a Python `bytearray`.
#[repr(transparent)]
pub struct PyByteArray(PyObject);

pyobject_native_type!(PyByteArray, ffi::PyByteArray_Type, ffi::PyByteArray_Check);

impl PyByteArray {
    /// Creates a new Python bytearray object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new<'p>(py: Python<'p>, src: &[u8]) -> &'p PyByteArray {
        let ptr = src.as_ptr() as *const c_char;
        let len = src.len() as ffi::Py_ssize_t;
        unsafe { py.from_owned_ptr::<PyByteArray>(ffi::PyByteArray_FromStringAndSize(ptr, len)) }
    }

    /// Creates a new Python bytearray object
    /// from other PyObject, that implements the buffer protocol.
    pub fn from<'p, I>(py: Python<'p>, src: &'p I) -> PyResult<&'p PyByteArray>
    where
        I: AsPyPointer,
    {
        unsafe { py.from_owned_ptr_or_err(ffi::PyByteArray_FromObject(src.as_ptr())) }
    }

    /// Gets the length of the bytearray.
    #[inline]
    pub fn len(&self) -> usize {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe { ffi::PyByteArray_Size(self.0.as_ptr()) as usize }
    }

    /// Check if bytearray is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the Python bytearray data as byte slice.
    pub fn data(&self) -> &[u8] {
        unsafe {
            let buffer = ffi::PyByteArray_AsString(self.0.as_ptr()) as *mut u8;
            let length = ffi::PyByteArray_Size(self.0.as_ptr()) as usize;
            slice::from_raw_parts_mut(buffer, length)
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
    use crate::exceptions;
    use crate::object::PyObject;
    use crate::types::PyByteArray;
    use crate::Python;

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
            assert!(err.is_instance::<exceptions::TypeError>(py));
        } else {
            panic!("error");
        }
        drop(none);
    }
}
