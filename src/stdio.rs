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

use crate::types::PyString;
use crate::intern;
use crate::prelude::*;
use std::io::{LineWriter, Write};

/// Implements `std::io::Write` for a `PyAny` object.    The underlying 
///  Python object must provide both `write` and `flush` methods.
pub struct PyWriter(Py<PyAny>);

impl PyWriter {
    /// Construct a new `PyWriter` from a `PyAny` object.
    pub fn buffered(self) -> LineWriter<PyWriter> {
        LineWriter::new(self)
    }
}

/// A GIL-attached equivalent to PyWriter.
pub struct PyWriterBound<'a, 'py>(&'a Bound<'py, PyAny>);

fn get_stdio_pywriter(stream: &str) -> PyWriter {
    Python::with_gil(|py| {
        let module = PyModule::import_bound(py, "sys").unwrap();
        let stream = module.getattr(stream).unwrap();
        PyWriter(stream.into())
    })
}

/// Construct a new `PyWriter` for Python's `sys.stdout` stream.
pub fn stdout() -> PyWriter {
    get_stdio_pywriter("stdout")
}

/// Construct a new `PyWriter` for Python's `sys.stderr` stream.
pub fn stderr() -> PyWriter {
    get_stdio_pywriter("stderr")
}   

/// Construct a new `PyWriter` for Python's `sys.__stdout__` stream.
pub fn __stdout__() -> PyWriter {
    get_stdio_pywriter("__stdout__")
}

/// Construct a new `PyWriter` for Python's `sys.__stderr__` stream.
pub fn __stderr__() -> PyWriter {
    get_stdio_pywriter("__stderr__")
}   

impl Write for PyWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Python::with_gil(|py| {
            PyWriterBound(self.0.bind(py)).write(buf)
        })
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Python::with_gil(|py| {
            PyWriterBound(self.0.bind(py)).flush()
        })
    }
}

impl Write for PyWriterBound<'_, '_>  {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let str = PyString::new_bound(self.0.py(),&String::from_utf8_lossy(buf));
        self.0.call_method1(intern!(self.0.py(),"write"), (str,))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
         self.0.call_method0(intern!(self.0.py(),"flush"))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }
}

