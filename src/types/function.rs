use crate::exceptions::PyValueError;
use crate::prelude::*;
use crate::{class, ffi, AsPyPointer, PyMethodDef, PyMethodType};

/// Represents a builtin Python function object.
#[repr(transparent)]
pub struct PyCFunction(PyAny);

pyobject_native_var_type!(PyCFunction, ffi::PyCFunction_Type, ffi::PyCFunction_Check);

impl PyCFunction {
    /// Create a new built-in function with keywords.
    pub fn new_with_keywords<'a>(
        fun: ffi::PyCFunctionWithKeywords,
        name: &str,
        doc: &str,
        module: Option<&'a PyModule>,
        py: Python<'a>,
    ) -> PyResult<&'a PyCFunction> {
        let fun = PyMethodType::PyCFunctionWithKeywords(fun);
        Self::new_(fun, name, doc, module, py)
    }

    /// Create a new built-in function without keywords.
    pub fn new<'a>(
        fun: ffi::PyCFunction,
        name: &str,
        doc: &str,
        module: Option<&'a PyModule>,
        py: Python<'a>,
    ) -> PyResult<&'a PyCFunction> {
        let fun = PyMethodType::PyCFunction(fun);
        Self::new_(fun, name, doc, module, py)
    }

    fn new_<'a>(
        fun: class::PyMethodType,
        name: &str,
        doc: &str,
        module: Option<&'a PyModule>,
        py: Python<'a>,
    ) -> PyResult<&'a PyCFunction> {
        let name = name.to_string();
        let name: &'static str = Box::leak(name.into_boxed_str());
        // this is ugly but necessary since `PyMethodDef::ml_doc` is &str and not `CStr`
        let doc = if doc.ends_with('\0') {
            doc.to_string()
        } else {
            format!("{}\0", doc)
        };
        let doc: &'static str = Box::leak(doc.into_boxed_str());
        let def = match &fun {
            PyMethodType::PyCFunction(_) => PyMethodDef {
                ml_name: name,
                ml_meth: fun,
                ml_flags: ffi::METH_VARARGS,
                ml_doc: doc,
            },
            PyMethodType::PyCFunctionWithKeywords(_) => PyMethodDef {
                ml_name: name,
                ml_meth: fun,
                ml_flags: ffi::METH_VARARGS | ffi::METH_KEYWORDS,
                ml_doc: doc,
            },
            _ => {
                return Err(PyValueError::py_err(
                    "Only PyCFunction and PyCFunctionWithKeywords are valid.",
                ))
            }
        };
        let def = def.as_method_def();
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
