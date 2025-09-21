use std::cell::UnsafeCell;

use crate::{
    ffi,
    ffi_ptr_ext::FfiPtrExt,
    py_result_ext::PyResultExt,
    types::{PyCFunction, PyModule, PyModuleMethods},
    Borrowed, Bound, PyResult, Python,
};

pub use crate::impl_::pymethods::PyMethodDef;

/// Wrapper around `ffi::PyMethodDef` suitable to use as a static variable for `#[pyfunction]` values.
///
/// The `UnsafeCell` is used because the Python interpreter consumes these as `*mut ffi::PyMethodDef`.
pub struct PyFunctionDef(UnsafeCell<ffi::PyMethodDef>);

// Safety: contents are only ever used by the Python interpreter, which uses global statics in this way.
unsafe impl Sync for PyFunctionDef {}

impl PyFunctionDef {
    pub const fn new(def: ffi::PyMethodDef) -> Self {
        Self(UnsafeCell::new(def))
    }

    pub const fn from_method_def(def: PyMethodDef) -> Self {
        Self::new(def.into_raw())
    }

    pub(crate) fn create_py_c_function<'py>(
        &'static self,
        py: Python<'py>,
        module: Option<&Bound<'py, PyModule>>,
    ) -> PyResult<Bound<'py, PyCFunction>> {
        // Safety: self is static
        unsafe { create_py_c_function(py, self.0.get(), module) }
    }
}

/// Trait to enable the use of `wrap_pyfunction` with both `Python` and `PyModule`,
/// and also to infer the return type of either `&'py PyCFunction` or `Bound<'py, PyCFunction>`.
pub trait WrapPyFunctionArg<'py, T> {
    fn wrap_pyfunction(self, function_def: &'static PyFunctionDef) -> PyResult<T>;
}

impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for Bound<'py, PyModule> {
    fn wrap_pyfunction(
        self,
        function_def: &'static PyFunctionDef,
    ) -> PyResult<Bound<'py, PyCFunction>> {
        function_def.create_py_c_function(self.py(), Some(&self))
    }
}

impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for &'_ Bound<'py, PyModule> {
    fn wrap_pyfunction(
        self,
        function_def: &'static PyFunctionDef,
    ) -> PyResult<Bound<'py, PyCFunction>> {
        function_def.create_py_c_function(self.py(), Some(self))
    }
}

impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for Borrowed<'_, 'py, PyModule> {
    fn wrap_pyfunction(
        self,
        function_def: &'static PyFunctionDef,
    ) -> PyResult<Bound<'py, PyCFunction>> {
        function_def.create_py_c_function(self.py(), Some(&self))
    }
}

impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for &'_ Borrowed<'_, 'py, PyModule> {
    fn wrap_pyfunction(
        self,
        function_def: &'static PyFunctionDef,
    ) -> PyResult<Bound<'py, PyCFunction>> {
        function_def.create_py_c_function(self.py(), Some(self))
    }
}

impl<'py> WrapPyFunctionArg<'py, Bound<'py, PyCFunction>> for Python<'py> {
    fn wrap_pyfunction(
        self,
        function_def: &'static PyFunctionDef,
    ) -> PyResult<Bound<'py, PyCFunction>> {
        function_def.create_py_c_function(self, None)
    }
}

/// Creates a `PyCFunction` object from a `PyMethodDef`.
///
/// # Safety
///
/// The `method_def` pointer must be valid for the lifetime of the returned `PyCFunction`
/// (effectively, it must be a static variable).
pub unsafe fn create_py_c_function<'py>(
    py: Python<'py>,
    method_def: *mut ffi::PyMethodDef,
    module: Option<&Bound<'py, PyModule>>,
) -> PyResult<Bound<'py, PyCFunction>> {
    let (mod_ptr, module_name) = if let Some(m) = module {
        let mod_ptr = m.as_ptr();
        (mod_ptr, Some(m.name()?))
    } else {
        (std::ptr::null_mut(), None)
    };

    let module_name_ptr = module_name
        .as_ref()
        .map_or(std::ptr::null_mut(), Bound::as_ptr);

    unsafe {
        ffi::PyCFunction_NewEx(method_def, mod_ptr, module_name_ptr)
            .assume_owned_or_err(py)
            .cast_into_unchecked()
    }
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    #[test]
    fn test_wrap_pyfunction_forms() {
        use crate::types::{PyAnyMethods, PyModule};
        use crate::{wrap_pyfunction, Python};

        #[crate::pyfunction(crate = "crate")]
        fn f() {}

        Python::attach(|py| {
            let module = PyModule::new(py, "test_wrap_pyfunction_forms").unwrap();

            let func = wrap_pyfunction!(f, module.clone()).unwrap();
            func.call0().unwrap();

            let func = wrap_pyfunction!(f, &module).unwrap();
            func.call0().unwrap();

            let module_borrowed = module.as_borrowed();

            let func = wrap_pyfunction!(f, module_borrowed).unwrap();
            func.call0().unwrap();

            let func = wrap_pyfunction!(f, &module_borrowed).unwrap();
            func.call0().unwrap();

            let func = wrap_pyfunction!(f, py).unwrap();
            func.call0().unwrap();
        });
    }
}
