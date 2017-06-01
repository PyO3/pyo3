// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::ptr;
use std::os::raw::c_char;
use ffi;
use python::{Python, ToPythonPointer};
use objects::PyObject;
use err::{PyResult, PyErr};
use pointers::{Ptr, PyPtr};
use token::PythonObjectWithGilToken;


/// Represents a Python bytearray.
pub struct PyByteArray<'p>(Ptr<'p>);
pub struct PyByteArrayPtr(PyPtr);

pyobject_nativetype!(PyByteArray, PyByteArray_Check, PyByteArray_Type, PyByteArrayPtr);


impl<'p> PyByteArray<'p> {
    /// Creates a new Python bytearray object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new<'a>(py: Python<'a>, src: &[u8]) -> PyByteArray<'a> {
        let ptr = src.as_ptr() as *const c_char;
        let len = src.len() as ffi::Py_ssize_t;
        let ptr = unsafe {ffi::PyByteArray_FromStringAndSize(ptr, len)};
        PyByteArray(Ptr::from_owned_ptr_or_panic(py, ptr))
    }

    /// Creates a new Python bytearray object
    /// from other PyObject, that implements the buffer protocol.
    pub fn from(src: &'p PyObject<'p>) -> PyResult<PyByteArray<'p>> {
        let res = unsafe {ffi::PyByteArray_FromObject(src.as_ptr())};
        if res != ptr::null_mut() {
            Ok(PyByteArray(Ptr::from_owned_ptr_or_panic(src.gil(), res)))
        } else {
            Err(PyErr::fetch(src.gil()))
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

    /// Gets the Python bytearray data as byte slice.
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
                Err(PyErr::fetch(self.0.token()))
            }
        }
    }
}


#[cfg(test)]
mod test {
    use ::ToPyObject;
    use exc;
    use python::Python;
    use typeob::PyTypeObject;
    use objects::PyByteArray;

    #[test]
    fn test_bytearray() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let src = b"Hello Python";
        let bytearray = PyByteArray::new(py, src);
        assert_eq!(src.len(), bytearray.len());
        assert_eq!(src, bytearray.data());

        let ba = bytearray.to_object(py);
        let bytearray = PyByteArray::from(&ba).unwrap();
        assert_eq!(src.len(), bytearray.len());
        assert_eq!(src, bytearray.data());

        bytearray.resize(20).unwrap();
        assert_eq!(20, bytearray.len());

        let none = py.None();
        if let Err(mut err) = PyByteArray::from(&none) {
            assert!(exc::TypeError::type_object(py).is_instance(&err.instance(py)))
        } else {
            panic!("error");
        }
        drop(none);
    }
}
