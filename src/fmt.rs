//! This module provides the `PyUnicodeWriter` struct, which is a utility for efficiently
//! constructing Python strings using Rust's `fmt::Write` trait.
//! It allows for incremental string construction, without the need for repeated allocations, and
//! is particularly useful for building strings in a performance-sensitive context.
#[cfg(Py_3_14)]
use {
    crate::ffi::{
        PyUnicodeWriter_Create, PyUnicodeWriter_Discard, PyUnicodeWriter_Finish,
        PyUnicodeWriter_WriteChar, PyUnicodeWriter_WriteUTF8,
    },
    crate::ffi_ptr_ext::FfiPtrExt,
    crate::impl_::callback::WrappingCastTo,
    crate::py_result_ext::PyResultExt,
    crate::types::PyString,
    crate::IntoPyObject,
    crate::{ffi, Bound, PyErr, PyResult, Python},
    std::fmt,
    std::mem::ManuallyDrop,
    std::ptr::NonNull,
};

/// This is like the `format!` macro, but it returns a `PyString` instead of a `String`.
#[macro_export]
macro_rules! py_format {
    ($py: expr, $($arg:tt)*) => {{
        if let Some(static_string) = format_args!($($arg)*).as_str() {
            static INTERNED: $crate::sync::PyOnceLock<$crate::Py<$crate::types::PyString>> = $crate::sync::PyOnceLock::new();
            Ok(
                INTERNED
                .get_or_init($py, || $crate::types::PyString::intern($py, static_string).unbind())
                .bind($py)
                .to_owned()
            )
        } else {
            $crate::types::PyString::from_fmt($py, format_args!($($arg)*))
        }
    }}
}

#[cfg(Py_3_14)]
/// The `PyUnicodeWriter` is a utility for efficiently constructing Python strings
pub struct PyUnicodeWriter<'py> {
    python: Python<'py>,
    writer: NonNull<ffi::PyUnicodeWriter>,
    last_error: Option<PyErr>,
}

#[cfg(Py_3_14)]
impl<'py> PyUnicodeWriter<'py> {
    /// Creates a new `PyUnicodeWriter`.
    pub fn new(py: Python<'py>) -> PyResult<Self> {
        Self::with_capacity(py, 0)
    }

    /// Creates a new `PyUnicodeWriter` with the specified initial capacity.
    pub fn with_capacity(py: Python<'py>, capacity: usize) -> PyResult<Self> {
        match NonNull::new(unsafe { PyUnicodeWriter_Create(capacity.wrapping_cast()) }) {
            Some(ptr) => Ok(PyUnicodeWriter {
                python: py,
                writer: ptr,
                last_error: None,
            }),
            None => Err(PyErr::fetch(py)),
        }
    }

    /// Consumes the `PyUnicodeWriter` and returns a `Bound<PyString>` containing the constructed string.
    pub fn into_py_string(mut self) -> PyResult<Bound<'py, PyString>> {
        let py = self.python;
        if let Some(error) = self.take_error() {
            Err(error)
        } else {
            unsafe {
                PyUnicodeWriter_Finish(ManuallyDrop::new(self).as_ptr())
                    .assume_owned_or_err(py)
                    .cast_into_unchecked()
            }
        }
    }

    /// When fmt::Write returned an error, this function can be used to retrieve the last error that occurred.
    pub fn take_error(&mut self) -> Option<PyErr> {
        self.last_error.take()
    }

    fn as_ptr(&self) -> *mut ffi::PyUnicodeWriter {
        self.writer.as_ptr()
    }

    fn set_error(&mut self) {
        self.last_error = Some(PyErr::fetch(self.python));
    }
}

#[cfg(Py_3_14)]
impl fmt::Write for PyUnicodeWriter<'_> {
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
        let result = unsafe { PyUnicodeWriter_WriteChar(self.as_ptr(), c.into()) };
        if result < 0 {
            self.set_error();
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }
}

#[cfg(Py_3_14)]
impl Drop for PyUnicodeWriter<'_> {
    fn drop(&mut self) {
        unsafe {
            PyUnicodeWriter_Discard(self.as_ptr());
        }
    }
}

#[cfg(Py_3_14)]
impl<'py> IntoPyObject<'py> for PyUnicodeWriter<'py> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, _py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
        self.into_py_string()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(Py_3_14)]
    use super::*;
    use crate::types::PyStringMethods;
    use crate::{IntoPyObject, Python};

    #[test]
    #[allow(clippy::write_literal)]
    #[cfg(Py_3_14)]
    fn unicode_writer_test() {
        use std::fmt::Write;
        Python::attach(|py| {
            let mut writer = PyUnicodeWriter::new(py).unwrap();
            write!(writer, "Hello {}!", "world").unwrap();
            writer.write_char('😎').unwrap();
            let result = writer.into_py_string().unwrap();
            assert_eq!(result.to_string(), "Hello world!😎");
        });
    }

    #[test]
    fn test_pystring_from_fmt() {
        Python::attach(|py| {
            py_format!(py, "Hello {}!", "world").unwrap();
        });
    }

    #[test]
    fn test_complex_format() {
        Python::attach(|py| {
            let complex_value = (42, "foo", [0; 0]).into_pyobject(py).unwrap();
            let py_string = py_format!(py, "This is some complex value: {complex_value}").unwrap();
            let actual = py_string.to_cow().unwrap();
            let expected = "This is some complex value: (42, 'foo', [])";
            assert_eq!(actual, expected);
        });
    }
}
