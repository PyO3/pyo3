// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::ptr;
use std::os::raw::c_char;
use ffi;
use python::{Python, ToPythonPointer, AsPy};
use objects::PyObject;
use err::{PyResult, PyErr};
use pyptr::Py;

/// Represents a Python bytearray.
pub struct PyByteArray;

pyobject_newtype!(PyByteArray, PyByteArray_Check, PyByteArray_Type);


impl PyByteArray {
    /// Creates a new Python bytearray object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new<'p>(py: Python<'p>, src: &[u8]) -> Py<'p, PyByteArray> {
        let ptr = src.as_ptr() as *const c_char;
        let len = src.len() as ffi::Py_ssize_t;
        let ptr = unsafe {ffi::PyByteArray_FromStringAndSize(ptr, len)};
        unsafe { Py::cast_from_owned_ptr_or_panic(py, ptr) }
    }

    /// Creates a new Python bytearray object
    /// from other PyObject, that implements the buffer protocol.
    pub fn from<'p>(py: Python<'p>, src: Py<PyObject>) -> PyResult<Py<'p, PyByteArray>> {
        let res = unsafe {ffi::PyByteArray_FromObject(src.as_ptr())};
        if res != ptr::null_mut() {
            Ok(unsafe{Py::cast_from_owned_ptr_or_panic(py, res)})
        } else {
            Err(PyErr::fetch(py))
        }
    }

    /// Gets the length of the bytearray.
    #[inline]
    pub fn len(&self) -> usize {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe {
            ffi::PyByteArray_Size(self.as_ptr()) as usize
        }
    }

    /// Gets the Python bytearray data as byte slice.
    pub fn data(&self) -> &mut [u8] {
        unsafe {
            let buffer = ffi::PyByteArray_AsString(self.as_ptr()) as *mut u8;
            let length = ffi::PyByteArray_Size(self.as_ptr()) as usize;
            std::slice::from_raw_parts_mut(buffer, length)
        }
    }

    /// Resize bytearray object.
    pub fn resize(&self, len: usize) -> PyResult<()> {
        unsafe {
            let result = ffi::PyByteArray_Resize(self.as_ptr(), len as ffi::Py_ssize_t);
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
    use class::PyTypeObject;
    use python::Python;
    use objects::PyByteArray;

    #[test]
    fn test_bytearray() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let src = b"Hello Python";
        let bytearray = PyByteArray::new(py, src);
        assert_eq!(src.len(), bytearray.len());
        assert_eq!(src, bytearray.data());

        //let bytearray = PyByteArray::from(py, bytearray.into_object()).unwrap();
        //assert_eq!(src.len(), bytearray.len(py));
        //assert_eq!(src, bytearray.data(py));

        bytearray.resize(20).unwrap();
        assert_eq!(20, bytearray.len());

        //if let Err(mut err) = PyByteArray::from(py, py.None()) {
        //    assert!(exc::TypeError::type_object(py).is_instance(py, &err.instance(py)))
        //} else {
        //    panic!("error");
        //}
    }
}
