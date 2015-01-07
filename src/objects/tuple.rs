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
    pub fn len(&self) -> uint {
        // non-negative Py_ssize_t should always fit into Rust uint
        unsafe {
            ffi::PyTuple_GET_SIZE(self.as_ptr()) as uint
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
}

impl<'p> std::ops::Index<uint> for PyTuple<'p> {
    type Output = PyObject<'p>;

    #[inline]
    fn index<'a>(&'a self, index: &uint) -> &'a PyObject<'p> {
        // use as_slice() to use the normal Rust bounds checking when indexing
        &self.as_slice()[*index]
    }
}

