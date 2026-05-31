use pyo3::prelude::*;
use pyo3::PyTraverseError;
use pyo3::PyVisit;

#[pyclass]
struct TraverseTriesToTakePyRef {}

#[pymethods]
impl TraverseTriesToTakePyRef {
    fn __traverse__(_slf: PyRef<Self>, _visit: PyVisit) -> Result<(), PyTraverseError> {
//~^ ERROR: __traverse__ may not take a receiver other than `&self`. Usually, an implementation of `__traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError>` should do nothing but calls to `visit.call`. Most importantly, safe access to the Python interpreter is prohibited inside implementations of `__traverse__`, i.e. `Python::attach` will panic.
        Ok(())
    }
}

#[pyclass]
struct TraverseTriesToTakePyRefMut {}

#[pymethods]
impl TraverseTriesToTakePyRefMut {
    fn __traverse__(_slf: PyRefMut<Self>, _visit: PyVisit) -> Result<(), PyTraverseError> {
//~^ ERROR: __traverse__ may not take a receiver other than `&self`. Usually, an implementation of `__traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError>` should do nothing but calls to `visit.call`. Most importantly, safe access to the Python interpreter is prohibited inside implementations of `__traverse__`, i.e. `Python::attach` will panic.
        Ok(())
    }
}

#[pyclass]
struct TraverseTriesToTakeBound {}

#[pymethods]
impl TraverseTriesToTakeBound {
    fn __traverse__(_slf: Bound<'_, Self>, _visit: PyVisit) -> Result<(), PyTraverseError> {
//~^ ERROR: __traverse__ may not take a receiver other than `&self`. Usually, an implementation of `__traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError>` should do nothing but calls to `visit.call`. Most importantly, safe access to the Python interpreter is prohibited inside implementations of `__traverse__`, i.e. `Python::attach` will panic.
        Ok(())
    }
}

#[pyclass]
struct TraverseTriesToTakeMutSelf {}

#[pymethods]
impl TraverseTriesToTakeMutSelf {
    fn __traverse__(&mut self, _visit: PyVisit) -> Result<(), PyTraverseError> {
//~^ ERROR: __traverse__ may not take a receiver other than `&self`. Usually, an implementation of `__traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError>` should do nothing but calls to `visit.call`. Most importantly, safe access to the Python interpreter is prohibited inside implementations of `__traverse__`, i.e. `Python::attach` will panic.
        Ok(())
    }
}

#[pyclass]
struct TraverseTriesToTakeSelf {}

#[pymethods]
impl TraverseTriesToTakeSelf {
    fn __traverse__(&self, _visit: PyVisit) -> Result<(), PyTraverseError> {
        Ok(())
    }
}

#[pyclass]
struct Class;

#[pymethods]
impl Class {
    fn __traverse__(&self, _py: Python<'_>, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
//~^ ERROR: __traverse__ may not take `Python`. Usually, an implementation of `__traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError>` should do nothing but calls to `visit.call`. Most importantly, safe access to the Python interpreter is prohibited inside implementations of `__traverse__`, i.e. `Python::attach` will panic.
        Ok(())
    }

    fn __clear__(&mut self) {}
}

fn main() {}
