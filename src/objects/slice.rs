// Copyright (c) 2017-present PyO3 Project and Contributors

use std::mem;
use std::os::raw::c_long;
use super::object::PyObject;
use python::{Python, PythonObject, ToPythonPointer, PyClone, PyDrop};
use err::{self, PyErr, PyResult};
use ffi::{self, Py_ssize_t};
use conversion::{ToPyObject, FromPyObject};

/// Represents a Python `slice` indices
pub struct PySliceIndices {
    pub start: isize,
    pub stop: isize,
    pub step: isize,
    pub slicelength: isize,
}

impl PySliceIndices {
    pub fn new(start: isize, stop: isize, step: isize) -> PySliceIndices {
        PySliceIndices {
            start: start,
            stop: stop,
            step: step,
            slicelength: 0,
        }
    }
}


/// Represents a Python `slice`. Only `c_long` indeces supprted
/// at the moment by PySlice object.
pub struct PySlice(PyObject);

pyobject_newtype!(PySlice, PySlice_Check, PySlice_Type);

impl PySlice {
    /// Construct a new slice with the given elements.
    pub fn new(py: Python, start: isize, stop: isize, step: isize) -> PySlice {
        unsafe {
            let ptr = ffi::PySlice_New(ffi::PyLong_FromLong(start as i64),
                                       ffi::PyLong_FromLong(stop as i64),
                                       ffi::PyLong_FromLong(step as i64));
            err::result_from_owned_ptr(py, ptr).unwrap().unchecked_cast_into::<PySlice>()
        }
    }

    /// Retrieve the start, stop, and step indices from the slice object slice assuming a sequence of length length, and store the length of the slice in slicelength.
    #[inline]
    pub fn indices(&self, py: Python, length: c_long) -> PyResult<PySliceIndices> {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe {
            let slicelen: isize = 0;
            let start: isize = 0;
            let stop: isize = 0;
            let step: isize = 0;
            let r = ffi::PySlice_GetIndicesEx(
                self.0.as_ptr(), length as Py_ssize_t,
                &start as *const _ as *mut _,
                &stop as *const _ as *mut _,
                &step as *const _ as *mut _,
                &slicelen as *const _ as *mut _);
            if r == 0{
                Ok(PySliceIndices {
                    start: start,
                    stop: stop,
                    step: step,
                    slicelength: slicelen,
                })
            } else {
                Err(PyErr::fetch(py))
            }
        }
    }
}

impl ToPyObject for PySliceIndices {
    type ObjectType = PySlice;

    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        PySlice::new(py, self.start, self.stop, self.step)
    }
}
