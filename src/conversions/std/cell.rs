use std::cell::Cell;

use crate::{
    conversion::IntoPyObject, types::any::PyAnyMethods, Bound, FromPyObject, PyAny, PyObject,
    PyResult, Python,
};

#[allow(deprecated)]
impl<T: Copy + crate::ToPyObject> crate::ToPyObject for Cell<T> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.get().to_object(py)
    }
}

#[allow(deprecated)]
impl<T: Copy + crate::IntoPy<PyObject>> crate::IntoPy<PyObject> for Cell<T> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.get().into_py(py)
    }
}

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

impl<'py, T: FromPyObject<'py>> FromPyObject<'py> for Cell<T> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        ob.extract().map(Cell::new)
    }
}
