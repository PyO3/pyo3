//! Enables direct write access to I/O streams in Python's `sys` module.
//!
//! In some cases printing to Rust's `std::io::stdout` or `std::io::stderr` will not appear
//! in the Python interpreter, e.g. in Jupyter notebooks. This module provides a way to write
//! directly to Python's I/O streams from Rust in such cases.
//!
//! ```rust
//! let mut stdout = pyo3::stdio::stdout();
//!
//! // This may not appear in Jupyter notebooks...
//! println!("Hello, world!");
//!
//! // ...but this will.
//! writeln!(stdout, "Hello, world!").unwrap();
//! ```

use crate::ffi;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::intern;
use crate::prelude::*;
use crate::types::PyString;
use std::ffi::CStr;
use std::io::Write;

/// Implements `std::io::Write` for a `PyAny` object. The underlying
/// Python object must provide both `write` and `flush` methods.
///
/// Because Python IO streams use UTF-8 encoding, this will convert the input bytes to a UTF-8 string before writing
/// using `String::from_utf8_lossy`.
pub struct PyStreamWriter(Py<PyAny>);

fn get_stdio_stream(stream: &CStr) -> PyStreamWriter {
    Python::attach(|py| {
        // SAFETY: `PySys_GetObject` returns a borrowed reference to a Python object.
        // FIXME: use `PySys_GetAttrString` on 3.14+ once it's released, which returns owned references.
        let stream = unsafe { ffi::PySys_GetObject(stream.as_ptr()).assume_borrowed_or_err(py) };
        PyStreamWriter(
            stream
                .expect("failed to get Python stream")
                .to_owned()
                .unbind(),
        )
    })
}

/// Construct a new [`PyStreamWriter`] for Python's `sys.stdout` stream.
pub fn stdout() -> PyStreamWriter {
    get_stdio_stream(ffi::c_str!("stdout"))
}

/// Construct a new [`PyStreamWriter`] for Python's `sys.stderr` stream.
pub fn stderr() -> PyStreamWriter {
    get_stdio_stream(ffi::c_str!("stderr"))
}

/// Construct a new [`PyStreamWriter`] for Python's `sys.__stdout__` stream.
pub fn __stdout__() -> PyStreamWriter {
    get_stdio_stream(ffi::c_str!("__stdout__"))
}

/// Construct a new [`PyStreamWriter`] for Python's `sys.__stderr__` stream.
pub fn __stderr__() -> PyStreamWriter {
    get_stdio_stream(ffi::c_str!("__stderr__"))
}

impl Write for PyStreamWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Python::attach(|py| PyStreamWriterBound(self.0.bind(py)).write(buf))
    }
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        // override `write_all` to avoid needing to attach for each write in the default
        // `write_all` loop
        Python::attach(|py| PyStreamWriterBound(self.0.bind(py)).write_all(buf))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Python::attach(|py| PyStreamWriterBound(self.0.bind(py)).flush())
    }
}

/// Thread-attached version of `PyStreamWriter` used internally to implement `Write`.
struct PyStreamWriterBound<'a, 'py>(&'a Bound<'py, PyAny>);

impl Write for PyStreamWriterBound<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let str = PyString::new(self.0.py(), &String::from_utf8_lossy(buf));
        self.0.call_method1(intern!(self.0.py(), "write"), (str,))?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.call_method0(intern!(self.0.py(), "flush"))?;
        Ok(())
    }
}
