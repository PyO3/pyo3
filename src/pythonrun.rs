use std::sync::{Once, ONCE_INIT};
use std::thread::Thread;
use ffi;
use python::Python;

static START: Once = ONCE_INIT;

/// Prepares the use of python in a free-threaded context.
pub fn prepare_freethreaded_python() {
    // Protect against race conditions when python is not yet initialized
    // and multiple threads concurrently call 'prepare_freethreaded_python()'.
    // Note that we do not protect against concurrent initialization of the python runtime
    // by other users of the python C API.
    START.call_once(|| unsafe {
        if ffi::Py_IsInitialized() != 0 {
            // If python is already initialized, we expect python threading to also be initialized,
            // as we can't make the existing python main thread acquire the GIL.
            assert!(ffi::PyEval_ThreadsInitialized() != 0);
        } else {
            // If python isn't initialized yet, we expect that python threading isn't initialized either.
            assert!(ffi::PyEval_ThreadsInitialized() == 0);
            // Initialize python.
            // We use Py_InitializeEx() with initsigs=0 to disable Python signal handling.
            // Signal handling depends on the notion of a 'main thread', which doesn't exist in this case.
            // Note that the 'main thread' notion in python isn't documented properly;
            // and running python without one is not officially supported.
            ffi::Py_InitializeEx(0);
            ffi::PyEval_InitThreads();
            // PyEval_InitThreads() will acquire the GIL,
            // but we don't want to hold it at this point
            // (it's not acquired in the other code paths)
            // So immediately release the GIL:
            let _thread_state = ffi::PyEval_SaveThread();
            // Note that the PyThreadState returned by PyEval_SaveThread is also held in TLS by the python runtime,
            // and will be restored by PyGILState_Ensure.
        }
    });
}

/// RAII type that represents an acquired GIL.
#[must_use]
pub struct GILGuard {
    gstate: ffi::PyGILState_STATE,
    nosend: ::std::marker::NoSend
}

//impl !Send for GILGuard {}

impl Drop for GILGuard {
    fn drop(&mut self) {
        unsafe { ffi::PyGILState_Release(self.gstate) }
    }
}

impl GILGuard {
    /// Acquires the global interpreter lock, which allows access to the Python runtime.
    /// If the python runtime is not already initialized, this function will initialize it.
    /// Note that in this case, the python runtime will not have any main thread, and will
    /// not deliver signals like KeyboardInterrupt.
    pub fn acquire() -> GILGuard {
        ::pythonrun::prepare_freethreaded_python();
        let gstate = unsafe { ffi::PyGILState_Ensure() }; // acquire GIL
        GILGuard { gstate: gstate, nosend: ::std::marker::NoSend }
    }
    
    pub fn python<'p>(&'p self) -> Python<'p> {
        unsafe { Python::assume_gil_acquired() }
    }
}

