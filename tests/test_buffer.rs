#![cfg(feature = "macros")]
#![cfg(not(Py_LIMITED_API))]

use pyo3::{buffer::PyBuffer, exceptions::PyBufferError, ffi, prelude::*, AsPyPointer};
use std::{
    ffi::CStr,
    os::raw::{c_int, c_void},
    ptr,
};

#[macro_use]
mod common;

enum TestGetBufferError {
    NullShape,
    NullStrides,
    IncorrectItemSize,
    IncorrectFormat,
    IncorrectAlignment,
}

#[pyclass]
struct TestBufferErrors {
    buf: Vec<u32>,
    error: Option<TestGetBufferError>,
}

#[pymethods]
impl TestBufferErrors {
    unsafe fn __getbuffer__(
        slf: PyRefMut<Self>,
        view: *mut ffi::Py_buffer,
        flags: c_int,
    ) -> PyResult<()> {
        if view.is_null() {
            return Err(PyBufferError::new_err("View is null"));
        }

        if (flags & ffi::PyBUF_WRITABLE) == ffi::PyBUF_WRITABLE {
            return Err(PyBufferError::new_err("Object is not writable"));
        }

        (*view).obj = ffi::_Py_NewRef(slf.as_ptr());

        let bytes = &slf.buf;

        (*view).buf = bytes.as_ptr() as *mut c_void;
        (*view).len = bytes.len() as isize;
        (*view).readonly = 1;
        (*view).itemsize = std::mem::size_of::<u32>() as isize;

        let msg = CStr::from_bytes_with_nul(b"I\0").unwrap();
        (*view).format = msg.as_ptr() as *mut _;

        (*view).ndim = 1;
        (*view).shape = &mut (*view).len;

        (*view).strides = &mut (*view).itemsize;

        (*view).suboffsets = ptr::null_mut();
        (*view).internal = ptr::null_mut();

        if let Some(err) = &slf.error {
            use TestGetBufferError::*;
            match err {
                NullShape => {
                    (*view).shape = std::ptr::null_mut();
                }
                NullStrides => {
                    (*view).strides = std::ptr::null_mut();
                }
                IncorrectItemSize => {
                    (*view).itemsize += 1;
                }
                IncorrectFormat => {
                    (*view).format = CStr::from_bytes_with_nul(b"B\0").unwrap().as_ptr() as _;
                }
                IncorrectAlignment => (*view).buf = (*view).buf.add(1),
            }
        }

        Ok(())
    }
}

#[test]
fn test_get_buffer_errors() {
    Python::with_gil(|py| {
        let instance = Py::new(
            py,
            TestBufferErrors {
                buf: vec![0, 1, 2, 3],
                error: None,
            },
        )
        .unwrap();

        assert!(PyBuffer::<u32>::get(instance.as_ref(py)).is_ok());

        instance.borrow_mut(py).error = Some(TestGetBufferError::NullShape);
        assert_eq!(
            PyBuffer::<u32>::get(instance.as_ref(py))
                .unwrap_err()
                .to_string(),
            "BufferError: shape is null"
        );

        instance.borrow_mut(py).error = Some(TestGetBufferError::NullStrides);
        assert_eq!(
            PyBuffer::<u32>::get(instance.as_ref(py))
                .unwrap_err()
                .to_string(),
            "BufferError: strides is null"
        );

        instance.borrow_mut(py).error = Some(TestGetBufferError::IncorrectItemSize);
        assert_eq!(
            PyBuffer::<u32>::get(instance.as_ref(py))
                .unwrap_err()
                .to_string(),
            "BufferError: buffer contents are not compatible with u32"
        );

        instance.borrow_mut(py).error = Some(TestGetBufferError::IncorrectFormat);
        assert_eq!(
            PyBuffer::<u32>::get(instance.as_ref(py))
                .unwrap_err()
                .to_string(),
            "BufferError: buffer contents are not compatible with u32"
        );

        instance.borrow_mut(py).error = Some(TestGetBufferError::IncorrectAlignment);
        assert_eq!(
            PyBuffer::<u32>::get(instance.as_ref(py))
                .unwrap_err()
                .to_string(),
            "BufferError: buffer contents are insufficiently aligned for u32"
        );
    });
}
