use crate::exceptions::PyTypeError;
use crate::sealed::Sealed;
use crate::types::{PyAnyMethods, PyInt};
use crate::{ffi, Bound, IntoPyObject, PyAny, PyErr, PyResult, PyTypeInfo, Python};
use std::ops::RangeBounds;

/// Represents a Python `range`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyTange>`][crate::Py] or [`Bound<'py, PyRange>`][Bound].
///
/// For APIs available on `range` objects, see the [`PyRangeMethods`] trait which is implemented for
/// [`Bound<'py, PyRange>`][Bound].
#[repr(transparent)]
pub struct PyRange(PyAny);

pyobject_native_type_core!(PyRange, pyobject_native_static_type_object!(ffi::PyRange_Type), #checkfunction=ffi::PyRange_Check);

impl<'py> PyRange {
    /// Creates a new Python `range` object with a default step of 1.
    pub fn new<T>(py: Python<'py>, start: T, stop: T) -> PyResult<Bound<'py, Self>>
    where
        T: IntoPyObject<'py, Target = PyInt>,
    {
        Self::new_with_step(py, start, stop, 1)
    }

    /// Creates a new Python `range` object with a specified step.
    pub fn new_with_step<T>(
        py: Python<'py>,
        start: T,
        stop: T,
        step: isize,
    ) -> PyResult<Bound<'py, Self>>
    where
        T: IntoPyObject<'py, Target = PyInt>,
    {
        unsafe {
            Ok(Self::type_object(py)
                .call1((start, stop, step))?
                .downcast_into_unchecked())
        }
    }

    /// Creates a new Python `range` object from a Rust range.
    pub fn from_range<T, R: RangeBounds<T>>(
        py: Python<'py>,
        range: &R,
    ) -> PyResult<Bound<'py, Self>>
    where
        T: TryInto<isize> + Copy,
        <T as TryInto<isize>>::Error: Into<PyErr>,
    {
        use std::ops::Bound::*;

        let start = match range.start_bound() {
            Included(value) => (*value).try_into().map_err(Into::into)?,
            Excluded(value) => (*value).try_into().map_err(Into::into)? + 1,
            Unbounded => {
                return Err(PyTypeError::new_err(
                    "Cannot convert range with unbounded start",
                ))
            }
        };

        let stop = match range.end_bound() {
            Included(value) => (*value).try_into().map_err(Into::into)? + 1,
            Excluded(value) => (*value).try_into().map_err(Into::into)?,
            Unbounded => {
                return Err(PyTypeError::new_err(
                    "Cannot convert range with unbounded end",
                ))
            }
        };

        PyRange::new(py, start, stop)
    }
}

/// Implementation of functionality for [`PyRange`].
///
/// These methods are defined for the `Bound<'py, PyRange>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyRange")]
pub trait PyRangeMethods<'py>: Sealed {
    /// Returns the start of the range.
    fn start(&self) -> PyResult<isize>;

    /// Returns the exclusive end of the range.
    fn stop(&self) -> PyResult<isize>;

    /// Returns the step of the range.
    fn step(&self) -> PyResult<isize>;
}

impl<'py> PyRangeMethods<'py> for Bound<'py, PyRange> {
    fn start(&self) -> PyResult<isize> {
        self.getattr(intern!(self.py(), "start"))?.extract()
    }

    fn stop(&self) -> PyResult<isize> {
        self.getattr(intern!(self.py(), "stop"))?.extract()
    }

    fn step(&self) -> PyResult<isize> {
        self.getattr(intern!(self.py(), "step"))?.extract()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    #[test]
    fn test_py_range_new() {
        Python::with_gil(|py| {
            let range = PyRange::new(py, isize::MIN, isize::MAX).unwrap();
            assert_eq!(range.start().unwrap(), isize::MIN);
            assert_eq!(range.stop().unwrap(), isize::MAX);
            assert_eq!(range.step().unwrap(), 1);
        });
    }

    #[test]
    fn test_range_into_py() {
        Python::with_gil(|py| {
            let mut range = 0..10;
            let py_range = (&range).into_pyobject(py).unwrap();
            for i in py_range.try_iter().unwrap() {
                assert_eq!(i.unwrap().extract::<i32>().unwrap(), range.next().unwrap());
            }
            assert_eq!(range.next(), None);
        })
    }

    #[test]
    fn test_range_from_python() {
        Python::with_gil(|py| {
            let py_range = PyRange::new(py, 0, 10).unwrap();
            let range: Range<i32> = py_range.extract().unwrap();
            assert_eq!(range.start, 0);
            assert_eq!(range.end, 10);
        });
    }

    #[test]
    fn test_range_from_python_with_step() {
        Python::with_gil(|py| {
            let py_range = PyRange::new_with_step(py, 0, 10, 2).unwrap();
            assert!(py_range.extract::<Range<i32>>().is_err());
        });
    }

    #[test]
    fn test_range_from_python_too_big() {
        Python::with_gil(|py| {
            let py_range = PyRange::new(py, 0, i32::MAX).unwrap();
            assert!(py_range.extract::<Range<i8>>().is_err());
        });
    }
}
