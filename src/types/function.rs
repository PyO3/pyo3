use crate::derive_utils::PyFunctionArguments;
use crate::exceptions::PyValueError;
use crate::prelude::*;
use crate::{
    class::methods::{self, PyMethodDef},
    ffi, AsPyPointer,
};

/// Represents a builtin Python function object.
#[repr(transparent)]
pub struct PyCFunction(PyAny);

pyobject_native_type_core!(PyCFunction, ffi::PyCFunction_Type, #checkfunction=ffi::PyCFunction_Check);

impl PyCFunction {
    /// Create a new built-in function with keywords.
    pub fn new_with_keywords<'a>(
        fun: ffi::PyCFunctionWithKeywords,
        name: &'static str,
        doc: &'static str,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a Self> {
        Self::internal_new(
            PyMethodDef::cfunction_with_keywords(name, methods::PyCFunctionWithKeywords(fun), doc),
            py_or_module,
        )
    }

    /// Create a new built-in function without keywords.
    pub fn new<'a>(
        fun: ffi::PyCFunction,
        name: &'static str,
        doc: &'static str,
        py_or_module: PyFunctionArguments<'a>,
    ) -> PyResult<&'a Self> {
        Self::internal_new(
            PyMethodDef::noargs(name, methods::PyCFunction(fun), doc),
            py_or_module,
        )
    }

    #[doc(hidden)]
    pub fn internal_new(
        method_def: PyMethodDef,
        py_or_module: PyFunctionArguments,
    ) -> PyResult<&Self> {
        let (py, module) = py_or_module.into_py_and_maybe_module();
        let def = method_def
            .as_method_def()
            .map_err(|err| PyValueError::new_err(err.0))?;
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
pyobject_native_type_core!(PyFunction, ffi::PyFunction_Type, #checkfunction=ffi::PyFunction_Check);
