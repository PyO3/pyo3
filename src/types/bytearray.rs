// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::err::{PyErr, PyResult};
use crate::ffi;
use crate::instance::PyNativeType;
use crate::internal_tricks::Unsendable;
use crate::object::PyObject;
use crate::AsPyPointer;
use crate::Python;
use std::os::raw::c_char;
use std::slice;

/// Represents a Python `bytearray`.
#[repr(transparent)]
pub struct PyByteArray(PyObject, Unsendable);

pyobject_native_var_type!(PyByteArray, ffi::PyByteArray_Type, ffi::PyByteArray_Check);

impl PyByteArray {
    /// Creates a new Python bytearray object.
    ///
    /// The byte string is initialized by copying the data from the `&[u8]`.
    pub fn new<'p>(py: Python<'p>, src: &[u8]) -> &'p PyByteArray {
        let ptr = src.as_ptr() as *const c_char;
        let len = src.len() as ffi::Py_ssize_t;
        unsafe { py.from_owned_ptr::<PyByteArray>(ffi::PyByteArray_FromStringAndSize(ptr, len)) }
    }

    /// Creates a new Python bytearray object from another PyObject that
    /// implements the buffer protocol.
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

    /// Checks if the bytearray is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Copies the contents of the bytearray to a Rust vector.
    ///
    /// # Example
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::types::PyByteArray;
    /// # use pyo3::types::IntoPyDict;
    /// # let gil = GILGuard::acquire();
    /// # let py = gil.python();
    /// #
    /// let bytearray = PyByteArray::new(py, b"Hello World.");
    /// let mut copied_message = bytearray.to_vec();
    /// assert_eq!(b"Hello World.", copied_message.as_slice());
    ///
    /// copied_message[11] = b'!';
    /// assert_eq!(b"Hello World!", copied_message.as_slice());
    ///
    /// let locals = [("bytearray", bytearray)].into_py_dict(py);
    /// py.run("assert bytearray == b'Hello World.'", None, Some(locals)).unwrap();
    /// ```
    pub fn to_vec(&self) -> Vec<u8> {
        let slice = unsafe {
            let buffer = ffi::PyByteArray_AsString(self.0.as_ptr()) as *mut u8;
            let length = ffi::PyByteArray_Size(self.0.as_ptr()) as usize;
            slice::from_raw_parts_mut(buffer, length)
        };
        slice.to_vec()
    }

    /// Resizes the bytearray object to the new length `len`.
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
        assert_eq!(src, bytearray.to_vec().as_slice());

        let ba: PyObject = bytearray.into();
        let bytearray = PyByteArray::from(py, &ba).unwrap();

        assert_eq!(src.len(), bytearray.len());
        assert_eq!(src, bytearray.to_vec().as_slice());

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
