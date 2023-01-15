use crate::{ffi, AsPyPointer, FromPyObject, IntoPy, Py, PyAny, PyResult, Python, ToPyObject};
use std::borrow::Cow;
use std::ops::Index;
use std::os::raw::c_char;
use std::slice::SliceIndex;
use std::str;

use super::bytearray::PyByteArray;

/// Represents a Python `bytes` object.
///
/// This type is immutable.
#[repr(transparent)]
pub struct PyBytes(PyAny);

pyobject_native_type_core!(PyBytes, ffi::PyBytes_Type, #checkfunction=ffi::PyBytes_Check);

impl PyBytes {
    /// Creates a new Python bytestring object.
    /// The bytestring is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new<'p>(py: Python<'p>, s: &[u8]) -> &'p PyBytes {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe { py.from_owned_ptr(ffi::PyBytes_FromStringAndSize(ptr, len)) }
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
    ///     let py_bytes = PyBytes::new_with(py, 10, |bytes: &mut [u8]| {
    ///         bytes.copy_from_slice(b"Hello Rust");
    ///         Ok(())
    ///     })?;
    ///     let bytes: &[u8] = FromPyObject::extract(py_bytes)?;
    ///     assert_eq!(bytes, b"Hello Rust");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn new_with<F>(py: Python<'_>, len: usize, init: F) -> PyResult<&PyBytes>
    where
        F: FnOnce(&mut [u8]) -> PyResult<()>,
    {
        unsafe {
            let pyptr = ffi::PyBytes_FromStringAndSize(std::ptr::null(), len as ffi::Py_ssize_t);
            // Check for an allocation error and return it
            let pypybytes: Py<PyBytes> = Py::from_owned_ptr_or_err(py, pyptr)?;
            let buffer = ffi::PyBytes_AsString(pyptr) as *mut u8;
            debug_assert!(!buffer.is_null());
            // Zero-initialise the uninitialised bytestring
            std::ptr::write_bytes(buffer, 0u8, len);
            // (Further) Initialise the bytestring in init
            // If init returns an Err, pypybytearray will automatically deallocate the buffer
            init(std::slice::from_raw_parts_mut(buffer, len)).map(|_| pypybytes.into_ref(py))
        }
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
    pub unsafe fn from_ptr(py: Python<'_>, ptr: *const u8, len: usize) -> &PyBytes {
        py.from_owned_ptr(ffi::PyBytes_FromStringAndSize(
            ptr as *const _,
            len as isize,
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

impl Py<PyBytes> {
    /// Gets the Python bytes as a byte slice. Because Python bytes are
    /// immutable, the result may be used for as long as the reference to
    /// `self` is held, including when the GIL is released.
    pub fn as_bytes<'a>(&'a self, _py: Python<'_>) -> &'a [u8] {
        // py is required here because `PyBytes_AsString` and `PyBytes_Size`
        // can both technically raise exceptions which require the GIL to be
        // held. The only circumstance in which they raise is if the value
        // isn't really a `PyBytes`, but better safe than sorry.
        unsafe {
            let buffer = ffi::PyBytes_AsString(self.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(self.as_ptr()) as usize;
            debug_assert!(!buffer.is_null());
            std::slice::from_raw_parts(buffer, length)
        }
    }
}

/// This is the same way [Vec] is indexed.
impl<I: SliceIndex<[u8]>> Index<I> for PyBytes {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_bytes()[index]
    }
}

/// Special-purpose trait impl to efficiently handle both `bytes` and `bytearray`
///
/// If the source object is a `bytes` object, the `Cow` will be borrowed and
/// pointing into the source object, and no copying or heap allocations will happen.
/// If it is a `bytearray`, its contents will be copied to an owned `Cow`.
impl<'source> FromPyObject<'source> for Cow<'source, [u8]> {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        if let Ok(bytes) = ob.downcast::<PyBytes>() {
            return Ok(Cow::Borrowed(bytes.as_bytes()));
        }

        let byte_array = ob.downcast::<PyByteArray>()?;
        Ok(Cow::Owned(byte_array.to_vec()))
    }
}

impl ToPyObject for Cow<'_, [u8]> {
    fn to_object(&self, py: Python<'_>) -> Py<PyAny> {
        PyBytes::new(py, self.as_ref()).into()
    }
}

impl IntoPy<Py<PyAny>> for Cow<'_, [u8]> {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        self.to_object(py)
    }
}

#[cfg(test)]
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
    fn test_bytes_new_with() -> super::PyResult<()> {
        Python::with_gil(|py| -> super::PyResult<()> {
            let py_bytes = PyBytes::new_with(py, 10, |b: &mut [u8]| {
                b.copy_from_slice(b"Hello Rust");
                Ok(())
            })?;
            let bytes: &[u8] = FromPyObject::extract(py_bytes)?;
            assert_eq!(bytes, b"Hello Rust");
            Ok(())
        })
    }

    #[test]
    fn test_bytes_new_with_zero_initialised() -> super::PyResult<()> {
        Python::with_gil(|py| -> super::PyResult<()> {
            let py_bytes = PyBytes::new_with(py, 10, |_b: &mut [u8]| Ok(()))?;
            let bytes: &[u8] = FromPyObject::extract(py_bytes)?;
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

    #[test]
    fn test_cow_impl() {
        Python::with_gil(|py| {
            let bytes = py.eval(r#"b"foobar""#, None, None).unwrap();
            let cow = bytes.extract::<Cow<'_, [u8]>>().unwrap();
            assert_eq!(cow, Cow::<[u8]>::Borrowed(b"foobar"));

            let byte_array = py.eval(r#"bytearray(b"foobar")"#, None, None).unwrap();
            let cow = byte_array.extract::<Cow<'_, [u8]>>().unwrap();
            assert_eq!(cow, Cow::<[u8]>::Owned(b"foobar".to_vec()));

            let something_else_entirely = py.eval("42", None, None).unwrap();
            something_else_entirely
                .extract::<Cow<'_, [u8]>>()
                .unwrap_err();

            let cow = Cow::<[u8]>::Borrowed(b"foobar").to_object(py);
            assert!(cow.as_ref(py).is_instance_of::<PyBytes>());

            let cow = Cow::<[u8]>::Owned(b"foobar".to_vec()).to_object(py);
            assert!(cow.as_ref(py).is_instance_of::<PyBytes>());
        });
    }
}
