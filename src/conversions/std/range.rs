use crate::exceptions::PyTypeError;
use crate::prelude::PyAnyMethods;
use crate::types::{PyInt, PyRange, PyRangeMethods};
use crate::{Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python};
use std::ops::{Range, RangeInclusive};

impl<'py, T> FromPyObject<'py> for Range<T>
where
    T: TryFrom<isize>,
    <T as TryFrom<isize>>::Error: Into<PyErr>,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let range = ob.downcast::<PyRange>()?;
        if range.step()? != 1 {
            return Err(PyTypeError::new_err(
                "Cannot convert a range with a step that is not 1",
            ));
        }
        let start = range.start()?.try_into().map_err(Into::into)?;
        let stop = range.stop()?.try_into().map_err(Into::into)?;
        Ok(start..stop)
    }
}

impl<'py, T> IntoPyObject<'py> for &Range<T>
where
    T: TryInto<isize> + Copy,
    <T as TryInto<isize>>::Error: Into<PyErr>,
{
    type Target = PyRange;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyRange::from_range(py, self)
    }
}

impl<'py, T> IntoPyObject<'py> for Range<T>
where
    T: IntoPyObject<'py, Target = PyInt> + Copy,
{
    type Target = PyRange;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyRange::new(py, self.start, self.end)
    }
}

impl<'py, T> IntoPyObject<'py> for RangeInclusive<T>
where
    T: TryInto<isize> + Copy,
    <T as TryInto<isize>>::Error: Into<PyErr>,
{
    type Target = PyRange;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyRange::from_range(py, &self)
    }
}

impl<'py, T> IntoPyObject<'py> for &RangeInclusive<T>
where
    T: TryInto<isize> + Copy,
    <T as TryInto<isize>>::Error: Into<PyErr>,
{
    type Target = PyRange;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyRange::from_range(py, self)
    }
}
