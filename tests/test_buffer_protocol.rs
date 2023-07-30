#![cfg(feature = "macros")]
#![cfg(any(not(Py_LIMITED_API), Py_3_11))]

use pyo3::buffer::PyBuffer;
use pyo3::exceptions::PyBufferError;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
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
        slf: &PyCell<Self>,
        view: *mut ffi::Py_buffer,
        flags: c_int,
    ) -> PyResult<()> {
        fill_view_from_readonly_data(view, flags, &slf.borrow().vec, slf)
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

    Python::with_gil(|py| {
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
    });

    assert!(drop_called.load(Ordering::Relaxed));
}

#[test]
fn test_buffer_referenced() {
    let drop_called = Arc::new(AtomicBool::new(false));

    let buf = {
        let input = vec![b' ', b'2', b'3'];
        Python::with_gil(|py| {
            let instance: PyObject = TestBufferClass {
                vec: input.clone(),
                drop_called: drop_called.clone(),
            }
            .into_py(py);

            let buf = PyBuffer::<u8>::get(instance.as_ref(py)).unwrap();
            assert_eq!(buf.to_vec(py).unwrap(), input);
            drop(instance);
            buf
        })
    };

    assert!(!drop_called.load(Ordering::Relaxed));

    Python::with_gil(|_| {
        drop(buf);
    });

    assert!(drop_called.load(Ordering::Relaxed));
}

#[test]
#[cfg(Py_3_8)] // sys.unraisablehook not available until Python 3.8
fn test_releasebuffer_unraisable_error() {
    use common::UnraisableCapture;
    use pyo3::exceptions::PyValueError;

    #[pyclass]
    struct ReleaseBufferError {}

    #[pymethods]
    impl ReleaseBufferError {
        unsafe fn __getbuffer__(
            slf: &PyCell<Self>,
            view: *mut ffi::Py_buffer,
            flags: c_int,
        ) -> PyResult<()> {
            static BUF_BYTES: &[u8] = b"hello world";
            fill_view_from_readonly_data(view, flags, BUF_BYTES, slf)
        }

        unsafe fn __releasebuffer__(&self, _view: *mut ffi::Py_buffer) -> PyResult<()> {
            Err(PyValueError::new_err("oh dear"))
        }
    }

    Python::with_gil(|py| {
        let capture = UnraisableCapture::install(py);

        let instance = Py::new(py, ReleaseBufferError {}).unwrap();
        let env = [("ob", instance.clone())].into_py_dict(py);

        assert!(capture.borrow(py).capture.is_none());

        py_assert!(py, *env, "bytes(ob) == b'hello world'");

        let (err, object) = capture.borrow_mut(py).capture.take().unwrap();
        assert_eq!(err.to_string(), "ValueError: oh dear");
        assert!(object.is(&instance));

        capture.borrow_mut(py).uninstall(py);
    });
}

/// # Safety
///
/// `view` must be a valid pointer to ffi::Py_buffer, or null
/// `data` must outlive the Python lifetime of `owner` (i.e. data must be owned by owner, or data
/// must be static data)
unsafe fn fill_view_from_readonly_data(
    view: *mut ffi::Py_buffer,
    flags: c_int,
    data: &[u8],
    owner: &PyAny,
) -> PyResult<()> {
    if view.is_null() {
        return Err(PyBufferError::new_err("View is null"));
    }

    if (flags & ffi::PyBUF_WRITABLE) == ffi::PyBUF_WRITABLE {
        return Err(PyBufferError::new_err("Object is not writable"));
    }

    (*view).obj = ffi::_Py_NewRef(owner.as_ptr());

    (*view).buf = data.as_ptr() as *mut c_void;
    (*view).len = data.len() as isize;
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
