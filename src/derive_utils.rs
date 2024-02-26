//! Functionality for the code generated by the derive backend

use crate::impl_::pymethods::PyMethodDef;
use crate::{
    types::{PyCFunction, PyModule},
    PyResult, Python,
};

/// Enum to abstract over the arguments of Python function wrappers.
pub enum PyFunctionArguments<'a> {
    Python(Python<'a>),
    PyModule(&'a PyModule),
}

impl<'a> PyFunctionArguments<'a> {
    pub fn into_py_and_maybe_module(self) -> (Python<'a>, Option<&'a PyModule>) {
        match self {
            PyFunctionArguments::Python(py) => (py, None),
            PyFunctionArguments::PyModule(module) => {
                let py = module.py();
                (py, Some(module))
            }
        }
    }

    pub fn wrap_pyfunction(self, method_def: &'a PyMethodDef) -> PyResult<&PyCFunction> {
        Ok(PyCFunction::internal_new(method_def, self)?.into_gil_ref())
    }
}

impl<'a> From<Python<'a>> for PyFunctionArguments<'a> {
    fn from(py: Python<'a>) -> PyFunctionArguments<'a> {
        PyFunctionArguments::Python(py)
    }
}

impl<'a> From<&'a PyModule> for PyFunctionArguments<'a> {
    fn from(module: &'a PyModule) -> PyFunctionArguments<'a> {
        PyFunctionArguments::PyModule(module)
    }
}
