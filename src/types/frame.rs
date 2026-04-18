#![deny(clippy::undocumented_unsafe_blocks)]
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::sealed::Sealed;
use crate::types::{PyCode, PyDict};
use crate::PyAny;
use crate::{ffi, Bound, PyResult, Python};
use std::ffi::CStr;

/// Represents a Python frame.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFrame>`][crate::Py] or [`Bound<'py, PyFrame>`][crate::Bound].
#[repr(transparent)]
pub struct PyFrame(PyAny);

pyobject_native_type_core!(
    PyFrame,
    |py| crate::backend::current::types::frame_type_object(py),
    "types",
    "FrameType",
    #checkfunction=crate::backend::current::types::frame_check
);

impl PyFrame {
    /// Creates a new frame object.
    pub fn new<'py>(
        py: Python<'py>,
        file_name: &CStr,
        func_name: &CStr,
        line_number: i32,
    ) -> PyResult<Bound<'py, PyFrame>> {
        crate::backend::current::types::new_frame(py, file_name, func_name, line_number)
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

    /// Gets this frame's next outer frame if there is one
    #[cfg(all(Py_3_9, not(Py_LIMITED_API)))]
    fn outer(&self) -> Option<Bound<'py, PyFrame>>;

    /// Gets the frame code
    #[cfg(any(all(Py_3_9, not(Py_LIMITED_API)), Py_3_10))]
    fn code(&self) -> Bound<'py, PyCode>;

    /// Gets the variable `name` of this frame.
    #[cfg(all(Py_3_12, not(Py_LIMITED_API)))]
    fn var(&self, name: &CStr) -> PyResult<Bound<'py, PyAny>>;

    /// Gets this frame's `f_builtins` attribute
    #[cfg(all(Py_3_11, not(Py_LIMITED_API)))]
    fn builtins(&self) -> Bound<'py, PyDict>;

    /// Gets this frame's `f_globals` attribute
    #[cfg(all(Py_3_11, not(Py_LIMITED_API)))]
    fn globals(&self) -> Bound<'py, PyDict>;

    /// Gets this frame's `f_locals` attribute
    #[cfg(all(Py_3_11, not(Py_LIMITED_API)))]
    fn locals(&self) -> Bound<'py, PyAny>;
}

impl<'py> PyFrameMethods<'py> for Bound<'py, PyFrame> {
    fn line_number(&self) -> i32 {
        // SAFETY:
        // - we're attached to the interpreter
        // - `self` is a `PyFrameObject`
        unsafe { ffi::PyFrame_GetLineNumber(self.as_ptr().cast()) }
    }

    #[cfg(all(Py_3_9, not(Py_LIMITED_API)))]
    fn outer(&self) -> Option<Bound<'py, PyFrame>> {
        // SAFETY:
        // - we're attached to the interpreter
        // - `self` is a `PyFrameObject`
        // - `PyFrame_GetBack` returns an owned reference
        // - the result may be null if there is no outer frame, but no exception is raised
        // - the result is a frame object
        unsafe {
            ffi::PyFrame_GetBack(self.as_ptr().cast())
                .cast::<ffi::PyObject>()
                .assume_owned_or_opt(self.py())
                .map(|obj| obj.cast_into_unchecked())
        }
    }

    #[cfg(any(all(Py_3_9, not(Py_LIMITED_API)), Py_3_10))]
    fn code(&self) -> Bound<'py, PyCode> {
        // SAFETY:
        // - we're attached to the interpreter
        // - `self` is a `PyFrameObject`
        // - `PyFrame_GetCode` returns an owned reference
        // - the result can not be null
        // - the result is a code object
        unsafe {
            ffi::PyFrame_GetCode(self.as_ptr().cast())
                .cast::<ffi::PyObject>()
                .assume_owned_unchecked(self.py())
                .cast_into_unchecked()
        }
    }

    #[cfg(all(Py_3_12, not(Py_LIMITED_API)))]
    fn var(&self, name: &CStr) -> PyResult<Bound<'py, PyAny>> {
        // SAFETY:
        // - we're attached to the interpreter
        // - `self` is a `PyFrameObject`
        // - `PyFrame_GetVarString` returns an owned reference or raises an exception
        // - `name` is a valid null terminated C string
        unsafe {
            ffi::PyFrame_GetVarString(self.as_ptr().cast(), name.as_ptr().cast_mut())
                .assume_owned_or_err(self.py())
        }
    }

    #[cfg(all(Py_3_11, not(Py_LIMITED_API)))]
    fn builtins(&self) -> Bound<'py, PyDict> {
        // SAFETY:
        // - we're attached to the interpreter
        // - `self` is a `PyFrameObject`
        // - `PyFrame_GetBuiltins` returns an owned reference
        // - the result can not be null
        // - the result is a dict object
        unsafe {
            ffi::PyFrame_GetBuiltins(self.as_ptr().cast())
                .assume_owned_unchecked(self.py())
                .cast_into_unchecked()
        }
    }

    #[cfg(all(Py_3_11, not(Py_LIMITED_API)))]
    fn globals(&self) -> Bound<'py, PyDict> {
        // SAFETY:
        // - we're attached to the interpreter
        // - `self` is a `PyFrameObject`
        // - `PyFrame_GetGlobals` returns an owned reference
        // - the result can not be null
        // - the result is a dict object
        unsafe {
            ffi::PyFrame_GetGlobals(self.as_ptr().cast())
                .assume_owned_unchecked(self.py())
                .cast_into_unchecked()
        }
    }

    #[cfg(all(Py_3_11, not(Py_LIMITED_API)))]
    fn locals(&self) -> Bound<'py, PyAny> {
        // SAFETY:
        // - we're attached to the interpreter
        // - `self` is a `PyFrameObject`
        // - `PyFrame_GetLocals` returns an owned reference
        // - the result can not be null
        unsafe { ffi::PyFrame_GetLocals(self.as_ptr().cast()).assume_owned_unchecked(self.py()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(Py_3_9)]
    fn get_frame(py: Python<'_>) -> Bound<'_, PyFrame> {
        use crate::types::PyAnyMethods as _;

        let m = crate::types::PyModule::from_code(
            py,
            cr#"
import sys
CONST = "global"
def get_frame():
    var = 42
    return sys._getframe()
"#,
            c"frame.py",
            c"frame",
        )
        .unwrap();

        m.getattr("get_frame")
            .unwrap()
            .call0()
            .unwrap()
            .cast_into()
            .unwrap()
    }

    #[test]
    fn test_frame_creation() {
        Python::attach(|py| {
            let frame = PyFrame::new(py, c"file.py", c"func", 42).unwrap();
            assert_eq!(frame.line_number(), 42);
        });
    }

    #[test]
    #[cfg(all(Py_3_9, not(Py_LIMITED_API)))]
    fn test_frame_outer() {
        Python::attach(|py| {
            use crate::types::PyAnyMethods as _;

            let m = crate::types::PyModule::from_code(
                py,
                cr#"
import sys
def inner():
    return sys._getframe()
def outer():
    return inner()
"#,
                c"outer.py",
                c"outer",
            )
            .unwrap();

            let frame = m
                .getattr("outer")
                .unwrap()
                .call0()
                .unwrap()
                .cast_into()
                .unwrap();

            let back = frame.outer().unwrap();
            let f_back = frame.getattr("f_back").unwrap();

            assert_eq!(back.as_ptr(), f_back.as_ptr());
            assert_eq!(back.line_number(), 6)
        })
    }

    #[test]
    #[cfg(any(all(Py_3_9, not(Py_LIMITED_API)), Py_3_10))]
    fn test_frame_get_code() {
        Python::attach(|py| {
            use crate::types::PyAnyMethods as _;

            let frame = get_frame(py);
            let code = frame.code();
            let f_code = frame.getattr("f_code").unwrap();

            assert_eq!(code.as_ptr(), f_code.as_ptr());
        })
    }

    #[test]
    #[cfg(all(Py_3_12, not(Py_LIMITED_API)))]
    fn test_frame_get_var() {
        Python::attach(|py| {
            use crate::types::PyAnyMethods as _;

            let frame = get_frame(py);
            assert_eq!(frame.var(c"var").unwrap().extract::<u32>().unwrap(), 42);
            assert!(frame.var(c"var2").is_err());
        })
    }

    #[test]
    #[cfg(all(Py_3_11, not(Py_LIMITED_API)))]
    fn test_frame_get_builtins() {
        Python::attach(|py| {
            use crate::types::PyAnyMethods as _;

            let frame = get_frame(py);
            let builtins = frame.builtins();

            assert_eq!(
                builtins
                    .get_item("__name__")
                    .unwrap()
                    .extract::<&str>()
                    .unwrap(),
                "builtins"
            );
            assert!(builtins.contains("len").unwrap());
        })
    }

    #[test]
    #[cfg(all(Py_3_11, not(Py_LIMITED_API)))]
    fn test_frame_get_globals() {
        Python::attach(|py| {
            use crate::types::PyAnyMethods as _;

            let frame = get_frame(py);
            let globals = frame.globals();

            assert_eq!(
                globals
                    .get_item("CONST")
                    .unwrap()
                    .extract::<&str>()
                    .unwrap(),
                "global"
            );
        })
    }

    #[test]
    #[cfg(all(Py_3_11, not(Py_LIMITED_API)))]
    fn test_frame_get_locals() {
        Python::attach(|py| {
            use crate::types::PyAnyMethods as _;

            let frame = get_frame(py);
            let locals = frame.locals();

            assert_eq!(
                locals.get_item("var").unwrap().extract::<u32>().unwrap(),
                42
            );
        })
    }
}
