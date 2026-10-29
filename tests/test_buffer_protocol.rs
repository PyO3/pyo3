// TODO https://github.com/PyO3/pyo3/issues/5487
#![allow(clippy::undocumented_unsafe_blocks)]
#![cfg(feature = "macros")]
#![cfg(any(not(Py_LIMITED_API), Py_3_11))]

use pyo3::buffer::{PyBuffer, PyBufferRequest, PyBufferView, PyUntypedBufferView};
use pyo3::exceptions::PyBufferError;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::ffi::CString;
use std::ffi::{c_int, c_void};
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

mod test_utils;

#[pyclass]
struct TestBufferClass {
    vec: Vec<u8>,
    drop_called: Arc<AtomicBool>,
}

#[pymethods]
impl TestBufferClass {
    unsafe fn __getbuffer__(
        slf: Bound<'_, Self>,
        view: *mut ffi::Py_buffer,
        flags: c_int,
    ) -> PyResult<()> {
        unsafe { fill_view_from_readonly_data(view, flags, &slf.borrow().vec, slf.into_any()) }
    }

    unsafe fn __releasebuffer__(&self, view: *mut ffi::Py_buffer) {
        // Release memory held by the format string
        drop(unsafe { CString::from_raw((*view).format) });
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

    Python::attach(|py| {
        let instance = Py::new(
            py,
            TestBufferClass {
                vec: vec![b' ', b'2', b'3'],
                drop_called: drop_called.clone(),
            },
        )
        .unwrap();
        let env = [("ob", instance)].into_py_dict(py).unwrap();
        py_assert!(py, *env, "bytes(ob) == b' 23'");
    });

    assert!(drop_called.load(Ordering::Relaxed));
}

#[test]
fn test_buffer_referenced() {
    let drop_called = Arc::new(AtomicBool::new(false));

    let buf = {
        let input = vec![b' ', b'2', b'3'];
        Python::attach(|py| {
            let instance = TestBufferClass {
                vec: input.clone(),
                drop_called: drop_called.clone(),
            }
            .into_pyobject(py)
            .unwrap();

            let buf = PyBuffer::<u8>::get(&instance).unwrap();
            assert_eq!(buf.to_vec(py).unwrap(), input);
            drop(instance);
            buf
        })
    };

    assert!(!drop_called.load(Ordering::Relaxed));

    Python::attach(|_| {
        drop(buf);
    });

    assert!(drop_called.load(Ordering::Relaxed));
}

#[test]
fn test_releasebuffer_unraisable_error() {
    use pyo3::exceptions::PyValueError;
    use test_utils::UnraisableCapture;

    #[pyclass]
    struct ReleaseBufferError {}

    #[pymethods]
    impl ReleaseBufferError {
        unsafe fn __getbuffer__(
            slf: Bound<'_, Self>,
            view: *mut ffi::Py_buffer,
            flags: c_int,
        ) -> PyResult<()> {
            static BUF_BYTES: &[u8] = b"hello world";
            unsafe { fill_view_from_readonly_data(view, flags, BUF_BYTES, slf.into_any()) }
        }

        unsafe fn __releasebuffer__(&self, _view: *mut ffi::Py_buffer) -> PyResult<()> {
            Err(PyValueError::new_err("oh dear"))
        }
    }

    Python::attach(|py| {
        let instance = Py::new(py, ReleaseBufferError {}).unwrap();

        let (err, object) = UnraisableCapture::enter(py, |capture| {
            let env = [("ob", instance.clone_ref(py))].into_py_dict(py).unwrap();

            assert!(capture.take_capture().is_none());

            py_assert!(py, *env, "bytes(ob) == b'hello world'");

            capture.take_capture().unwrap()
        });

        assert_eq!(err.to_string(), "ValueError: oh dear");
        assert!(object.is(&instance));
    });
}

#[pyclass]
#[derive(Default)]
struct SelfReferentialBuffer {
    releases: usize,
    moved_on_release: bool,
}

#[pymethods]
impl SelfReferentialBuffer {
    unsafe fn __getbuffer__(
        slf: Bound<'_, Self>,
        view: *mut ffi::Py_buffer,
        flags: c_int,
    ) -> PyResult<()> {
        // This helper sets shape and strides to point into the Py_buffer itself.
        unsafe { fill_view_from_readonly_data(view, flags, b"abc", slf.into_any()) }?;
        // Remember the address passed to __getbuffer__ for comparison on release.
        unsafe { (*view).internal = view.cast() };
        Ok(())
    }

    unsafe fn __releasebuffer__(&mut self, view: *mut ffi::Py_buffer) {
        self.releases += 1;
        self.moved_on_release |= unsafe { (*view).internal != view.cast() };
        if !unsafe { (*view).format.is_null() } {
            drop(unsafe { CString::from_raw((*view).format) });
        }
    }
}

fn assert_view_released(instance: &Bound<'_, SelfReferentialBuffer>, releases: usize) {
    let exporter = instance.borrow();
    assert_eq!(exporter.releases, releases);
    assert!(!exporter.moved_on_release);
    drop(exporter);
    // SAFETY: The bound instance is a live Python object on this attached thread.
    assert_eq!(unsafe { ffi::Py_REFCNT(instance.as_ptr()) }, 1);
}

#[test]
fn test_buffer_views_keep_export_in_place() {
    Python::attach(|py| {
        let instance = Bound::new(py, SelfReferentialBuffer::default()).unwrap();

        PyUntypedBufferView::with_flags(&instance, PyBufferRequest::full_ro(), |view| {
            assert_eq!(view.shape(), [3]);
            assert_eq!(view.strides(), [1]);
        })
        .unwrap();
        assert_view_released(&instance, 1);

        PyBufferView::<u8>::with(&instance, |_| {}).unwrap();
        assert_view_released(&instance, 2);

        PyBufferView::<u8>::with_flags(
            &instance,
            PyBufferRequest::simple().strides().format(),
            |_| {},
        )
        .unwrap();
        assert_view_released(&instance, 3);
    });
}

#[test]
fn test_buffer_views_release_after_errors() {
    Python::attach(|py| {
        let instance = Bound::new(py, SelfReferentialBuffer::default()).unwrap();

        let result = PyBufferView::<u32>::with_flags(
            &instance,
            PyBufferRequest::simple().strides().format(),
            |_| panic!("incompatible buffer"),
        );
        assert!(result.unwrap_err().is_instance_of::<PyBufferError>(py));
        assert_view_released(&instance, 1);

        let result = PyUntypedBufferView::with_flags(
            &instance,
            PyBufferRequest::simple().writable(),
            |_| panic!("read-only buffer"),
        );
        assert!(result.unwrap_err().is_instance_of::<PyBufferError>(py));
        assert_view_released(&instance, 1);
    });
}

#[test]
#[cfg(panic = "unwind")]
fn test_buffer_views_release_on_panic() {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    Python::attach(|py| {
        let instance = Bound::new(py, SelfReferentialBuffer::default()).unwrap();

        let result = catch_unwind(AssertUnwindSafe(|| {
            PyUntypedBufferView::with_flags(&instance, PyBufferRequest::full_ro(), |_| {
                panic!("buffer callback panicked")
            })
        }));
        assert!(result.is_err());
        assert_view_released(&instance, 1);
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
    owner: Bound<'_, PyAny>,
) -> PyResult<()> {
    if view.is_null() {
        return Err(PyBufferError::new_err("View is null"));
    }

    if (flags & ffi::PyBUF_WRITABLE) == ffi::PyBUF_WRITABLE {
        return Err(PyBufferError::new_err("Object is not writable"));
    }

    unsafe {
        (*view).obj = owner.into_ptr();

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
    }
    Ok(())
}
