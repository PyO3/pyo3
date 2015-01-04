use ffi;

use std::sync::{Once, ONCE_INIT};
use std::thread::Thread;

static START: Once = ONCE_INIT;

/// Prepares the use of python in a free-threaded context.
pub fn prepare_freethreaded_python() {
    START.call_once(|| unsafe {
        ::ffi::Py_InitializeEx(0);
        if ::ffi::PyEval_ThreadsInitialized() == 0 {
            ::ffi::PyEval_InitThreads();
            // InitThreads() will acquire the GIL,
            // but we don't want to acquire it at this point
            // (it's not acquired in the other code paths)
            ::ffi::PyEval_ReleaseLock();
        }
    });
}

