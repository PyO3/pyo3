use crate::{
    ffi,
    objects::{FromPyObject, PyAny},
    types::Bytes,
    AsPyPointer, IntoPy, PyObject, PyResult, Python,
};
use std::ops::Index;
use std::os::raw::c_char;
use std::slice::SliceIndex;

/// Represents a Python `bytes` object.
///
/// This type is immutable.
#[repr(transparent)]
pub struct PyBytes<'py>(pub(crate) PyAny<'py>);

pyo3_native_object!(PyBytes<'py>, Bytes, 'py);

impl<'py> PyBytes<'py> {
    /// Creates a new Python bytestring object.
    /// The bytestring is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python<'py>, s: &[u8]) -> Self {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            Self(PyAny::from_raw_or_panic(
                py,
                ffi::PyBytes_FromStringAndSize(ptr, len),
            ))
        }
    }

    /// Creates a new Python `bytes` object with an `init` closure to write its contents.
    /// Before calling `init` the bytes' contents are zero-initialised.
    /// * If Python raises a MemoryError on the allocation, `new_with` will return
    ///   it inside `Err`.
    /// * If `init` returns `Err(e)`, `new_with` will return `Err(e)`.
    /// * If `init` returns `Ok(())`, `new_with` will return `Ok(&PyBytes)`.
    ///
    /// # Example
    /// ```
    /// use pyo3::experimental::{prelude::*, objects::PyBytes};
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let py_bytes = PyBytes::new_with(py, 10, |bytes: &mut [u8]| {
    ///         bytes.copy_from_slice(b"Hello Rust");
    ///         Ok(())
    ///     })?;
    ///     let bytes: &[u8] = FromPyObject::extract(&py_bytes)?;
    ///     assert_eq!(bytes, b"Hello Rust");
    ///     Ok(())
    /// });
    /// ```
    pub fn new_with<F>(py: Python<'py>, len: usize, init: F) -> PyResult<Self>
    where
        F: FnOnce(&mut [u8]) -> PyResult<()>,
    {
        unsafe {
            let pyptr = ffi::PyBytes_FromStringAndSize(std::ptr::null(), len as ffi::Py_ssize_t);
            // Check for an allocation error and return it
            let bytes = Self(PyAny::from_raw_or_fetch_err(py, pyptr)?);
            let buffer = ffi::PyBytes_AsString(pyptr) as *mut u8;
            debug_assert!(!buffer.is_null());
            // Zero-initialise the uninitialised bytestring
            std::ptr::write_bytes(buffer, 0u8, len);
            // (Further) Initialise the bytestring in init
            // If init returns an Err, bytes will automatically deallocate the buffer
            init(std::slice::from_raw_parts_mut(buffer, len))?;
            Ok(bytes)
        }
    }

    /// Creates a new Python byte string object from a raw pointer and length.
    ///
    /// Panics if out of memory.
    pub unsafe fn from_ptr(py: Python<'py>, ptr: *const u8, len: usize) -> Self {
        Self(PyAny::from_raw_or_panic(
            py,
            ffi::PyBytes_FromStringAndSize(ptr as *const _, len as isize),
        ))
    }

    /// Gets the Python string as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let buffer = ffi::PyBytes_AsString(self.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(self.as_ptr()) as usize;
            debug_assert!(!buffer.is_null());
            std::slice::from_raw_parts(buffer, length)
        }
    }
}

/// This is the same way [Vec] is indexed.
impl<I: SliceIndex<[u8]>> Index<I> for PyBytes<'_> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_bytes()[index]
    }
}

impl<'a> IntoPy<PyObject> for &'a [u8] {
    fn into_py(self, py: Python) -> PyObject {
        PyBytes::new(py, self).into()
    }
}

impl<'a> FromPyObject<'a, '_> for &'a [u8] {
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
        Ok(obj.downcast::<PyBytes>()?.as_bytes())
    }
}
#[cfg(test)]
mod test {
    use super::PyBytes;
    use crate::FromPyObject;
    use crate::Python;

    #[test]
    fn test_extract_bytes() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let py_bytes = py.eval("b'Hello Python'", None, None).unwrap();
        let bytes: &[u8] = FromPyObject::extract(py_bytes).unwrap();
        assert_eq!(bytes, b"Hello Python");
    }

    #[test]
    fn test_bytes_index() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let bytes = PyBytes::new(py, b"Hello World");
        assert_eq!(bytes[1], b'e');
    }

    #[test]
    fn test_bytes_new_with() -> super::PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_bytes = PyBytes::new_with(py, 10, |b: &mut [u8]| {
            b.copy_from_slice(b"Hello Rust");
            Ok(())
        })?;
        let bytes: &[u8] = py_bytes.extract()?;
        assert_eq!(bytes, b"Hello Rust");
        Ok(())
    }

    #[test]
    fn test_bytes_new_with_zero_initialised() -> super::PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_bytes = PyBytes::new_with(py, 10, |_b: &mut [u8]| Ok(()))?;
        let bytes: &[u8] = py_bytes.extract()?;
        assert_eq!(bytes, &[0; 10]);
        Ok(())
    }

    #[test]
    fn test_bytes_new_with_error() {
        use crate::exceptions::PyValueError;
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_bytes_result = PyBytes::new_with(py, 10, |_b: &mut [u8]| {
            Err(PyValueError::new_err("Hello Crustaceans!"))
        });
        assert!(py_bytes_result.is_err());
        assert!(py_bytes_result
            .err()
            .unwrap()
            .is_instance::<PyValueError>(py));
    }
}
