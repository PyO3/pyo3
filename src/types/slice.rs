// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::err::{PyErr, PyResult};
use crate::ffi::{self, Py_ssize_t};
use crate::{AsPyPointer, PyAny, PyObject, Python, ToPyObject};
use std::os::raw::c_long;

/// Represents a Python `slice`.
///
/// Only `c_long` indices supported at the moment by the `PySlice` object.
#[repr(transparent)]
pub struct PySlice(PyAny);

pyobject_native_type!(
    PySlice,
    ffi::PySliceObject,
    ffi::PySlice_Type,
    #checkfunction=ffi::PySlice_Check
);

/// Represents Python `slice` indices.
pub struct PySliceIndices {
    pub start: isize,
    pub stop: isize,
    pub step: isize,
    pub slicelength: isize,
}

impl PySliceIndices {
    pub fn new(start: isize, stop: isize, step: isize) -> PySliceIndices {
        PySliceIndices {
            start,
            stop,
            step,
            slicelength: 0,
        }
    }
}

impl PySlice {
    /// Constructs a new slice with the given elements.
    pub fn new(py: Python, start: isize, stop: isize, step: isize) -> &PySlice {
        unsafe {
            let ptr = ffi::PySlice_New(
                ffi::PyLong_FromLong(start as c_long),
                ffi::PyLong_FromLong(stop as c_long),
                ffi::PyLong_FromLong(step as c_long),
            );
            py.from_owned_ptr(ptr)
        }
    }

    /// Retrieves the start, stop, and step indices from the slice object,
    /// assuming a sequence of length `length`, and stores the length of the
    /// slice in its `slicelength` member.
    #[inline]
    pub fn indices(&self, length: c_long) -> PyResult<PySliceIndices> {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe {
            let mut slicelength: isize = 0;
            let mut start: isize = 0;
            let mut stop: isize = 0;
            let mut step: isize = 0;
            let r = ffi::PySlice_GetIndicesEx(
                self.as_ptr(),
                length as Py_ssize_t,
                &mut start,
                &mut stop,
                &mut step,
                &mut slicelength,
            );
            if r == 0 {
                Ok(PySliceIndices {
                    start,
                    stop,
                    step,
                    slicelength,
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
