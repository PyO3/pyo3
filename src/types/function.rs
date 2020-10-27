use std::ffi::{CStr, CString};

use crate::derive_utils::PyFunctionArguments;
use crate::exceptions::PyValueError;
use crate::prelude::*;
use crate::{ffi, AsPyPointer, PyMethodDef, PyMethodType};

/// Represents a builtin Python function object.
#[repr(transparent)]
pub struct PyCFunction(PyAny);

pyobject_native_var_type!(PyCFunction, ffi::PyCFunction_Type, ffi::PyCFunction_Check);

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

impl PyCFunction {
    /// Create a new built-in function with keywords.
    ///
    /// See [raw_pycfunction] for documentation on how to get the `fun` argument.
    pub fn new_with_keywords<'a>(
        fun: ffi::PyCFunctionWithKeywords,
        name: &str,
        doc: &str,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a Self> {
        Self::internal_new(
            get_name(name)?,
            get_doc(doc)?,
            PyMethodType::PyCFunctionWithKeywords(fun),
            ffi::METH_VARARGS | ffi::METH_KEYWORDS,
            py_or_module,
        )
    }

    /// Create a new built-in function without keywords.
    pub fn new<'a>(
        fun: ffi::PyCFunction,
        name: &str,
        doc: &str,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a Self> {
        Self::internal_new(
            get_name(name)?,
            get_doc(doc)?,
            PyMethodType::PyCFunction(fun),
            ffi::METH_NOARGS,
            py_or_module,
        )
    }

    #[doc(hidden)]
    pub fn internal_new<'a>(
        name: &'static CStr,
        doc: &'static CStr,
        method_type: PyMethodType,
        flags: std::os::raw::c_int,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a Self> {
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
            (mod_ptr, name.as_ptr())
        } else {
            (std::ptr::null_mut(), std::ptr::null_mut())
        };

        unsafe {
            py.from_owned_ptr_or_err::<PyCFunction>(ffi::PyCFunction_NewEx(
                Box::into_raw(Box::new(def)),
                mod_ptr,
                module_name,
            ))
        }
    }
}

/// Represents a Python function object.
#[repr(transparent)]
pub struct PyFunction(PyAny);

#[cfg(not(Py_LIMITED_API))]
pyobject_native_var_type!(PyFunction, ffi::PyFunction_Type, ffi::PyFunction_Check);
