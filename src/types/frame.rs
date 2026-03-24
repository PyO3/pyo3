use crate::ffi_ptr_ext::FfiPtrExt;
use crate::sealed::Sealed;
use crate::types::{PyCode, PyDict};
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
        let code = PyCode::empty(py, file_name, func_name, line_number);
        let globals = PyDict::new(py);
        let locals = PyDict::new(py);

        unsafe {
            Ok(ffi::PyFrame_New(
                state,
                code.into_ptr().cast(),
                globals.as_ptr(),
                locals.as_ptr(),
            )
            .cast::<PyObject>()
            .assume_owned_or_err(py)?
            .cast_into_unchecked::<PyFrame>())
        }
    }
}

/// Implementation of functionality for [`PyFrame`].
///
/// These methods are defined for the `Bound<'py, PyFrame>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyFrame")]
pub trait PyFrameMethods<'py>: Sealed {
    /// Returns the line number of the current instruction in the frame.
    fn line_number(&self) -> i32;
}

impl<'py> PyFrameMethods<'py> for Bound<'py, PyFrame> {
    fn line_number(&self) -> i32 {
        unsafe { ffi::PyFrame_GetLineNumber(self.as_ptr().cast()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        Python::attach(|py| {
            let frame = PyFrame::new(py, c"file.py", c"func", 42).unwrap();
            assert_eq!(frame.line_number(), 42);
        });
    }
}
