use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

#[pyclass(extends=PyException, subclass)]
struct CustomException {}

#[pymethods]
impl CustomException {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &PyTuple, _kwargs: Option<&PyDict>) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CustomException {})
    }
}

#[pyclass(extends=CustomException, subclass)]
struct ExceptionSubclassA {}

#[pymethods]
impl ExceptionSubclassA {
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn new(args: &PyTuple, kwargs: Option<&PyDict>) -> PyClassInitializer<Self> {
        CustomException::new(args, kwargs).add_subclass(Self {})
    }
}

#[pyclass(extends=ExceptionSubclassA, subclass)]
struct ExceptionSubclassAChild {}

#[pymethods]
impl ExceptionSubclassAChild {
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn new(args: &PyTuple, kwargs: Option<&PyDict>) -> PyClassInitializer<Self> {
        ExceptionSubclassA::new(args, kwargs).add_subclass(Self {})
    }
}

#[pyclass(extends=CustomException)]
struct ExceptionSubclassB {}

#[pymethods]
impl ExceptionSubclassB {
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn new(args: &PyTuple, kwargs: Option<&PyDict>) -> PyClassInitializer<Self> {
        CustomException::new(args, kwargs).add_subclass(Self {})
    }
}

#[pyfunction]
fn do_something(op: &str) -> PyResult<()> {
    match op {
        "success" => Ok(()),

        "subclass_a" => Err(PyErr::new::<ExceptionSubclassA, _>("subclass_a")),
        "subclass_a_child" => Err(PyErr::new::<ExceptionSubclassAChild, _>("subclass_a_child")),
        "subclass_b" => Err(PyErr::new::<ExceptionSubclassB, _>("subclass_b")),
        _ => Err(PyErr::new::<CustomException, _>(format!(
            "unknown op `{}`",
            op
        ))),
    }
}

#[pymodule]
pub fn exceptions(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<CustomException>()?;
    m.add_class::<ExceptionSubclassA>()?;
    m.add_class::<ExceptionSubclassAChild>()?;
    m.add_class::<ExceptionSubclassB>()?;
    m.add_function(wrap_pyfunction!(do_something, m)?)?;
    Ok(())
}
