// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::sync::{Once, ONCE_INIT};
use ffi;
use python::{Python, ToPythonPointer};
use objects::PyObject;

static START: Once = ONCE_INIT;

/// Prepares the use of python in a free-threaded context.
///
/// If the python interpreter is not already initialized, this function
/// will initialize it with disabled signal handling
/// (python will not raise the `KeyboardInterrupt` exception).
/// Python signal handling depends on the notion of a 'main thread', which must be
/// the thread that initializes the python interpreter.
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
    gstate: ffi::PyGILState_STATE
}

/// GILGuard is not Send because the GIL must be released
/// by the same thread that acquired it.
impl !Send for GILGuard {}

/// The Drop implementation for GILGuard will release the GIL.
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
        GILGuard { gstate: gstate }
    }

    /// Retrieves the marker type that proves that the GIL was acquired.
    #[inline]
    pub fn python<'p>(&'p self) -> Python<'p> {
        unsafe { Python::assume_gil_acquired() }
    }
}

/// Mutex-like wrapper object for data that is protected by the python GIL.
pub struct GILProtected<T> {
    data: T
}

unsafe impl<T: Send> Send for GILProtected<T> { }
unsafe impl<T: Send> Sync for GILProtected<T> { }

impl <T> GILProtected<T> {
    #[inline]
    pub const fn new(data: T) -> GILProtected<T> {
        GILProtected { data: data }
    }

    #[inline]
    pub fn get<'p>(&'p self, py: Python<'p>) -> &'p T {
        &self.data
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.data
    }
}

