use super::PyAnyMethods as _;
use super::PyDict;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::{ffi, Bound, PyAny, PyErr, PyResult, Python};
use std::ffi::CStr;

/// Represents a Python code object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyCode>`][crate::Py] or [`Bound<'py, PyCode>`][crate::Bound].
#[repr(transparent)]
pub struct PyCode(PyAny);

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
pyobject_native_type_core!(
    PyCode,
    pyobject_native_static_type_object!(ffi::PyCode_Type),
    #checkfunction=ffi::PyCode_Check
);

#[cfg(any(Py_LIMITED_API, PyPy))]
pyobject_native_type_named!(PyCode);

#[cfg(any(Py_LIMITED_API, PyPy))]
impl crate::PyTypeCheck for PyCode {
    const NAME: &'static str = "PyCode";
    #[cfg(feature = "experimental-inspect")]
    const PYTHON_TYPE: &'static str = "types.CodeType";

    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        let py = object.py();
        static TYPE: crate::sync::PyOnceLock<crate::Py<super::PyType>> =
            crate::sync::PyOnceLock::new();

        TYPE.import(py, "types", "CodeType")
            .and_then(|ty| object.is_instance(ty))
            .unwrap_or_default()
    }
}

/// Compilation mode of [`PyCode::compile`]
pub enum PyCodeInput {
    /// Python grammar for isolated expressions
    Eval,
    /// Python grammar for sequences of statements as read from a file
    File,
}

impl PyCode {
    /// Compiles code in the given context.
    ///
    /// `input` decides whether `code` is treated as
    /// - [`PyCodeInput::Eval`]: an isolated expression
    /// - [`PyCodeInput::File`]: a sequence of statements
    pub fn compile<'py>(
        py: Python<'py>,
        code: &CStr,
        filename: &CStr,
        input: PyCodeInput,
    ) -> PyResult<Bound<'py, PyCode>> {
        let start = match input {
            PyCodeInput::Eval => ffi::Py_eval_input,
            PyCodeInput::File => ffi::Py_file_input,
        };
        unsafe {
            ffi::Py_CompileString(code.as_ptr(), filename.as_ptr(), start)
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }
}

/// Implementation of functionality for [`PyCode`].
///
/// These methods are defined for the `Bound<'py, PyCode>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
pub trait PyCodeMethods<'py> {
    /// Runs code object.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    fn run(
        &self,
        globals: Option<&Bound<'py, PyDict>>,
        locals: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>>;
}

impl<'py> PyCodeMethods<'py> for Bound<'py, PyCode> {
    fn run(
        &self,
        globals: Option<&Bound<'py, PyDict>>,
        locals: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let mptr = unsafe {
            ffi::compat::PyImport_AddModuleRef(ffi::c_str!("__main__").as_ptr())
                .assume_owned_or_err(self.py())?
        };
        let attr = mptr.getattr(crate::intern!(self.py(), "__dict__"))?;
        let globals = match globals {
            Some(globals) => globals,
            None => attr.cast::<PyDict>()?,
        };
        let locals = locals.unwrap_or(globals);

        // If `globals` don't provide `__builtins__`, most of the code will fail if Python
        // version is <3.10. That's probably not what user intended, so insert `__builtins__`
        // for them.
        //
        // See also:
        // - https://github.com/python/cpython/pull/24564 (the same fix in CPython 3.10)
        // - https://github.com/PyO3/pyo3/issues/3370
        let builtins_s = crate::intern!(self.py(), "__builtins__");
        let has_builtins = globals.contains(builtins_s)?;
        if !has_builtins {
            crate::sync::with_critical_section(globals, || {
                // check if another thread set __builtins__ while this thread was blocked on the critical section
                let has_builtins = globals.contains(builtins_s)?;
                if !has_builtins {
                    // Inherit current builtins.
                    let builtins = unsafe { ffi::PyEval_GetBuiltins() };

                    // `PyDict_SetItem` doesn't take ownership of `builtins`, but `PyEval_GetBuiltins`
                    // seems to return a borrowed reference, so no leak here.
                    if unsafe {
                        ffi::PyDict_SetItem(globals.as_ptr(), builtins_s.as_ptr(), builtins)
                    } == -1
                    {
                        return Err(PyErr::fetch(self.py()));
                    }
                }
                Ok(())
            })?;
        }

        unsafe {
            ffi::PyEval_EvalCode(self.as_ptr(), globals.as_ptr(), locals.as_ptr())
                .assume_owned_or_err(self.py())
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    fn test_type_object() {
        use crate::types::PyTypeMethods;
        use crate::{PyTypeInfo, Python};

        Python::attach(|py| {
            assert_eq!(super::PyCode::type_object(py).name().unwrap(), "code");
        })
    }
}
