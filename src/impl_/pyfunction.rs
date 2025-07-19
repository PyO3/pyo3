use crate::{
    types::{PyCFunction, PyModule},
    Borrowed, Bound, PyResult, Python,
};

pub use crate::impl_::pymethods::PyMethodDef;

/// Trait to enable the use of `wrap_pyfunction` with both `Python` and `PyModule`,
/// and also to infer the return type of either `&'py PyCFunction` or `Bound<'py, PyCFunction>`.
pub trait WrapPyFunctionArg<'py, T> {
    fn wrap_pyfunction(self, method_def: &PyMethodDef) -> PyResult<T>;
}

impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for Bound<'py, PyModule> {
    fn wrap_pyfunction(self, method_def: &PyMethodDef) -> PyResult<Bound<'py, PyCFunction>> {
        PyCFunction::internal_new(self.py(), method_def, Some(&self))
    }
}

impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for &'_ Bound<'py, PyModule> {
    fn wrap_pyfunction(self, method_def: &PyMethodDef) -> PyResult<Bound<'py, PyCFunction>> {
        PyCFunction::internal_new(self.py(), method_def, Some(self))
    }
}

impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for Borrowed<'_, 'py, PyModule> {
    fn wrap_pyfunction(self, method_def: &PyMethodDef) -> PyResult<Bound<'py, PyCFunction>> {
        PyCFunction::internal_new(self.py(), method_def, Some(&self))
    }
}

impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for &'_ Borrowed<'_, 'py, PyModule> {
    fn wrap_pyfunction(self, method_def: &PyMethodDef) -> PyResult<Bound<'py, PyCFunction>> {
        PyCFunction::internal_new(self.py(), method_def, Some(self))
    }
}

impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for Python<'py> {
    fn wrap_pyfunction(self, method_def: &PyMethodDef) -> PyResult<Bound<'py, PyCFunction>> {
        PyCFunction::internal_new(self, method_def, None)
    }
}
