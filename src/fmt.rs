//! This module provides the `PyUnicodeWriter` struct, which is a utility for efficiently
//! constructing Python strings using Rust's `fmt::Write` trait.
//! It allows for incremental string construction, without the need for repeated allocations, and
//! is particularly useful for building strings in a performance-sensitive context.
use crate::ffi::compat::{
    PyUnicodeWriter_Create, PyUnicodeWriter_Discard, PyUnicodeWriter_Finish,
    PyUnicodeWriter_WriteChar, PyUnicodeWriter_WriteUTF8,
};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::impl_::callback::WrappingCastTo;
use crate::types::{PyAnyMethods, PyString};
use crate::{ffi, Bound, PyErr, PyResult, Python};
use std::ptr::NonNull;
use std::{fmt, mem};

/// The `PyUnicodeWriter` is a utility for efficiently constructing Python strings
pub struct PyUnicodeWriter {
    writer: NonNull<ffi::PyUnicodeWriter>,
    last_error: Option<PyErr>,
}

impl PyUnicodeWriter {
    /// Creates a new `PyUnicodeWriter`.
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        Self::with_capacity(py, 0)
    }

    /// Creates a new `PyUnicodeWriter` with the specified initial capacity.
    pub fn with_capacity(py: Python<'_>, capacity: usize) -> PyResult<Self> {
        match NonNull::new(unsafe { PyUnicodeWriter_Create(capacity.wrapping_cast()) }) {
            Some(ptr) => Ok(PyUnicodeWriter {
                writer: ptr,
                last_error: None,
            }),
            None => Err(PyErr::fetch(py)),
        }
    }

    /// Consumes the `PyUnicodeWriter` and returns a `Bound<PyString>` containing the constructed string.
    pub fn into_py_string(self, py: Python<'_>) -> PyResult<Bound<'_, PyString>> {
        let writer_ptr = self.as_ptr();
        mem::forget(self);
        Ok(unsafe {
            PyUnicodeWriter_Finish(writer_ptr)
                .assume_owned_or_err(py)?
                .downcast_into_unchecked()
        })
    }

    /// When fmt::Write returned an error, this function can be used to retrieve the last error that occurred.
    pub fn take_error(&mut self) -> Option<PyErr> {
        self.last_error.take()
    }

    fn as_ptr(&self) -> *mut ffi::PyUnicodeWriter {
        self.writer.as_ptr()
    }

    fn set_error(&mut self) {
        Python::with_gil(|py| {
            self.last_error = Some(PyErr::fetch(py));
        })
    }
}

impl fmt::Write for PyUnicodeWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let result = unsafe {
            PyUnicodeWriter_WriteUTF8(self.as_ptr(), s.as_ptr().cast(), s.len() as isize)
        };
        if result < 0 {
            self.set_error();
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        let result = unsafe { PyUnicodeWriter_WriteChar(self.as_ptr(), c as u32) };
        if result < 0 {
            self.set_error();
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }
}

impl Drop for PyUnicodeWriter {
    fn drop(&mut self) {
        unsafe {
            PyUnicodeWriter_Discard(self.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyStringMethods;
    use crate::{IntoPyObject, Python};
    use std::fmt::Write;

    #[test]
    fn unicode_writer_test() {
        Python::with_gil(|py| {
            let mut writer = PyUnicodeWriter::new(py).unwrap();
            write!(writer, "Hello {}!", "world").unwrap();
            writer.write_char('😎').unwrap();
            let result = writer.into_py_string(py).unwrap();
            assert_eq!(result.to_string(), "Hello world!😎");
        });
    }

    #[test]
    fn test_pystring_from_fmt() {
        Python::with_gil(|py| {
            PyString::from_fmt(py, format_args!("Hello {}!", "world")).unwrap();
        });
    }

    #[test]
    fn test_complex_format() {
        Python::with_gil(|py| {
            let complex_value = (42, "foo", 3.14).into_pyobject(py).unwrap();
            let py_string = PyString::from_fmt(
                py,
                format_args!("This is some complex value: {complex_value}"),
            )
            .unwrap();
            let actual = py_string.to_cow().unwrap();
            let expected = "This is some complex value: (42, 'foo', 3.14)";
            assert_eq!(actual, expected);
        });
    }
}
