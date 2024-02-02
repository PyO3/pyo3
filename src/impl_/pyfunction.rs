use crate::{
    derive_utils::{PyFunctionArguments, PyFunctionArgumentsBound},
    types::PyCFunction,
    Bound, PyResult,
};

pub use crate::impl_::pymethods::PyMethodDef;

pub fn _wrap_pyfunction<'a>(
    method_def: &PyMethodDef,
    py_or_module: impl Into<PyFunctionArguments<'a>>,
) -> PyResult<&'a PyCFunction> {
    PyCFunction::internal_new(method_def, py_or_module.into())
}

pub fn _wrap_pyfunction_bound<'a, 'py: 'a>(
    method_def: &PyMethodDef,
    py_or_module: impl Into<PyFunctionArgumentsBound<'a, 'py>>,
) -> PyResult<Bound<'py, PyCFunction>> {
    PyCFunction::internal_new_bound(method_def, py_or_module.into())
}
