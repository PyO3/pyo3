use crate::ffi_ptr_ext::FfiPtrExt;
use crate::sealed::Sealed;
use crate::types::PyDict;
use crate::PyAny;
use crate::{ffi, Bound, PyResult, Python};
use pyo3_ffi::PyObject;
use std::ffi::CStr;

/// Represents a Python frame.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFrame>`][crate::Py] or [`Bound<'py, PyFrame>`][crate::Bound].
#[repr(transparent)]
pub struct PyFrame(PyAny);

pyobject_native_type_core!(
    PyFrame,
    pyobject_native_static_type_object!(ffi::PyFrame_Type),
    "types",
    "FrameType",
    #checkfunction=ffi::PyFrame_Check
);

impl PyFrame {
    /// Creates a new frame object.
    pub fn new<'py>(
        py: Python<'py>,
        file_name: &CStr,
        func_name: &CStr,
        line_number: i32,
    ) -> PyResult<Bound<'py, PyFrame>> {
        // Safety: Thread is attached because we have a python token
        let state = unsafe { ffi::compat::PyThreadState_GetUnchecked() };
        let globals = PyDict::new(py);
        let locals = PyDict::new(py);

        unsafe {
            let code = ffi::PyCode_NewEmpty(file_name.as_ptr(), func_name.as_ptr(), line_number);
            Ok(
                ffi::PyFrame_New(state, code, globals.as_ptr(), locals.as_ptr())
                    .cast::<PyObject>()
                    .assume_owned_or_err(py)?
                    .cast_into_unchecked::<PyFrame>(),
            )
        }
    }
}

#[doc(alias = "PyFrame")]
pub trait PyFrameMethods<'py>: Sealed {
    fn line_number(&self) -> i32;
}

impl<'py> PyFrameMethods<'py> for Bound<'py, PyFrame> {
    fn line_number(&self) -> i32 {
        unsafe { ffi::PyFrame_GetLineNumber(self.as_ptr().cast()) }
    }
}
