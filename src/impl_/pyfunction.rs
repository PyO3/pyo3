use crate::{derive_utils::PyFunctionArguments, types::PyCFunction, PyResult};

pub use crate::impl_::pymethods::PyMethodDef;

pub trait PyFunctionDef {
    const DEF: crate::PyMethodDef;
}

pub fn wrap_pyfunction<'a>(
    method_def: PyMethodDef,
    args: impl Into<PyFunctionArguments<'a>>,
) -> PyResult<&'a PyCFunction> {
    PyCFunction::internal_new(method_def, args.into())
}
