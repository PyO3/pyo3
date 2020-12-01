use std::ffi::{CStr, CString};

use crate::derive_utils::PyFunctionArguments;
use crate::exceptions::PyValueError;
use crate::{
    ffi,
    objects::PyAny,
    types::{CFunction, Function},
    AsPyPointer, PyMethodDef, PyMethodType, PyResult, IntoPy,
};

/// Represents a builtin Python function object.
#[repr(transparent)]
pub struct PyCFunction<'py>(pub(crate) PyAny<'py>);
pyo3_native_object!(PyCFunction<'py>, CFunction, 'py);

fn get_name(name: &str) -> PyResult<&'static CStr> {
    let cstr = CString::new(name)
        .map_err(|_| PyValueError::new_err("Function name cannot contain contain NULL byte."))?;
    Ok(Box::leak(cstr.into_boxed_c_str()))
}

fn get_doc(doc: &str) -> PyResult<&'static CStr> {
    let cstr = CString::new(doc)
        .map_err(|_| PyValueError::new_err("Document cannot contain contain NULL byte."))?;
    Ok(Box::leak(cstr.into_boxed_c_str()))
}

impl<'py> PyCFunction<'py> {
    /// Create a new built-in function with keywords.
    ///
    /// See [raw_pycfunction] for documentation on how to get the `fun` argument.
    pub fn new_with_keywords(
        fun: ffi::PyCFunctionWithKeywords,
        name: &str,
        doc: &str,
        py_or_module: PyFunctionArguments<'py>,
    ) -> PyResult<Self> {
        Self::internal_new(
            get_name(name)?,
            get_doc(doc)?,
            PyMethodType::PyCFunctionWithKeywords(fun),
            ffi::METH_VARARGS | ffi::METH_KEYWORDS,
            py_or_module,
        )
    }

    /// Create a new built-in function without keywords.
    pub fn new(
        fun: ffi::PyCFunction,
        name: &str,
        doc: &str,
        py_or_module: PyFunctionArguments<'py>,
    ) -> PyResult<Self> {
        Self::internal_new(
            get_name(name)?,
            get_doc(doc)?,
            PyMethodType::PyCFunction(fun),
            ffi::METH_NOARGS,
            py_or_module,
        )
    }

    #[doc(hidden)]
    pub fn internal_new(
        name: &'static CStr,
        doc: &'static CStr,
        method_type: PyMethodType,
        flags: std::os::raw::c_int,
        py_or_module: PyFunctionArguments<'py>,
    ) -> PyResult<Self> {
        let (py, module) = py_or_module.into_py_and_maybe_module();
        let method_def = PyMethodDef {
            ml_name: name,
            ml_meth: method_type,
            ml_flags: flags,
            ml_doc: doc,
        };
        let def = method_def.as_method_def();
        let (mod_ptr, module_name) = if let Some(m) = module {
            let mod_ptr = m.as_ptr();
            let name = m.name()?.into_py(py);
            (mod_ptr, Some(name))
        } else {
            (std::ptr::null_mut(), None)
        };

        unsafe {
            PyAny::from_raw_or_fetch_err(py, ffi::PyCFunction_NewEx(
                Box::into_raw(Box::new(def)),
                mod_ptr,
                module_name.as_ptr(),
            )).map(Self)
        }
    }
}

/// Represents a Python function object.
#[repr(transparent)]
pub struct PyFunction<'py>(pub(crate) PyAny<'py>);

#[cfg(not(Py_LIMITED_API))]
pyo3_native_object!(PyFunction<'py>, Function, 'py);
