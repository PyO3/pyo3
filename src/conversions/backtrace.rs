#![cfg(all(feature = "btparse", not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
//! Conversion from standard backtrace

use alloc::{ffi::CString, vec::Vec};

use crate::{
    exceptions::PyRuntimeError,
    types::{PyFrame, PyTraceback},
    Bound, IntoPyObject, PyErr, PyResult, Python,
};

impl<'py> IntoPyObject<'py> for &std::backtrace::Backtrace {
    type Target = PyTraceback;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        btparse::deserialize(self)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?
            .into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for btparse::Backtrace {
    type Target = PyTraceback;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let mut frames = self.frames;
        trim_at_ffi_boundary(&mut frames);

        let mut tb: PyResult<Bound<'_, PyTraceback>> =
            Err(PyErr::new::<PyRuntimeError, _>("no frames"));
        for frame in frames {
            let line_number = frame.line.unwrap_or(0).try_into().unwrap_or(0);
            tb = Ok(PyTraceback::new(
                py,
                tb.ok(),
                frame.into_pyobject(py)?,
                0,
                line_number,
            )?);
        }
        tb
    }
}

impl<'py> IntoPyObject<'py> for btparse::Frame {
    type Target = PyFrame;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let file_name = self.file.and_then(|s| CString::new(s).ok());
        let function = cstring_maybe_trunc(self.function.as_str());
        PyFrame::new(
            py,
            file_name.as_deref().unwrap_or(c"<unknown>"),
            function.as_c_str(),
            self.line.unwrap_or(0).try_into().unwrap_or(0),
        )
    }
}

/// Keep only the innermost Rust segment: below the FFI boundary sit Python frames,
/// which the interpreter appends itself as the exception propagates out of the trampoline.
fn trim_at_ffi_boundary(frames: &mut Vec<btparse::Frame>) {
    if let Some(boundary) = frames.iter().position(|f| {
        f.function
            .starts_with(crate::impl_::trampoline::MODULE_PATH)
    }) {
        frames.truncate(boundary);
    }
}

fn cstring_maybe_trunc(s: &str) -> CString {
    CString::new(s).unwrap_or_else(|e| {
        let end = e.nul_position();
        let mut v = e.into_vec();
        v.truncate(end + 1);
        CString::from_vec_with_nul(v).expect("NulError lied")
    })
}
