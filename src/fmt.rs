#[cfg(any(doc, all(Py_3_14, not(Py_LIMITED_API))))]
use crate::{types::PyString, Python};
#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
use {
    crate::ffi::{
        PyUnicodeWriter_Create, PyUnicodeWriter_Discard, PyUnicodeWriter_Finish,
        PyUnicodeWriter_WriteChar, PyUnicodeWriter_WriteUTF8,
    },
    crate::ffi_ptr_ext::FfiPtrExt,
    crate::impl_::callback::WrappingCastTo,
    crate::py_result_ext::PyResultExt,
    crate::IntoPyObject,
    crate::{ffi, Bound, PyErr, PyResult},
    std::fmt,
    std::mem::ManuallyDrop,
    std::ptr::NonNull,
};

/// This macro is analogous to Rust's [`format!`] macro, but returns a [`PyString`] instead of a [`String`].
///
/// # Arguments
///
/// The arguments are exactly like [`format!`], but with `py` (a [`Python`] token) as the first argument:
///
/// # Interning Advantage
///
/// If the format string is a static string and all arguments are constant at compile time,
/// this macro will intern the string in Python, offering better performance and memory usage
/// compared to [`PyString::from_fmt`].
///
/// ```rust
/// # use pyo3::{py_format, Python, types::PyString, Bound};
/// Python::attach(|py| {
///     let py_string: Bound<'_, PyString> = py_format!(py, "{} {}", "hello", "world").unwrap();
///     assert_eq!(py_string.to_string(), "hello world");
/// });
/// ```
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

#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
/// The `PyUnicodeWriter` is a utility for efficiently constructing Python strings
pub(crate) struct PyUnicodeWriter<'py> {
    python: Python<'py>,
    writer: NonNull<ffi::PyUnicodeWriter>,
    last_error: Option<PyErr>,
}

#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
impl<'py> PyUnicodeWriter<'py> {
    /// Creates a new `PyUnicodeWriter`.
    pub fn new(py: Python<'py>) -> PyResult<Self> {
        Self::with_capacity(py, 0)
    }

    /// Creates a new `PyUnicodeWriter` with the specified initial capacity.
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn take_error(&mut self) -> Option<PyErr> {
        self.last_error.take()
    }

    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyUnicodeWriter {
        self.writer.as_ptr()
    }

    #[inline]
    fn set_error(&mut self) {
        self.last_error = Some(PyErr::fetch(self.python));
    }
}

#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
impl fmt::Write for PyUnicodeWriter<'_> {
    #[inline]
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

    #[inline]
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

#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
impl Drop for PyUnicodeWriter<'_> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            PyUnicodeWriter_Discard(self.as_ptr());
        }
    }
}

#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
impl<'py> IntoPyObject<'py> for PyUnicodeWriter<'py> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, _py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
        self.into_py_string()
    }
}

#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
impl<'py> TryInto<Bound<'py, PyString>> for PyUnicodeWriter<'py> {
    type Error = PyErr;

    #[inline]
    fn try_into(self) -> PyResult<Bound<'py, PyString>> {
        self.into_py_string()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
    use super::*;
    use crate::types::PyStringMethods;
    use crate::{IntoPyObject, Python};

    #[test]
    #[allow(clippy::write_literal)]
    #[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
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
    #[allow(clippy::write_literal)]
    #[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
    fn unicode_writer_with_capacity() {
        use std::fmt::Write;
        Python::attach(|py| {
            let mut writer = PyUnicodeWriter::with_capacity(py, 10).unwrap();
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
