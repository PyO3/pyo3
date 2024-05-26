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

// For Python<'py>, only the GIL Ref form exists to avoid causing type inference to kick in.
// The `wrap_pyfunction_bound!` macro is needed for the Bound form.
#[cfg(feature = "gil-refs")]
impl<'py> WrapPyFunctionArg<'py, &'py PyCFunction> for Python<'py> {
    fn wrap_pyfunction(self, method_def: &PyMethodDef) -> PyResult<&'py PyCFunction> {
        PyCFunction::internal_new(self, method_def, None).map(Bound::into_gil_ref)
    }
}

#[cfg(not(feature = "gil-refs"))]
impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for Python<'py> {
    fn wrap_pyfunction(self, method_def: &PyMethodDef) -> PyResult<Bound<'py, PyCFunction>> {
        PyCFunction::internal_new(self, method_def, None)
    }
}

#[cfg(feature = "gil-refs")]
impl<'py> WrapPyFunctionArg<'py, &'py PyCFunction> for &'py PyModule {
    fn wrap_pyfunction(self, method_def: &PyMethodDef) -> PyResult<&'py PyCFunction> {
        use crate::PyNativeType;
        PyCFunction::internal_new(self.py(), method_def, Some(&self.as_borrowed()))
            .map(Bound::into_gil_ref)
    }
}

/// Helper for `wrap_pyfunction_bound!` to guarantee return type of `Bound<'py, PyCFunction>`.
pub struct OnlyBound<T>(pub T);

impl<'py, T> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for OnlyBound<T>
where
    T: WrapPyFunctionArg<'py, Bound<'py, PyCFunction>>,
{
    fn wrap_pyfunction(self, method_def: &PyMethodDef) -> PyResult<Bound<'py, PyCFunction>> {
        WrapPyFunctionArg::wrap_pyfunction(self.0, method_def)
    }
}

#[cfg(feature = "gil-refs")]
impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for OnlyBound<Python<'py>> {
    fn wrap_pyfunction(self, method_def: &PyMethodDef) -> PyResult<Bound<'py, PyCFunction>> {
        PyCFunction::internal_new(self.0, method_def, None)
    }
}
