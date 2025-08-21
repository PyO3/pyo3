use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::{Borrowed, Bound};
use crate::{ffi, Py, PyAny, PyResult, Python};
use std::ops::Index;
use std::slice::SliceIndex;
use std::str;

/// Represents a Python `bytes` object.
///
/// This type is immutable.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyBytes>`][crate::Py] or [`Bound<'py, PyBytes>`][Bound].
///
/// For APIs available on `bytes` objects, see the [`PyBytesMethods`] trait which is implemented for
/// [`Bound<'py, PyBytes>`][Bound].
///
/// # Equality
///
/// For convenience, [`Bound<'py, PyBytes>`][Bound] implements [`PartialEq<[u8]>`][PartialEq] to allow comparing the
/// data in the Python bytes to a Rust `[u8]` byte slice.
///
/// This is not always the most appropriate way to compare Python bytes, as Python bytes subclasses
/// may have different equality semantics. In situations where subclasses overriding equality might
/// be relevant, use [`PyAnyMethods::eq`](crate::types::any::PyAnyMethods::eq), at cost of the
/// additional overhead of a Python method call.
///
/// ```rust
/// # use pyo3::prelude::*;
/// use pyo3::types::PyBytes;
///
/// # Python::attach(|py| {
/// let py_bytes = PyBytes::new(py, b"foo".as_slice());
/// // via PartialEq<[u8]>
/// assert_eq!(py_bytes, b"foo".as_slice());
///
/// // via Python equality
/// let other = PyBytes::new(py, b"foo".as_slice());
/// assert!(py_bytes.as_any().eq(other).unwrap());
///
/// // Note that `eq` will convert its argument to Python using `IntoPyObject`.
/// // Byte collections are specialized, so that the following slice will indeed
/// // convert into a `bytes` object and not a `list`:
/// assert!(py_bytes.as_any().eq(b"foo".as_slice()).unwrap());
/// # });
/// ```
#[repr(transparent)]
pub struct PyBytes(PyAny);

pyobject_native_type_core!(PyBytes, pyobject_native_static_type_object!(ffi::PyBytes_Type), #checkfunction=ffi::PyBytes_Check);

impl PyBytes {
    /// Creates a new Python bytestring object.
    /// The bytestring is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new<'p>(py: Python<'p>, s: &[u8]) -> Bound<'p, PyBytes> {
        let ptr = s.as_ptr().cast();
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            ffi::PyBytes_FromStringAndSize(ptr, len)
                .assume_owned(py)
                .cast_into_unchecked()
        }
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
    /// Python::attach(|py| -> PyResult<()> {
    ///     let py_bytes = PyBytes::new_with(py, 10, |bytes: &mut [u8]| {
    ///         bytes.copy_from_slice(b"Hello Rust");
    ///         Ok(())
    ///     })?;
    ///     let bytes: &[u8] = py_bytes.extract()?;
    ///     assert_eq!(bytes, b"Hello Rust");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    #[inline]
    pub fn new_with<F>(py: Python<'_>, len: usize, init: F) -> PyResult<Bound<'_, PyBytes>>
    where
        F: FnOnce(&mut [u8]) -> PyResult<()>,
    {
        unsafe {
            let pyptr = ffi::PyBytes_FromStringAndSize(std::ptr::null(), len as ffi::Py_ssize_t);
            // Check for an allocation error and return it
            let pybytes = pyptr.assume_owned_or_err(py)?.cast_into_unchecked();
            let buffer: *mut u8 = ffi::PyBytes_AsString(pyptr).cast();
            debug_assert!(!buffer.is_null());
            // Zero-initialise the uninitialised bytestring
            std::ptr::write_bytes(buffer, 0u8, len);
            // (Further) Initialise the bytestring in init
            // If init returns an Err, pypybytearray will automatically deallocate the buffer
            init(std::slice::from_raw_parts_mut(buffer, len)).map(|_| pybytes)
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
    pub unsafe fn from_ptr(py: Python<'_>, ptr: *const u8, len: usize) -> Bound<'_, PyBytes> {
        unsafe {
            ffi::PyBytes_FromStringAndSize(ptr.cast(), len as isize)
                .assume_owned(py)
                .cast_into_unchecked()
        }
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
    pub(crate) fn as_bytes(self) -> &'a [u8] {
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
impl<I: SliceIndex<[u8]>> Index<I> for Bound<'_, PyBytes> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_bytes()[index]
    }
}

/// Compares whether the Python bytes object is equal to the [u8].
///
/// In some cases Python equality might be more appropriate; see the note on [`PyBytes`].
impl PartialEq<[u8]> for Bound<'_, PyBytes> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_borrowed() == *other
    }
}

/// Compares whether the Python bytes object is equal to the [u8].
///
/// In some cases Python equality might be more appropriate; see the note on [`PyBytes`].
impl PartialEq<&'_ [u8]> for Bound<'_, PyBytes> {
    #[inline]
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_borrowed() == **other
    }
}

/// Compares whether the Python bytes object is equal to the [u8].
///
/// In some cases Python equality might be more appropriate; see the note on [`PyBytes`].
impl PartialEq<Bound<'_, PyBytes>> for [u8] {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyBytes>) -> bool {
        *self == other.as_borrowed()
    }
}

/// Compares whether the Python bytes object is equal to the [u8].
///
/// In some cases Python equality might be more appropriate; see the note on [`PyBytes`].
impl PartialEq<&'_ Bound<'_, PyBytes>> for [u8] {
    #[inline]
    fn eq(&self, other: &&Bound<'_, PyBytes>) -> bool {
        *self == other.as_borrowed()
    }
}

/// Compares whether the Python bytes object is equal to the [u8].
///
/// In some cases Python equality might be more appropriate; see the note on [`PyBytes`].
impl PartialEq<Bound<'_, PyBytes>> for &'_ [u8] {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyBytes>) -> bool {
        **self == other.as_borrowed()
    }
}

/// Compares whether the Python bytes object is equal to the [u8].
///
/// In some cases Python equality might be more appropriate; see the note on [`PyBytes`].
impl PartialEq<[u8]> for &'_ Bound<'_, PyBytes> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_borrowed() == other
    }
}

/// Compares whether the Python bytes object is equal to the [u8].
///
/// In some cases Python equality might be more appropriate; see the note on [`PyBytes`].
impl PartialEq<[u8]> for Borrowed<'_, '_, PyBytes> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_bytes() == other
    }
}

/// Compares whether the Python bytes object is equal to the [u8].
///
/// In some cases Python equality might be more appropriate; see the note on [`PyBytes`].
impl PartialEq<&[u8]> for Borrowed<'_, '_, PyBytes> {
    #[inline]
    fn eq(&self, other: &&[u8]) -> bool {
        *self == **other
    }
}

/// Compares whether the Python bytes object is equal to the [u8].
///
/// In some cases Python equality might be more appropriate; see the note on [`PyBytes`].
impl PartialEq<Borrowed<'_, '_, PyBytes>> for [u8] {
    #[inline]
    fn eq(&self, other: &Borrowed<'_, '_, PyBytes>) -> bool {
        other == self
    }
}

/// Compares whether the Python bytes object is equal to the [u8].
///
/// In some cases Python equality might be more appropriate; see the note on [`PyBytes`].
impl PartialEq<Borrowed<'_, '_, PyBytes>> for &'_ [u8] {
    #[inline]
    fn eq(&self, other: &Borrowed<'_, '_, PyBytes>) -> bool {
        other == self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyAnyMethods as _;

    #[test]
    fn test_bytes_index() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"Hello World");
            assert_eq!(bytes[1], b'e');
        });
    }

    #[test]
    fn test_bound_bytes_index() {
        Python::attach(|py| {
            let bytes = PyBytes::new(py, b"Hello World");
            assert_eq!(bytes[1], b'e');

            let bytes = &bytes;
            assert_eq!(bytes[1], b'e');
        });
    }

    #[test]
    fn test_bytes_new_with() -> super::PyResult<()> {
        Python::attach(|py| -> super::PyResult<()> {
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
        Python::attach(|py| -> super::PyResult<()> {
            let py_bytes = PyBytes::new_with(py, 10, |_b: &mut [u8]| Ok(()))?;
            let bytes: &[u8] = py_bytes.extract()?;
            assert_eq!(bytes, &[0; 10]);
            Ok(())
        })
    }

    #[test]
    fn test_bytes_new_with_error() {
        use crate::exceptions::PyValueError;
        Python::attach(|py| {
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
    fn test_comparisons() {
        Python::attach(|py| {
            let b = b"hello, world".as_slice();
            let py_bytes = PyBytes::new(py, b);

            assert_eq!(py_bytes, b"hello, world".as_slice());

            assert_eq!(py_bytes, b);
            assert_eq!(&py_bytes, b);
            assert_eq!(b, py_bytes);
            assert_eq!(b, &py_bytes);

            assert_eq!(py_bytes, *b);
            assert_eq!(&py_bytes, *b);
            assert_eq!(*b, py_bytes);
            assert_eq!(*b, &py_bytes);

            let py_string = py_bytes.as_borrowed();

            assert_eq!(py_string, b);
            assert_eq!(&py_string, b);
            assert_eq!(b, py_string);
            assert_eq!(b, &py_string);

            assert_eq!(py_string, *b);
            assert_eq!(*b, py_string);
        })
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_as_string() {
        Python::attach(|py| {
            let b = b"hello, world".as_slice();
            let py_bytes = PyBytes::new(py, b);
            unsafe {
                assert_eq!(
                    ffi::PyBytes_AsString(py_bytes.as_ptr()) as *const std::ffi::c_char,
                    ffi::PyBytes_AS_STRING(py_bytes.as_ptr()) as *const std::ffi::c_char
                );
            }
        })
    }
}
