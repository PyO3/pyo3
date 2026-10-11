use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

create_exception!(pytests.exception, MyValueError, PyValueError);

#[cfg(any(not(Py_LIMITED_API), Py_3_12))]
#[pyclass(extends = PyValueError)]
pub struct MyValueErrorClass;

#[cfg(any(not(Py_LIMITED_API), Py_3_12))]
#[pymethods]
impl MyValueErrorClass {
    #[new]
    fn new() -> PyClassInitializer<Self> {
        PyClassInitializer::from(MyValueErrorClass)
    }
}

#[pymodule(gil_used = false)]
pub mod exception {
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;

    #[pymodule_export]
    use super::MyValueError;

    #[cfg(any(not(Py_LIMITED_API), Py_3_12))]
    #[pymodule_export]
    use super::MyValueErrorClass;

    #[pyfunction]
    fn raise_my_value_error() -> PyResult<()> {
        Err(MyValueError::new_err("error"))
    }

    #[pyfunction]
    fn return_value_error<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyValueError>> {
        Ok(PyValueError::new_err("error")
            .into_pyobject(py)?
            .cast_into()?)
    }

    #[pyfunction]
    fn return_my_value_error<'py>(py: Python<'py>) -> PyResult<Bound<'py, MyValueError>> {
        Ok(MyValueError::new_err("error")
            .into_pyobject(py)?
            .cast_into()?)
    }

    #[pyfunction]
    fn return_pyerr() -> PyErr {
        MyValueError::new_err("error")
    }
}
