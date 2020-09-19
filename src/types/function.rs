use std::ffi::{CStr, CString};

use crate::derive_utils::PyFunctionArguments;
use crate::exceptions::PyValueError;
use crate::prelude::*;
use crate::{class, ffi, AsPyPointer, PyMethodType};

/// Represents a builtin Python function object.
#[repr(transparent)]
pub struct PyCFunction(PyAny);

pyobject_native_var_type!(PyCFunction, ffi::PyCFunction_Type, ffi::PyCFunction_Check);

impl PyCFunction {
    /// Create a new built-in function with keywords.
    ///
    /// See [raw_pycfunction] for documentation on how to get the `fun` argument.
    pub fn new_with_keywords<'a>(
        fun: ffi::PyCFunctionWithKeywords,
        name: &str,
        doc: &'static str,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a PyCFunction> {
        let fun = PyMethodType::PyCFunctionWithKeywords(fun);
        Self::new_(fun, name, doc, py_or_module)
    }

    /// Create a new built-in function without keywords.
    pub fn new<'a>(
        fun: ffi::PyCFunction,
        name: &str,
        doc: &'static str,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a PyCFunction> {
        let fun = PyMethodType::PyCFunction(fun);
        Self::new_(fun, name, doc, py_or_module)
    }

    fn new_<'a>(
        fun: class::PyMethodType,
        name: &str,
        doc: &'static str,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a PyCFunction> {
        let (py, module) = py_or_module.into_py_and_maybe_module();
        let doc: &'static CStr = CStr::from_bytes_with_nul(doc.as_bytes())
            .map_err(|_| PyValueError::new_err("docstring must end with NULL byte."))?;
        let name = CString::new(name.as_bytes()).map_err(|_| {
            PyValueError::new_err("Function name cannot contain contain NULL byte.")
        })?;
        let def = match fun {
            PyMethodType::PyCFunction(fun) => ffi::PyMethodDef {
                ml_name: name.into_raw() as _,
                ml_meth: Some(fun),
                ml_flags: ffi::METH_VARARGS,
                ml_doc: doc.as_ptr() as _,
            },
            PyMethodType::PyCFunctionWithKeywords(fun) => ffi::PyMethodDef {
                ml_name: name.into_raw() as _,
                ml_meth: Some(unsafe { std::mem::transmute(fun) }),
                ml_flags: ffi::METH_VARARGS | ffi::METH_KEYWORDS,
                ml_doc: doc.as_ptr() as _,
            },
            _ => {
                return Err(PyValueError::new_err(
                    "Only PyCFunction and PyCFunctionWithKeywords are valid.",
                ))
            }
        };
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

pyobject_native_var_type!(PyFunction, ffi::PyFunction_Type, ffi::PyFunction_Check);
