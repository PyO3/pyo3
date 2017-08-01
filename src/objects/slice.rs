// Copyright (c) 2017-present PyO3 Project and Contributors

use std::os::raw::c_long;

use object::PyObject;
use python::{ToPyPointer, Python};
use err::{PyErr, PyResult};
use ffi::{self, Py_ssize_t};
use instance::PyObjectWithToken;
use conversion::ToPyObject;

/// Represents a Python `slice`.
///
/// Only `c_long` indeces supprted at the moment by `PySlice` object.
pub struct PySlice(PyObject);

pyobject_convert!(PySlice);
pyobject_nativetype!(PySlice, PySlice_Type, PySlice_Check);


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


impl PySlice {

    /// Construct a new slice with the given elements.
    pub fn new(py: Python, start: isize, stop: isize, step: isize) -> &PySlice {
        unsafe {
            let ptr = ffi::PySlice_New(ffi::PyLong_FromLong(start as c_long),
                                       ffi::PyLong_FromLong(stop as c_long),
                                       ffi::PyLong_FromLong(step as c_long));
            py.from_owned_ptr(ptr)
        }
    }

    /// Retrieve the start, stop, and step indices from the slice object slice assuming a sequence of length length, and store the length of the slice in slicelength.
    #[inline]
    pub fn indices(&self, length: c_long) -> PyResult<PySliceIndices> {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe {
            let slicelen: isize = 0;
            let start: isize = 0;
            let stop: isize = 0;
            let step: isize = 0;
            let r = ffi::PySlice_GetIndicesEx(
                self.as_ptr(), length as Py_ssize_t,
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
                Err(PyErr::fetch(self.py()))
            }
        }
    }
}

impl ToPyObject for PySliceIndices {
    fn to_object(&self, py: Python) -> PyObject {
        PySlice::new(py, self.start, self.stop, self.step).into()
    }
}
