#![cfg(feature = "macros")]
#![cfg(any(not(Py_LIMITED_API), Py_3_11))]

use pyo3::buffer::PyBuffer;
use pyo3::exceptions::PyBufferError;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::AsPyPointer;
use std::ffi::CString;
use std::os::raw::{c_int, c_void};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod common;

#[pyclass]
struct TestBufferClass {
    vec: Vec<u8>,
    drop_called: Arc<AtomicBool>,
}

#[pymethods]
impl TestBufferClass {
    unsafe fn __getbuffer__(
        mut slf: PyRefMut<'_, Self>,
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

        (*view).buf = slf.vec.as_mut_ptr() as *mut c_void;
        (*view).len = slf.vec.len() as isize;
        (*view).readonly = 1;
        (*view).itemsize = 1;

        (*view).format = if (flags & ffi::PyBUF_FORMAT) == ffi::PyBUF_FORMAT {
            let msg = CString::new("B").unwrap();
            msg.into_raw()
        } else {
            ptr::null_mut()
        };

        (*view).ndim = 1;
        (*view).shape = if (flags & ffi::PyBUF_ND) == ffi::PyBUF_ND {
            &mut (*view).len
        } else {
            ptr::null_mut()
        };

        (*view).strides = if (flags & ffi::PyBUF_STRIDES) == ffi::PyBUF_STRIDES {
            &mut (*view).itemsize
        } else {
            ptr::null_mut()
        };

        (*view).suboffsets = ptr::null_mut();
        (*view).internal = ptr::null_mut();

        Ok(())
    }

    unsafe fn __releasebuffer__(&self, view: *mut ffi::Py_buffer) {
        // Release memory held by the format string
        drop(CString::from_raw((*view).format));
    }
}

impl Drop for TestBufferClass {
    fn drop(&mut self) {
        print!("dropped");
        self.drop_called.store(true, Ordering::Relaxed);
    }
}

#[test]
fn test_buffer() {
    let drop_called = Arc::new(AtomicBool::new(false));

    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let instance = Py::new(
            py,
            TestBufferClass {
                vec: vec![b' ', b'2', b'3'],
                drop_called: drop_called.clone(),
            },
        )
        .unwrap();
        let env = [("ob", instance)].into_py_dict(py);
        py_assert!(py, *env, "bytes(ob) == b' 23'");
    }

    assert!(drop_called.load(Ordering::Relaxed));
}

#[test]
fn test_buffer_referenced() {
    let drop_called = Arc::new(AtomicBool::new(false));

    let buf = {
        let input = vec![b' ', b'2', b'3'];
        let gil = Python::acquire_gil();
        let py = gil.python();
        let instance: PyObject = TestBufferClass {
            vec: input.clone(),
            drop_called: drop_called.clone(),
        }
        .into_py(py);

        let buf = PyBuffer::<u8>::get(instance.as_ref(py)).unwrap();
        assert_eq!(buf.to_vec(py).unwrap(), input);
        drop(instance);
        buf
    };

    assert!(!drop_called.load(Ordering::Relaxed));

    {
        let _py = Python::acquire_gil().python();
        drop(buf);
    }

    assert!(drop_called.load(Ordering::Relaxed));
}
