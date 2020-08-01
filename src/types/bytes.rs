use crate::{
    ffi, AsPyPointer, FromPy, FromPyObject, PyAny, PyObject, PyResult, PyTryFrom, Python,
    ToPyObject,
};
use std::ops::Index;
use std::os::raw::c_char;
use std::slice::SliceIndex;
use std::str;

/// Represents a Python `bytes` object.
///
/// This type is immutable.
#[repr(transparent)]
pub struct PyBytes(PyAny);

pyobject_native_var_type!(PyBytes, ffi::PyBytes_Type, ffi::PyBytes_Check);

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

    /// Creates a new Python bytestring object and a mutable slice to its underlying buffer.
    /// The bytestring is uninitialised and must not be read.
    ///
    /// Panics if out of memory.
    unsafe fn new_with_uninit_impl(py: Python<'_>, len: usize) -> (&mut [u8], &PyBytes) {
        let length = len as ffi::Py_ssize_t;
        let pyptr = ffi::PyBytes_FromStringAndSize(std::ptr::null(), length);
        // Iff pyptr is null, py.from_owned_ptr(pyptr) will panic
        let pybytes = py.from_owned_ptr(pyptr);
        let buffer = ffi::PyBytes_AsString(pyptr) as *mut u8;
        debug_assert!(!buffer.is_null());
        let slice = std::slice::from_raw_parts_mut(buffer, len);
        (slice, pybytes)
    }

    /// Creates a new Python bytestring object.
    /// The `init` closure can initialise the bytestring.
    ///
    /// # Safety
    /// * The bytestring is zero-initialised and can be read inside `init`.
    ///
    /// Panics if out of memory.
    pub fn new_with<F: Fn(&mut [u8])>(py: Python<'_>, len: usize, init: F) -> &PyBytes {
        let (slice, pybytes) = unsafe { Self::new_with_uninit_impl(py, len) };
        #[cfg(feature = "slice_fill")]
        {
            slice.fill(0);
        }
        #[cfg(not(feature = "slice_fill"))]
        {
            slice.iter_mut().for_each(|x| *x = 0);
        }
        init(slice);
        pybytes
    }

    /// Creates a new Python bytestring object.
    ///
    /// # Safety
    /// * The bytestring is uninitialised and must not be read inside `init`.
    /// * The entire bytestring must be written to inside `init` such that any code that
    /// follows does not read uninitialised memory.
    ///
    /// Panics if out of memory.
    pub unsafe fn new_with_uninit<F: Fn(&mut [u8])>(
        py: Python<'_>,
        len: usize,
        init: F,
    ) -> &PyBytes {
        let (slice, pybytes) = Self::new_with_uninit_impl(py, len);
        init(slice);
        pybytes
    }

    /// Creates a new Python byte string object from a raw pointer and length.
    ///
    /// Panics if out of memory.
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

/// This is the same way [Vec] is indexed.
impl<I: SliceIndex<[u8]>> Index<I> for PyBytes {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_bytes()[index]
    }
}

impl<'a> FromPy<&'a [u8]> for PyObject {
    fn from_py(other: &'a [u8], py: Python) -> Self {
        PyBytes::new(py, other).to_object(py)
    }
}

impl<'a> FromPyObject<'a> for &'a [u8] {
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
        Ok(<PyBytes as PyTryFrom>::try_from(obj)?.as_bytes())
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
    fn test_bytes_new_with() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_bytes = PyBytes::new_with(py, 10, |b: &mut [u8]| {
            b.copy_from_slice(b"Hello Rust");
        });
        let bytes: &[u8] = FromPyObject::extract(py_bytes).unwrap();
        assert_eq!(bytes, b"Hello Rust");
    }

    #[test]
    fn test_bytes_new_with_zero_initialised() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_bytes = PyBytes::new_with(py, 10, |_b: &mut [u8]| ());
        let bytes: &[u8] = FromPyObject::extract(py_bytes).unwrap();
        assert_eq!(bytes, &[0; 10]);
    }
}
