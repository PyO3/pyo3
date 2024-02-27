use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::{Borrowed, Bound};
use crate::types::any::PyAnyMethods;
use crate::{ffi, Py, PyAny, PyNativeType, PyResult, Python};
use std::ops::Index;
use std::os::raw::c_char;
use std::slice::SliceIndex;
use std::str;

/// Represents a Python `bytes` object.
///
/// This type is immutable.
#[repr(transparent)]
pub struct PyBytes(PyAny);

pyobject_native_type_core!(PyBytes, pyobject_native_static_type_object!(ffi::PyBytes_Type), #checkfunction=ffi::PyBytes_Check);

impl PyBytes {
    /// Deprecated form of [`PyBytes::new_bound`].
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyBytes::new` will be replaced by `PyBytes::new_bound` in a future PyO3 version"
        )
    )]
    pub fn new<'p>(py: Python<'p>, s: &[u8]) -> &'p PyBytes {
        Self::new_bound(py, s).into_gil_ref()
    }

    /// Creates a new Python bytestring object.
    /// The bytestring is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new_bound<'p>(py: Python<'p>, s: &[u8]) -> Bound<'p, PyBytes> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            ffi::PyBytes_FromStringAndSize(ptr, len)
                .assume_owned(py)
                .downcast_into_unchecked()
        }
    }

    /// Deprecated form of [`PyBytes::new_bound_with`].
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyBytes::new_with` will be replaced by `PyBytes::new_bound_with` in a future PyO3 version"
        )
    )]
    pub fn new_with<F>(py: Python<'_>, len: usize, init: F) -> PyResult<&PyBytes>
    where
        F: FnOnce(&mut [u8]) -> PyResult<()>,
    {
        Self::new_bound_with(py, len, init).map(Bound::into_gil_ref)
    }

    /// Creates a new Python `bytes` object with an `init` closure to write its contents.
    /// Before calling `init` the bytes' contents are zero-initialised.
    /// * If Python raises a MemoryError on the allocation, `new_with` will return
    ///   it inside `Err`.
    /// * If `init` returns `Err(e)`, `new_with` will return `Err(e)`.
    /// * If `init` returns `Ok(())`, `new_with` will return `Ok(&PyBytes)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pyo3::{prelude::*, types::PyBytes};
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let py_bytes = PyBytes::new_bound_with(py, 10, |bytes: &mut [u8]| {
    ///         bytes.copy_from_slice(b"Hello Rust");
    ///         Ok(())
    ///     })?;
    ///     let bytes: &[u8] = py_bytes.extract()?;
    ///     assert_eq!(bytes, b"Hello Rust");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn new_bound_with<F>(py: Python<'_>, len: usize, init: F) -> PyResult<Bound<'_, PyBytes>>
    where
        F: FnOnce(&mut [u8]) -> PyResult<()>,
    {
        unsafe {
            let pyptr = ffi::PyBytes_FromStringAndSize(std::ptr::null(), len as ffi::Py_ssize_t);
            // Check for an allocation error and return it
            let pybytes = pyptr.assume_owned_or_err(py)?.downcast_into_unchecked();
            let buffer: *mut u8 = ffi::PyBytes_AsString(pyptr).cast();
            debug_assert!(!buffer.is_null());
            // Zero-initialise the uninitialised bytestring
            std::ptr::write_bytes(buffer, 0u8, len);
            // (Further) Initialise the bytestring in init
            // If init returns an Err, pypybytearray will automatically deallocate the buffer
            init(std::slice::from_raw_parts_mut(buffer, len)).map(|_| pybytes)
        }
    }

    /// Deprecated form of [`PyBytes::bound_from_ptr`].
    ///
    /// # Safety
    /// See [`PyBytes::bound_from_ptr`].
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyBytes::from_ptr` will be replaced by `PyBytes::bound_from_ptr` in a future PyO3 version"
        )
    )]
    pub unsafe fn from_ptr(py: Python<'_>, ptr: *const u8, len: usize) -> &PyBytes {
        Self::bound_from_ptr(py, ptr, len).into_gil_ref()
    }

    /// Creates a new Python byte string object from a raw pointer and length.
    ///
    /// Panics if out of memory.
    ///
    /// # Safety
    ///
    /// This function dereferences the raw pointer `ptr` as the
    /// leading pointer of a slice of length `len`. [As with
    /// `std::slice::from_raw_parts`, this is
    /// unsafe](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety).
    pub unsafe fn bound_from_ptr(py: Python<'_>, ptr: *const u8, len: usize) -> Bound<'_, PyBytes> {
        ffi::PyBytes_FromStringAndSize(ptr as *const _, len as isize)
            .assume_owned(py)
            .downcast_into_unchecked()
    }

    /// Gets the Python string as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.as_borrowed().as_bytes()
    }
}

/// Implementation of functionality for [`PyBytes`].
///
/// These methods are defined for the `Bound<'py, PyBytes>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyBytes")]
pub trait PyBytesMethods<'py>: crate::sealed::Sealed {
    /// Gets the Python string as a byte slice.
    fn as_bytes(&self) -> &[u8];
}

impl<'py> PyBytesMethods<'py> for Bound<'py, PyBytes> {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self.as_borrowed().as_bytes()
    }
}

impl<'a> Borrowed<'a, '_, PyBytes> {
    /// Gets the Python string as a byte slice.
    #[allow(clippy::wrong_self_convention)]
    fn as_bytes(self) -> &'a [u8] {
        unsafe {
            let buffer = ffi::PyBytes_AsString(self.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(self.as_ptr()) as usize;
            debug_assert!(!buffer.is_null());
            std::slice::from_raw_parts(buffer, length)
        }
    }
}

impl Py<PyBytes> {
    /// Gets the Python bytes as a byte slice. Because Python bytes are
    /// immutable, the result may be used for as long as the reference to
    /// `self` is held, including when the GIL is released.
    pub fn as_bytes<'a>(&'a self, py: Python<'_>) -> &'a [u8] {
        self.bind_borrowed(py).as_bytes()
    }
}

/// This is the same way [Vec] is indexed.
impl<I: SliceIndex<[u8]>> Index<I> for PyBytes {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_bytes()[index]
    }
}

/// This is the same way [Vec] is indexed.
impl<I: SliceIndex<[u8]>> Index<I> for Bound<'_, PyBytes> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_bytes()[index]
    }
}

#[cfg(test)]
#[cfg_attr(not(feature = "gil-refs"), allow(deprecated))]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_index() {
        Python::with_gil(|py| {
            let bytes = PyBytes::new(py, b"Hello World");
            assert_eq!(bytes[1], b'e');
        });
    }

    #[test]
    fn test_bound_bytes_index() {
        Python::with_gil(|py| {
            let bytes = PyBytes::new_bound(py, b"Hello World");
            assert_eq!(bytes[1], b'e');

            let bytes = &bytes;
            assert_eq!(bytes[1], b'e');
        });
    }

    #[test]
    fn test_bytes_new_with() -> super::PyResult<()> {
        Python::with_gil(|py| -> super::PyResult<()> {
            let py_bytes = PyBytes::new_with(py, 10, |b: &mut [u8]| {
                b.copy_from_slice(b"Hello Rust");
                Ok(())
            })?;
            let bytes: &[u8] = py_bytes.extract()?;
            assert_eq!(bytes, b"Hello Rust");
            Ok(())
        })
    }

    #[test]
    fn test_bytes_new_with_zero_initialised() -> super::PyResult<()> {
        Python::with_gil(|py| -> super::PyResult<()> {
            let py_bytes = PyBytes::new_with(py, 10, |_b: &mut [u8]| Ok(()))?;
            let bytes: &[u8] = py_bytes.extract()?;
            assert_eq!(bytes, &[0; 10]);
            Ok(())
        })
    }

    #[test]
    fn test_bytes_new_with_error() {
        use crate::exceptions::PyValueError;
        Python::with_gil(|py| {
            let py_bytes_result = PyBytes::new_with(py, 10, |_b: &mut [u8]| {
                Err(PyValueError::new_err("Hello Crustaceans!"))
            });
            assert!(py_bytes_result.is_err());
            assert!(py_bytes_result
                .err()
                .unwrap()
                .is_instance_of::<PyValueError>(py));
        });
    }
}
