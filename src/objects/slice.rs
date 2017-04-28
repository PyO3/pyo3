// Copyright (c) 2017 Nikolay Kim
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::mem;
use libc::c_long;
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
