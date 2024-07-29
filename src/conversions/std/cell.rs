use std::cell::Cell;

use crate::{conversion::IntoPyObject, Borrowed, FromPyObject, PyAny, PyResult, Python};

impl<'py, T: Copy + IntoPyObject<'py>> IntoPyObject<'py> for Cell<T> {
    type Target = T::Target;
    type Output = T::Output;
    type Error = T::Error;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.get().into_pyobject(py)
    }
}

impl<'py, T: Copy + IntoPyObject<'py>> IntoPyObject<'py> for &Cell<T> {
    type Target = T::Target;
    type Output = T::Output;
    type Error = T::Error;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.get().into_pyobject(py)
    }
}

impl<'a, 'py, T: FromPyObject<'a, 'py>> FromPyObject<'a, 'py> for Cell<T> {
    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        ob.extract().map(Cell::new)
    }
}
