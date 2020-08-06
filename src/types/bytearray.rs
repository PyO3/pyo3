// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::err::{PyErr, PyResult};
use crate::instance::PyNativeType;
use crate::{ffi, AsPyPointer, PyAny, Python};
use std::os::raw::c_char;
use std::slice;

/// Represents a Python `bytearray`.
#[repr(transparent)]
pub struct PyByteArray(PyAny);

pyobject_native_var_type!(PyByteArray, ffi::PyByteArray_Type, ffi::PyByteArray_Check);

impl PyByteArray {
    /// Creates a new Python bytearray object.
    ///
    /// The byte string is initialized by copying the data from the `&[u8]`.
    pub fn new<'p>(py: Python<'p>, src: &[u8]) -> &'p PyByteArray {
        let ptr = src.as_ptr() as *const c_char;
        let len = src.len() as ffi::Py_ssize_t;
        unsafe { py.from_owned_ptr::<PyByteArray>(ffi::PyByteArray_FromStringAndSize(ptr, len)) }
    }

    /// Creates a new Python `bytearray` object with an `init` closure to write its contents.
    /// Before calling `init` the bytearray is zero-initialised.
    /// * If `init` returns `Err(e)`, `new_with` will return `Err(e)`.
    /// * If `init` returns `Ok(new_len)`, the allocated bytearray will be truncated to length
    ///   `new_len` and `new_with` will return `Ok(&PyByteArray)`.
    /// * If `init` returns `Ok(None)`, `new_with` will return `Ok(&PyByteArray)`.
    ///
    /// Iff Python raises a MemoryError on the allocation, `new_with` will return it inside `Err`.
    ///
    /// # Example
    /// ```
    /// use pyo3::{prelude::*, types::PyByteArray};
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let py_bytearray = PyByteArray::new_with(py, 10, |bytes: &mut [u8]| {
    ///         bytes.copy_from_slice(b"Hello Rust");
    ///         Ok(None)
    ///     })?;
    ///     let bytearray: &[u8] = unsafe { py_bytearray.as_bytes() };
    ///     assert_eq!(bytearray, b"Hello Rust");
    ///     Ok(())
    /// });
    /// ```
    pub fn new_with<F>(py: Python, len: usize, init: F) -> PyResult<&PyByteArray>
    where
        F: FnOnce(&mut [u8]) -> PyResult<Option<usize>>,
    {
        unsafe {
            let pyptr =
                ffi::PyByteArray_FromStringAndSize(std::ptr::null(), len as ffi::Py_ssize_t);
            if pyptr.is_null() {
                return Err(PyErr::fetch(py));
            }
            let buffer = ffi::PyByteArray_AsString(pyptr) as *mut u8;
            debug_assert!(!buffer.is_null());
            // Zero-initialise the uninitialised bytearray
            std::ptr::write_bytes(buffer, 0u8, len);
            // (Further) Initialise the bytearray in init
            match init(std::slice::from_raw_parts_mut(buffer, len)) {
                Ok(Some(new_len)) if new_len < len => {
                    if ffi::PyByteArray_Resize(pyptr, new_len as ffi::Py_ssize_t) != 0 {
                        return Err(PyErr::fetch(py));
                    }
                }
                Ok(_) => (),
                Err(e) => {
                    // Deallocate pyptr to avoid leaking the bytearray
                    ffi::Py_DECREF(pyptr);
                    return Err(e);
                }
            };
            Ok(py.from_owned_ptr(pyptr))
        }
    }

    /// Creates a new Python bytearray object from another PyObject that
    /// implements the buffer protocol.
    pub fn from<'p, I>(py: Python<'p>, src: &'p I) -> PyResult<&'p PyByteArray>
    where
        I: AsPyPointer,
    {
        unsafe { py.from_owned_ptr_or_err(ffi::PyByteArray_FromObject(src.as_ptr())) }
    }

    /// Gets the length of the bytearray.
    #[inline]
    pub fn len(&self) -> usize {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe { ffi::PyByteArray_Size(self.as_ptr()) as usize }
    }

    /// Checks if the bytearray is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the start of the buffer containing the contents of the bytearray.
    ///
    /// Note that this bytearray object is both shared and mutable, and the backing buffer may be
    /// reallocated if the bytearray is resized. This can occur from Python code as well as from
    /// Rust via [PyByteArray::resize].
    ///
    /// As a result, the returned pointer should be dereferenced only if since calling this method
    /// no Python code has executed, [PyByteArray::resize] has not been called.
    pub fn data(&self) -> *mut u8 {
        unsafe { ffi::PyByteArray_AsString(self.as_ptr()) as *mut u8 }
    }

    /// Get the contents of this buffer as a slice.
    ///
    /// # Safety
    /// This bytearray must not be resized or edited while holding the slice.
    ///
    /// ## Safety Detail
    /// This method is equivalent to `std::slice::from_raw_parts(self.data(), self.len())`, and so
    /// all the safety notes of `std::slice::from_raw_parts` apply here.
    ///
    /// In particular, note that this bytearray object is both shared and mutable, and the backing
    /// buffer may be reallocated if the bytearray is resized. Mutations can occur from Python
    /// code as well as from Rust, via [PyByteArray::as_bytes_mut] and [PyByteArray::resize].
    ///
    /// Extreme care should be exercised when using this slice, as the Rust compiler will
    /// make optimizations based on the assumption the contents of this slice cannot change. This
    /// can easily lead to undefined behavior.
    ///
    /// As a result, this slice should only be used for short-lived operations to read this
    /// bytearray without executing any Python code, such as copying into a Vec.
    pub unsafe fn as_bytes(&self) -> &[u8] {
        slice::from_raw_parts(self.data(), self.len())
    }

    /// Get the contents of this buffer as a mutable slice.
    ///
    /// # Safety
    /// This slice should only be used for short-lived operations that write to this bytearray
    /// without executing any Python code. See the safety note for [PyByteArray::as_bytes].
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn as_bytes_mut(&self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.data(), self.len())
    }

    /// Copies the contents of the bytearray to a Rust vector.
    ///
    /// # Example
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::types::PyByteArray;
    /// # use pyo3::types::IntoPyDict;
    /// # let gil = GILGuard::acquire();
    /// # let py = gil.python();
    /// #
    /// let bytearray = PyByteArray::new(py, b"Hello World.");
    /// let mut copied_message = bytearray.to_vec();
    /// assert_eq!(b"Hello World.", copied_message.as_slice());
    ///
    /// copied_message[11] = b'!';
    /// assert_eq!(b"Hello World!", copied_message.as_slice());
    ///
    /// let locals = [("bytearray", bytearray)].into_py_dict(py);
    /// py.run("assert bytearray == b'Hello World.'", None, Some(locals)).unwrap();
    /// ```
    pub fn to_vec(&self) -> Vec<u8> {
        unsafe { self.as_bytes() }.to_vec()
    }

    /// Resizes the bytearray object to the new length `len`.
    ///
    /// Note that this will invalidate any pointers obtained by [PyByteArray::data], as well as
    /// any (unsafe) slices obtained from [PyByteArray::as_bytes] and [PyByteArray::as_bytes_mut].
    pub fn resize(&self, len: usize) -> PyResult<()> {
        unsafe {
            let result = ffi::PyByteArray_Resize(self.as_ptr(), len as ffi::Py_ssize_t);
            if result == 0 {
                Ok(())
            } else {
                Err(PyErr::fetch(self.py()))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::exceptions;
    use crate::object::PyObject;
    use crate::types::PyByteArray;
    use crate::Python;

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let src = b"Hello Python";
        let bytearray = PyByteArray::new(py, src);
        assert_eq!(src.len(), bytearray.len());
    }

    #[test]
    fn test_as_bytes() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let src = b"Hello Python";
        let bytearray = PyByteArray::new(py, src);

        let slice = unsafe { bytearray.as_bytes() };
        assert_eq!(src, slice);
        assert_eq!(bytearray.data() as *const _, slice.as_ptr());
    }

    #[test]
    fn test_as_bytes_mut() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let src = b"Hello Python";
        let bytearray = PyByteArray::new(py, src);

        let slice = unsafe { bytearray.as_bytes_mut() };
        assert_eq!(src, slice);
        assert_eq!(bytearray.data(), slice.as_mut_ptr());

        slice[0..5].copy_from_slice(b"Hi...");

        assert_eq!(
            bytearray.str().unwrap().to_str().unwrap(),
            "bytearray(b'Hi... Python')"
        );
    }

    #[test]
    fn test_to_vec() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let src = b"Hello Python";
        let bytearray = PyByteArray::new(py, src);

        let vec = bytearray.to_vec();
        assert_eq!(src, vec.as_slice());
    }

    #[test]
    fn test_from() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let src = b"Hello Python";
        let bytearray = PyByteArray::new(py, src);

        let ba: PyObject = bytearray.into();
        let bytearray = PyByteArray::from(py, &ba).unwrap();

        assert_eq!(src, unsafe { bytearray.as_bytes() });
    }

    #[test]
    fn test_from_err() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        if let Err(err) = PyByteArray::from(py, &py.None()) {
            assert!(err.is_instance::<exceptions::PyTypeError>(py));
        } else {
            panic!("error");
        }
    }

    #[test]
    fn test_resize() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let src = b"Hello Python";
        let bytearray = PyByteArray::new(py, src);

        bytearray.resize(20).unwrap();
        assert_eq!(20, bytearray.len());
    }

    #[test]
    fn test_byte_array_new_with() -> super::PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_bytearray = PyByteArray::new_with(py, 10, |b: &mut [u8]| {
            b.copy_from_slice(b"Hello Rust");
            Ok(None)
        })?;
        let bytearray: &[u8] = unsafe { py_bytearray.as_bytes() };
        assert_eq!(bytearray, b"Hello Rust");
        Ok(())
    }

    #[test]
    fn test_byte_array_new_with_zero_initialised() -> super::PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_bytearray = PyByteArray::new_with(py, 10, |_b: &mut [u8]| Ok(None))?;
        let bytearray: &[u8] = unsafe { py_bytearray.as_bytes() };
        assert_eq!(bytearray, &[0; 10]);
        Ok(())
    }

    #[test]
    fn test_byte_array_new_with_truncated() -> super::PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_bytearray = PyByteArray::new_with(py, 10, |b: &mut [u8]| {
            b.copy_from_slice(b"Hello Rust");
            Ok(Some(5))
        })?;
        let bytearray: &[u8] = unsafe { py_bytearray.as_bytes() };
        assert_eq!(bytearray, b"Hello");
        Ok(())
    }

    #[test]
    fn test_byte_array_new_with_no_growing() -> super::PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_bytearray = PyByteArray::new_with(py, 10, |b: &mut [u8]| {
            b.copy_from_slice(b"Hello Rust");
            Ok(Some(15))
        })?;
        let bytearray: &[u8] = unsafe { py_bytearray.as_bytes() };
        assert_eq!(bytearray, b"Hello Rust");
        Ok(())
    }

    #[test]
    fn test_byte_array_new_with_error() -> super::PyResult<()> {
        use crate::exceptions::PyValueError;
        use crate::types::PyDict;

        let alloc_size = 1024 * 1024;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let locals = PyDict::new(py);

        py.run(
            r#"
import tracemalloc

tracemalloc.start()
memory_usage_before = sum(
    alloc.size
    for alloc in tracemalloc.take_snapshot()
    .filter_traces((tracemalloc.Filter(True, "<unknown>"),))
    .statistics("filename")
)
"#,
            None,
            Some(locals),
        )?;

        let err = PyValueError::py_err("Hello Crustaceans!");
        let py_bytearray_result = PyByteArray::new_with(py, alloc_size, |_b: &mut [u8]| Err(err));

        assert!(py_bytearray_result.is_err());
        assert!(py_bytearray_result
            .err()
            .unwrap()
            .is_instance::<PyValueError>(py));

        py.run(
            r#"
memory_usage_after = sum(
    alloc.size
    for alloc in tracemalloc.take_snapshot()
    .filter_traces((tracemalloc.Filter(True, "<unknown>"),))
    .statistics("filename")
)
tracemalloc.stop()
"#,
            None,
            Some(locals),
        )?;

        let memory_usage_before: usize =
            locals.get_item("memory_usage_before").unwrap().extract()?;
        let memory_usage_after: usize = locals.get_item("memory_usage_after").unwrap().extract()?;

        assert!((memory_usage_after - memory_usage_before) < alloc_size);

        Ok(())
    }
}
