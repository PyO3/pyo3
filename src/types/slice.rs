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

/// Return value from [`PySlice::indices`].
#[derive(Debug, Eq, PartialEq)]
pub struct PySliceIndices {
    /// Start of the slice
    pub start: isize,
    /// End of the slice
    pub stop: isize,
    /// Increment to use when iterating the slice from `start` to `stop`.
    pub step: isize,
    /// The length of the slice calculated from the original input sequence.
    pub slicelength: isize,
}

impl PySliceIndices {
    /// Creates a new `PySliceIndices`.
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
    pub fn new(py: Python<'_>, start: isize, stop: isize, step: isize) -> &PySlice {
        unsafe {
            let ptr = ffi::PySlice_New(
                ffi::PyLong_FromSsize_t(start),
                ffi::PyLong_FromSsize_t(stop),
                ffi::PyLong_FromSsize_t(step),
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
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PySlice::new(py, self.start, self.stop, self.step).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_slice_new() {
        Python::with_gil(|py| {
            let slice = PySlice::new(py, isize::MIN, isize::MAX, 1);
            assert_eq!(
                slice.getattr("start").unwrap().extract::<isize>().unwrap(),
                isize::MIN
            );
            assert_eq!(
                slice.getattr("stop").unwrap().extract::<isize>().unwrap(),
                isize::MAX
            );
            assert_eq!(
                slice.getattr("step").unwrap().extract::<isize>().unwrap(),
                1
            );
        });
    }

    #[test]
    fn test_py_slice_indices_new() {
        let start = 0;
        let stop = 0;
        let step = 0;
        assert_eq!(
            PySliceIndices::new(start, stop, step),
            PySliceIndices {
                start,
                stop,
                step,
                slicelength: 0
            }
        );

        let start = 0;
        let stop = 100;
        let step = 10;
        assert_eq!(
            PySliceIndices::new(start, stop, step),
            PySliceIndices {
                start,
                stop,
                step,
                slicelength: 0
            }
        );

        let start = 0;
        let stop = -10;
        let step = -1;
        assert_eq!(
            PySliceIndices::new(start, stop, step),
            PySliceIndices {
                start,
                stop,
                step,
                slicelength: 0
            }
        );

        let start = 0;
        let stop = -10;
        let step = 20;
        assert_eq!(
            PySliceIndices::new(start, stop, step),
            PySliceIndices {
                start,
                stop,
                step,
                slicelength: 0
            }
        );
    }
}
