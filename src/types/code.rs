use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::types::PyDict;
use crate::{ffi, PyResult, Python};
use crate::{Bound, PyAny};
use std::ffi::CStr;
use std::os::raw::c_int;

/// Represents a Python code object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyCode>`][crate::Py] or [`Bound<'py, PyCode>`][crate::Bound].
#[repr(transparent)]
pub struct PyCode(PyAny);

pyobject_native_type_core!(
    PyCode,
    pyobject_native_static_type_object!(ffi::PyCode_Type),
    #checkfunction=ffi::PyCode_Check
);

impl PyCode {
    /// Compiles an arbitrarily large string of code into a runnable code object.
    pub fn compile<'py>(
        py: Python<'py>,
        code_str: &CStr,
        start: c_int,
    ) -> PyResult<Bound<'py, Self>> {
        let code_obj = unsafe {
            ffi::Py_CompileString(code_str.as_ptr(), ffi::c_str!("<string>").as_ptr(), start)
                .assume_owned_or_err(py)
        };
        code_obj.downcast_into()
    }

    /// Runs compiled code object in the given context.
    pub fn run<'py>(
        py: Python<'py>,
        code_obj: &Bound<'py, PyCode>,
        globals: Option<&Bound<'py, PyDict>>,
        locals: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        py.run_code_object(code_obj, globals, locals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{IntoPyDict, PyAnyMethods, PyTypeMethods};
    use crate::{PyTypeInfo, Python};

    #[test]
    fn test_type_object() {
        Python::with_gil(|py| {
            assert_eq!(PyCode::type_object(py).name().unwrap(), "code");
        })
    }

    #[test]
    fn test_reuse_compiled_code() {
        Python::with_gil(|py| {
            // Perform one-off compilation of a code string
            let code_obj = PyCode::compile(
                py,
                ffi::c_str!("total = local_int + global_int"),
                ffi::Py_file_input,
            )
            .unwrap();

            // Run compiled code with globals & locals
            let globals = [("global_int", 50)].into_py_dict(py).unwrap();
            let locals = [("local_int", 100)].into_py_dict(py).unwrap();
            PyCode::run(py, &code_obj, Some(&globals), Some(&locals)).unwrap();

            let py_total = locals.get_item("total").unwrap();
            assert_eq!(py_total.extract::<i32>().unwrap(), 150);

            // Run compiled code with different globals & locals
            let globals = [("global_int", 150)].into_py_dict(py).unwrap();
            let locals = [("local_int", 350)].into_py_dict(py).unwrap();
            PyCode::run(py, &code_obj, Some(&globals), Some(&locals)).unwrap();

            let py_total = locals.get_item("total").unwrap();
            assert_eq!(py_total.extract::<i32>().unwrap(), 500);
        });
    }
}
