use crate::err::{PyErr, PyResult};
use crate::ffi;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::types::{PyRange, PyRangeMethods};
use crate::{Bound, IntoPyObject, PyAny, Python};
use std::convert::Infallible;

/// Represents a Python `slice`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PySlice>`][crate::Py] or [`Bound<'py, PySlice>`][Bound].
///
/// For APIs available on `slice` objects, see the [`PySliceMethods`] trait which is implemented for
/// [`Bound<'py, PySlice>`][Bound].
///
/// Only `isize` indices supported at the moment by the `PySlice` object.
#[repr(transparent)]
pub struct PySlice(PyAny);

pyobject_native_type!(
    PySlice,
    ffi::PySliceObject,
    pyobject_native_static_type_object!(ffi::PySlice_Type),
    #checkfunction=ffi::PySlice_Check
);

/// Return value from [`PySliceMethods::indices`].
#[derive(Debug, Eq, PartialEq)]
pub struct PySliceIndices {
    /// Start of the slice
    ///
    /// It can be -1 when the step is negative, otherwise it's non-negative.
    pub start: isize,
    /// End of the slice
    ///
    /// It can be -1 when the step is negative, otherwise it's non-negative.
    pub stop: isize,
    /// Increment to use when iterating the slice from `start` to `stop`.
    pub step: isize,
    /// The length of the slice calculated from the original input sequence.
    pub slicelength: usize,
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
    pub fn new(py: Python<'_>, start: isize, stop: isize, step: isize) -> Bound<'_, PySlice> {
        unsafe {
            ffi::PySlice_New(
                ffi::PyLong_FromSsize_t(start),
                ffi::PyLong_FromSsize_t(stop),
                ffi::PyLong_FromSsize_t(step),
            )
            .assume_owned(py)
            .cast_into_unchecked()
        }
    }

    /// Constructs a new full slice that is equivalent to `::`.
    pub fn full(py: Python<'_>) -> Bound<'_, PySlice> {
        unsafe {
            ffi::PySlice_New(ffi::Py_None(), ffi::Py_None(), ffi::Py_None())
                .assume_owned(py)
                .cast_into_unchecked()
        }
    }
}

/// Implementation of functionality for [`PySlice`].
///
/// These methods are defined for the `Bound<'py, PyTuple>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PySlice")]
pub trait PySliceMethods<'py>: crate::sealed::Sealed {
    /// Retrieves the start, stop, and step indices from the slice object,
    /// assuming a sequence of length `length`, and stores the length of the
    /// slice in its `slicelength` member.
    fn indices(&self, length: isize) -> PyResult<PySliceIndices>;
}

impl<'py> PySliceMethods<'py> for Bound<'py, PySlice> {
    fn indices(&self, length: isize) -> PyResult<PySliceIndices> {
        unsafe {
            let mut slicelength: isize = 0;
            let mut start: isize = 0;
            let mut stop: isize = 0;
            let mut step: isize = 0;
            let r = ffi::PySlice_GetIndicesEx(
                self.as_ptr(),
                length,
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
                    // non-negative isize should always fit into usize
                    slicelength: slicelength as _,
                })
            } else {
                Err(PyErr::fetch(self.py()))
            }
        }
    }
}

impl<'py> IntoPyObject<'py> for PySliceIndices {
    type Target = PySlice;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PySlice::new(py, self.start, self.stop, self.step))
    }
}

impl<'py> IntoPyObject<'py> for &PySliceIndices {
    type Target = PySlice;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PySlice::new(py, self.start, self.stop, self.step))
    }
}

impl<'py> TryFrom<Bound<'py, PyRange>> for Bound<'py, PySlice> {
    type Error = PyErr;

    fn try_from(range: Bound<'py, PyRange>) -> Result<Self, Self::Error> {
        Ok(PySlice::new(
            range.py(),
            range.start()?,
            range.stop()?,
            range.step()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyAnyMethods as _;

    #[test]
    fn test_py_slice_new() {
        Python::attach(|py| {
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
    fn test_py_slice_full() {
        Python::attach(|py| {
            let slice = PySlice::full(py);
            assert!(slice.getattr("start").unwrap().is_none(),);
            assert!(slice.getattr("stop").unwrap().is_none(),);
            assert!(slice.getattr("step").unwrap().is_none(),);
            assert_eq!(
                slice.indices(0).unwrap(),
                PySliceIndices {
                    start: 0,
                    stop: 0,
                    step: 1,
                    slicelength: 0,
                },
            );
            assert_eq!(
                slice.indices(42).unwrap(),
                PySliceIndices {
                    start: 0,
                    stop: 42,
                    step: 1,
                    slicelength: 42,
                },
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
