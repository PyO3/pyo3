// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::ptr;
use std::os::raw::c_char;
use ffi;
use python::{Python, ToPyPointer};
use objects::PyObject;
use err::{PyResult, PyErr};
use pointers::PyPtr;


/// Represents a Python bytearray.
pub struct PyByteArray(PyPtr);

pyobject_convert!(PyByteArray);
pyobject_nativetype!(PyByteArray, PyByteArray_Check, PyByteArray_Type);

impl PyByteArray {
    /// Creates a new Python bytearray object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new(_py: Python, src: &[u8]) -> PyByteArray {
        let ptr = src.as_ptr() as *const c_char;
        let len = src.len() as ffi::Py_ssize_t;
        let ptr = unsafe {ffi::PyByteArray_FromStringAndSize(ptr, len)};
        PyByteArray(PyPtr::from_owned_ptr_or_panic(ptr))
    }

    /// Creates a new Python bytearray object
    /// from other PyObject, that implements the buffer protocol.
    pub fn from<I>(py: Python, src: I) -> PyResult<PyByteArray>
        where I: ToPyPointer
    {
        let res = unsafe {ffi::PyByteArray_FromObject(src.as_ptr())};
        if res != ptr::null_mut() {
            Ok(PyByteArray(PyPtr::from_owned_ptr_or_panic(res)))
        } else {
            Err(PyErr::fetch(py))
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
            let buffer = ffi::PyByteArray_AsString(self.0.as_ptr()) as *mut u8;
            let length = ffi::PyByteArray_Size(self.0.as_ptr()) as usize;
            std::slice::from_raw_parts_mut(buffer, length)
        }
    }

    /// Resize bytearray object.
    pub fn resize(&self, py: Python, len: usize) -> PyResult<()> {
        unsafe {
            let result = ffi::PyByteArray_Resize(self.0.as_ptr(), len as ffi::Py_ssize_t);
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
    use python::Python;
    use objects::{PyObject, PyByteArray};

    #[test]
    fn test_bytearray() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let src = b"Hello Python";
        let bytearray = PyByteArray::new(py, src);
        assert_eq!(src.len(), bytearray.len(py));
        assert_eq!(src, bytearray.data(py));

        let ba: PyObject = bytearray.into();
        let bytearray = PyByteArray::from(py, &ba).unwrap();
        assert_eq!(src.len(), bytearray.len(py));
        assert_eq!(src, bytearray.data(py));

        bytearray.resize(py, 20).unwrap();
        assert_eq!(20, bytearray.len(py));

        let none = py.None();
        if let Err(mut err) = PyByteArray::from(py, &none) {
            assert!(py.is_instance::<exc::TypeError>(&err.instance(py)).unwrap())
        } else {
            panic!("error");
        }
        drop(none);
    }
}
