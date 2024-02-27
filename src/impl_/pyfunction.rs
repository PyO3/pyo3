use crate::{
    derive_utils::PyFunctionArguments,
    types::{PyCFunction, PyModule},
    Borrowed, Bound, PyResult, Python,
};

pub use crate::impl_::pymethods::PyMethodDef;

pub fn wrap_pyfunction<'a>(
    method_def: &PyMethodDef,
    py_or_module: impl Into<PyFunctionArguments<'a>>,
) -> PyResult<&'a PyCFunction> {
    PyCFunction::internal_new(method_def, py_or_module.into()).map(|x| x.into_gil_ref())
}

/// Trait to enable the use of `wrap_pyfunction` with both `Python` and `PyModule`.
pub trait WrapPyFunctionArg<'py> {
    fn py(&self) -> Python<'py>;
    fn module(&self) -> Option<&Bound<'py, PyModule>>;
}

impl<'py> WrapPyFunctionArg<'py> for Bound<'py, PyModule> {
    fn py(&self) -> Python<'py> {
        Bound::py(self)
    }
    fn module(&self) -> Option<&Bound<'py, PyModule>> {
        Some(self)
    }
}

impl<'py> WrapPyFunctionArg<'py> for &'_ Bound<'py, PyModule> {
    fn py(&self) -> Python<'py> {
        Bound::py(self)
    }
    fn module(&self) -> Option<&Bound<'py, PyModule>> {
        Some(self)
    }
}

impl<'py> WrapPyFunctionArg<'py> for Borrowed<'_, 'py, PyModule> {
    fn py(&self) -> Python<'py> {
        Bound::py(self)
    }
    fn module(&self) -> Option<&Bound<'py, PyModule>> {
        Some(self)
    }
}

impl<'py> WrapPyFunctionArg<'py> for &'_ Borrowed<'_, 'py, PyModule> {
    fn py(&self) -> Python<'py> {
        Bound::py(self)
    }
    fn module(&self) -> Option<&Bound<'py, PyModule>> {
        Some(self)
    }
}

impl<'py> WrapPyFunctionArg<'py> for Python<'py> {
    fn py(&self) -> Python<'py> {
        *self
    }
    fn module(&self) -> Option<&Bound<'py, PyModule>> {
        None
    }
}

pub fn wrap_pyfunction_bound<'py>(
    method_def: &PyMethodDef,
    arg: impl WrapPyFunctionArg<'py>,
) -> PyResult<Bound<'py, PyCFunction>> {
    PyCFunction::internal_new_bound(arg.py(), method_def, arg.module())
}
