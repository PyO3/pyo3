use crate::{derive_utils::PyFunctionArguments, types::PyCFunction, PyResult};

pub use crate::impl_::pymethods::PyMethodDef;

pub fn wrap_pyfunction_impl<'a>(
    method_def: &PyMethodDef,
    py_or_module: impl Into<PyFunctionArguments<'a>>,
) -> PyResult<&'a PyCFunction> {
    PyCFunction::internal_new(method_def, py_or_module.into())
}
