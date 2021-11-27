// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::err::{error_on_minusone, PyResult};
use crate::ffi;
use crate::types::PyString;
use crate::{AsPyPointer, PyAny};

/// Represents a Python traceback.
#[repr(transparent)]
pub struct PyTraceback(PyAny);

pyobject_native_type_core!(
    PyTraceback,
    ffi::PyTraceBack_Type,
    #checkfunction=ffi::PyTraceBack_Check
);

impl PyTraceback {
    /// Formats the traceback as a string.
    ///
    /// This does not include the exception type and value. The exception type and value can be
    /// formatted using the `Display` implementation for `PyErr`.
    ///
    /// # Example
    ///
    /// The following code formats a Python traceback and exception pair from Rust:
    ///
    /// ```rust
    /// # use pyo3::{Python, PyResult};
    /// # let result: PyResult<()> =
    /// Python::with_gil(|py| {
    ///     let err = py
    ///         .run("raise Exception('banana')", None, None)
    ///         .expect_err("raise will create a Python error");
    ///
    ///     let traceback = err.traceback(py).expect("raised exception will have a traceback");
    ///     assert_eq!(
    ///         format!("{}{}", traceback.format()?, err),
    ///         "\
    /// Traceback (most recent call last):
    ///   File \"<string>\", line 1, in <module>
    /// Exception: banana\
    /// "
    ///     );
    ///     Ok(())
    /// })
    /// # ;
    /// # result.expect("example failed");
    /// ```
    pub fn format(&self) -> PyResult<String> {
        let py = self.py();
        let string_io = py.import("io")?.getattr("StringIO")?.call0()?;
        let result = unsafe { ffi::PyTraceBack_Print(self.as_ptr(), string_io.as_ptr()) };
        error_on_minusone(py, result)?;
        let formatted = string_io
            .getattr("getvalue")?
            .call0()?
            .downcast::<PyString>()?
            .to_str()?
            .to_owned();
        Ok(formatted)
    }
}

#[cfg(test)]
mod tests {
    use crate::Python;

    #[test]
    fn format_traceback() {
        Python::with_gil(|py| {
            let err = py
                .run("raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error");

            assert_eq!(
                err.traceback(py).unwrap().format().unwrap(),
                "Traceback (most recent call last):\n  File \"<string>\", line 1, in <module>\n"
            );
        })
    }
}
