use std;
use python::{Python, ToPythonPointer};
use err::{self, PyResult};
use super::object::PyObject;
use ffi::{self, Py_ssize_t};

pyobject_newtype!(PyTuple, PyTuple_Check, PyTuple_Type);

impl <'p> PyTuple<'p> {
    pub fn new(py: Python<'p>, elements: &[PyObject<'p>]) -> PyResult<'p, PyTuple<'p>> {
        unsafe {
            let len = elements.len();
            let ptr = ffi::PyTuple_New(len as Py_ssize_t);
            let t = try!(err::result_from_owned_ptr(py, ptr)).unchecked_cast_into::<PyTuple>();
            for (i, e) in elements.iter().enumerate() {
                ffi::PyTuple_SET_ITEM(ptr, i as Py_ssize_t, e.clone().steal_ptr());
            }
            Ok(t)
        }
    }
    
    #[inline]
    pub fn len(&self) -> usize {
        // non-negative Py_ssize_t should always fit into Rust uint
        unsafe {
            ffi::PyTuple_GET_SIZE(self.as_ptr()) as usize
        }
    }
    
    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [PyObject<'p>] {
        // This is safe because PyObject has the same memory layout as *mut ffi::PyObject,
        // and because tuples are immutable.
        unsafe {
            let ptr = self.as_ptr() as *mut ffi::PyTupleObject;
            std::mem::transmute(std::raw::Slice {
                data: (*ptr).ob_item.as_ptr(),
                len: self.len()
            })
        }
    }
    
    #[inline]
    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, PyObject<'p>> {
        self.as_slice().iter()
    }
}

impl<'p> std::ops::Index<usize> for PyTuple<'p> {
    type Output = PyObject<'p>;

    #[inline]
    fn index<'a>(&'a self, index: &usize) -> &'a PyObject<'p> {
        // use as_slice() to use the normal Rust bounds checking when indexing
        &self.as_slice()[*index]
    }
}

