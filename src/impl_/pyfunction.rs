use crate::{
    derive_utils::PyFunctionArguments, impl_::pymethods::PyMethodDef, types::PyCFunction, PyResult,
};

pub trait PyFunctionDef {
    const DEF: crate::PyMethodDef;
}

pub fn wrap_pyfunction<'a>(
    method_def: PyMethodDef,
    args: impl Into<PyFunctionArguments<'a>>,
) -> PyResult<&'a PyCFunction> {
    PyCFunction::internal_new(method_def, args.into())
}
