#[cfg(not(any(PyPy, GraalPy)))]
use crate::{ffi, Python};

static START: std::sync::Once = std::sync::Once::new();

#[cfg(not(any(PyPy, GraalPy)))]
pub(crate) fn initialize() {
    // Protect against race conditions when Python is not yet initialized and multiple threads
    // concurrently call 'initialize()'. Note that we do not protect against
    // concurrent initialization of the Python runtime by other users of the Python C API.
    START.call_once_force(|_| unsafe {
        // Use call_once_force because if initialization panics, it's okay to try again.
        if ffi::Py_IsInitialized() == 0 {
            ffi::Py_InitializeEx(0);

            // Release the GIL.
            ffi::PyEval_SaveThread();
        }
    });
}

#[cfg(any(PyPy, GraalPy))]
pub(crate) fn initialize() {}

#[cfg(not(any(PyPy, GraalPy)))]
pub(crate) fn initialize_embedded() {
    unsafe { ffi::Py_InitializeEx(0) };
}

#[cfg(any(PyPy, GraalPy))]
pub(crate) fn initialize_embedded() {}

#[cfg(not(any(PyPy, GraalPy)))]
#[allow(dead_code)]
pub(crate) fn finalize() {
    unsafe { ffi::Py_Finalize() };
}

#[cfg(any(PyPy, GraalPy))]
#[allow(dead_code)]
pub(crate) fn finalize() {}

#[cfg(not(any(PyPy, GraalPy)))]
pub(crate) fn finalize_embedded() {
    unsafe { ffi::Py_Finalize() };
}

#[cfg(any(PyPy, GraalPy))]
pub(crate) fn finalize_embedded() {}

pub(crate) fn is_initialized() -> bool {
    #[cfg(not(any(PyPy, GraalPy)))]
    {
        unsafe { ffi::Py_IsInitialized() != 0 }
    }

    #[cfg(any(PyPy, GraalPy))]
    {
        let _ = Python::attach;
        false
    }
}

pub(crate) fn wait_for_initialization() {
    // TODO: use START.wait_force() on MSRV 1.86
    // TODO: may not be needed on Python 3.15 (https://github.com/python/cpython/pull/146303)
    START.call_once(|| {
        assert_ne!(unsafe { crate::ffi::Py_IsInitialized() }, 0);
    });
}

#[allow(dead_code)]
pub(crate) fn ensure_initialized_or_panic() {
    START.call_once_force(|_| unsafe {
        assert_ne!(
            crate::ffi::Py_IsInitialized(),
            0,
            "The Python interpreter is not initialized and the `auto-initialize` \
                    feature is not enabled.\n\n\
                    Consider calling `Python::initialize()` before attempting \
                    to use Python APIs."
        );
    });
}

#[cfg(not(any(PyPy, GraalPy)))]
pub(crate) fn prepare_embedded_python_main_thread(py: Python<'_>) {
    py.import("threading").unwrap();
}

#[cfg(any(PyPy, GraalPy))]
pub(crate) fn prepare_embedded_python_main_thread(_: Python<'_>) {}
