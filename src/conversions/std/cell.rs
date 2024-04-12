use std::cell::Cell;

use crate::{
    conversion::IntoPyObject, types::any::PyAnyMethods, Bound, FromPyObject, IntoPy, PyAny,
    PyObject, PyResult, Python, ToPyObject,
};

impl<T: Copy + ToPyObject> ToPyObject for Cell<T> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.get().to_object(py)
    }
}

impl<T: Copy + IntoPy<PyObject>> IntoPy<PyObject> for Cell<T> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.get().into_py(py)
    }
}

impl<'py, Target, T: Copy + IntoPyObject<'py, Target>> IntoPyObject<'py, Target> for Cell<T> {
    type Error = T::Error;

    fn into_pyobj(self, py: Python<'py>) -> Result<Bound<'py, Target>, Self::Error> {
        self.get().into_pyobj(py)
    }
}

impl<'py, T: FromPyObject<'py>> FromPyObject<'py> for Cell<T> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        ob.extract().map(Cell::new)
    }
}
