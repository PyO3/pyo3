#![feature(specialization)]

extern crate pyo3;

use std::os::raw::{c_int, c_void};
use std::ptr;

use pyo3::class::PyBufferProtocol;
use pyo3::exceptions::BufferError;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass]
struct TestClass {
    vec: Vec<u8>,
}

#[pyproto]
impl PyBufferProtocol for TestClass {
    fn bf_getbuffer(&self, view: *mut ffi::Py_buffer, flags: c_int) -> PyResult<()> {
        if view.is_null() {
            return Err(BufferError::py_err("View is null"));
        }

        unsafe {
            (*view).obj = ptr::null_mut();
        }

        if (flags & ffi::PyBUF_WRITABLE) == ffi::PyBUF_WRITABLE {
            return Err(BufferError::py_err("Object is not writable"));
        }

        let bytes = &self.vec;

        unsafe {
            (*view).buf = bytes.as_ptr() as *mut c_void;
            (*view).len = bytes.len() as isize;
            (*view).readonly = 1;
            (*view).itemsize = 1;

            (*view).format = ptr::null_mut();
            if (flags & ffi::PyBUF_FORMAT) == ffi::PyBUF_FORMAT {
                let msg = ::std::ffi::CStr::from_ptr("B\0".as_ptr() as *const _);
                (*view).format = msg.as_ptr() as *mut _;
            }

            (*view).ndim = 1;
            (*view).shape = ptr::null_mut();
            if (flags & ffi::PyBUF_ND) == ffi::PyBUF_ND {
                (*view).shape = (&((*view).len)) as *const _ as *mut _;
            }

            (*view).strides = ptr::null_mut();
            if (flags & ffi::PyBUF_STRIDES) == ffi::PyBUF_STRIDES {
                (*view).strides = &((*view).itemsize) as *const _ as *mut _;
            }

            (*view).suboffsets = ptr::null_mut();
            (*view).internal = ptr::null_mut();
        }

        Ok(())
    }
}

#[cfg(Py_3)]
#[test]
fn test_buffer() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let t = py
        .init(|_| TestClass {
            vec: vec![b' ', b'2', b'3'],
        })
        .unwrap();

    let d = PyDict::new(py);
    d.set_item("ob", t).unwrap();
    py.run("assert bytes(ob) == b' 23'", None, Some(d)).unwrap();
}

#[cfg(not(Py_3))]
#[test]
fn test_buffer() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let t = py
        .init(|_| TestClass {
            vec: vec![b' ', b'2', b'3'],
        })
        .unwrap();

    let d = PyDict::new(py);
    d.set_item("ob", t).unwrap();
    py.run("assert memoryview(ob).tobytes() == ' 23'", None, Some(d))
        .unwrap();
}
