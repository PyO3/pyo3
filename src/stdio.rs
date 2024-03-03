//! Enables direct write access to I/O streams in Python's `sys` module.

//! In some cases printing to Rust's `std::io::stdout` or `std::io::stderr` will not appear
//! in the Python interpreter, e.g. in Jupyter notebooks. This module provides a way to write
//! directly to Python's I/O streams from Rust in such cases.

//! ```rust
//! let mut stdout = pyo3::stdio::stdout();
//!   
//! // This may not appear in Jupyter notebooks...
//! println!("Hello, world!");
//!
//! // ...but this will.
//! writeln!(stdout, "Hello, world!").unwrap();
//! ```

use crate::ffi::{PySys_WriteStderr, PySys_WriteStdout};
use crate::prelude::*;
use std::io::{LineWriter, Write};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int};

trait PyStdioRawConfig {
    const STREAM: &'static str;
    const PRINTFCN: unsafe extern "C" fn(*const i8, ...);
}

struct PyStdoutRaw {}
impl PyStdioRawConfig for PyStdoutRaw {
    const STREAM: &'static str = "stdout";
    const PRINTFCN: unsafe extern "C" fn(*const i8, ...) = PySys_WriteStdout;
}

struct PyStderrRaw {}
impl PyStdioRawConfig for PyStderrRaw {
    const STREAM: &'static str = "stderr";
    const PRINTFCN: unsafe extern "C" fn(*const i8, ...) = PySys_WriteStderr;
}

struct PyStdioRaw<T: PyStdioRawConfig> {
    pystream: Py<PyAny>,
    _phantom: PhantomData<T>,
}

impl<T: PyStdioRawConfig> PyStdioRaw<T> {
    fn new() -> Self {
        let pystream: Py<PyAny> = Python::with_gil(|py| {
            let module = PyModule::import_bound(py, "sys").unwrap();
            module.getattr(T::STREAM).unwrap().into()
        });

        Self {
            pystream,
            _phantom: PhantomData,
        }
    }
}

impl<T: PyStdioRawConfig> Write for PyStdioRaw<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Python::with_gil(|_py| unsafe {
            (T::PRINTFCN)(
                b"%.*s\0".as_ptr().cast(),
                buf.len() as c_int,
                buf.as_ptr() as *const c_char,
            );
        });
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Python::with_gil(|py| -> std::io::Result<()> {
            self.pystream
                .call_method0(py, "flush")
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            Ok(())
        })
    }
}


struct PyStdio<T: PyStdioRawConfig> {
    inner: LineWriter<PyStdioRaw<T>>,
}

impl<T: PyStdioRawConfig> PyStdio<T> {
    fn new() -> Self {
        Self {
            inner: LineWriter::new(PyStdioRaw::new()),
        }
    }
}

impl<T: PyStdioRawConfig> Write for PyStdio<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// A handle to Python's `sys.stdout` stream.
pub struct PyStdout(PyStdio<PyStdoutRaw>);
/// A handle to Python's `sys.stderr` stream.
pub struct PyStderr(PyStdio<PyStderrRaw>);

/// Construct a new handle to Python's `sys.stdout` stream.
pub fn stdout() -> PyStdout {
    PyStdout(PyStdio::new())
}
/// Construct a new handle to Python's `sys.stderr` stream.
pub fn stderr() -> PyStderr {
    PyStderr(PyStdio::new())
}

impl Write for PyStdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}
impl Write for PyStderr {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}
