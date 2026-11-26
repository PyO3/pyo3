#![cfg(all(Py_3_13, not(any(PyPy, GraalPy))))]
#![cfg(not(target_arch = "wasm32"))]

use pyo3::{Python, ffi};
use std::sync::atomic::{AtomicBool, Ordering};

static CHECK_PASSED: AtomicBool = AtomicBool::new(false);

extern "C" fn check_attach_during_finalization() {
    let result = std::panic::catch_unwind(|| {
        // SAFETY: both functions are always safe to call.
        unsafe {
            assert_eq!(ffi::Py_IsInitialized(), 0);
            assert_ne!(ffi::Py_IsFinalizing(), 0);
        }
        assert!(Python::try_attach(|_| panic!("unexpected attachment")).is_none());
        let panic = std::panic::catch_unwind(|| Python::attach(|_| {})).unwrap_err();
        assert_eq!(
            panic.downcast_ref::<&str>().copied(),
            Some("Cannot attach to the Python interpreter while it is finalizing.")
        );
    });
    CHECK_PASSED.store(result.is_ok(), Ordering::Relaxed);
}

#[test]
fn test_attach_during_finalization() {
    Python::initialize();
    Python::attach(|_| {
        assert_eq!(
            // SAFETY: the callback matches Py_AtExit's C signature and catches panics
            // before returning to CPython.
            unsafe { ffi::Py_AtExit(Some(check_attach_during_finalization)) },
            0
        );
    });

    // SAFETY: Python is initialized
    unsafe {
        ffi::PyGILState_Ensure();
    }
    // SAFETY: on the thread that initialized Python, with the main interpreter active
    unsafe {
        assert_eq!(ffi::Py_FinalizeEx(), 0);
    }
    assert!(CHECK_PASSED.load(Ordering::Relaxed));
}
