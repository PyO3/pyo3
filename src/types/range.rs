use crate::sealed::Sealed;
use crate::types::PyAnyMethods;
use crate::{ffi, Bound, PyAny, PyResult, PyTypeInfo, Python};

/// Represents a Python `range`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyRange>`][crate::Py] or [`Bound<'py, PyRange>`][Bound].
///
/// For APIs available on `range` objects, see the [`PyRangeMethods`] trait which is implemented for
/// [`Bound<'py, PyRange>`][Bound].
#[repr(transparent)]
pub struct PyRange(PyAny);

pyobject_native_type_core!(PyRange, pyobject_native_static_type_object!(ffi::PyRange_Type), #checkfunction=ffi::PyRange_Check);

impl<'py> PyRange {
    /// Creates a new Python `range` object with a default step of 1.
    pub fn new(py: Python<'py>, start: isize, stop: isize) -> PyResult<Bound<'py, Self>> {
        Self::new_with_step(py, start, stop, 1)
    }

    /// Creates a new Python `range` object with a specified step.
    pub fn new_with_step(
        py: Python<'py>,
        start: isize,
        stop: isize,
        step: isize,
    ) -> PyResult<Bound<'py, Self>> {
        unsafe {
            Ok(Self::type_object(py)
                .call1((start, stop, step))?
                .cast_into_unchecked())
        }
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

    #[test]
    fn test_py_range_new() {
        Python::attach(|py| {
            let range = PyRange::new(py, isize::MIN, isize::MAX).unwrap();
            assert_eq!(range.start().unwrap(), isize::MIN);
            assert_eq!(range.stop().unwrap(), isize::MAX);
            assert_eq!(range.step().unwrap(), 1);
        });
    }

    #[test]
    fn test_py_range_new_with_step() {
        Python::attach(|py| {
            let range = PyRange::new_with_step(py, 1, 10, 2).unwrap();
            assert_eq!(range.start().unwrap(), 1);
            assert_eq!(range.stop().unwrap(), 10);
            assert_eq!(range.step().unwrap(), 2);
        });
    }
}
