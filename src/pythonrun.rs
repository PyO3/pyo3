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

use std::{sync, rc, marker};
use ffi;
use python::Python;

static START: sync::Once = sync::ONCE_INIT;

/// Prepares the use of Python in a free-threaded context.
///
/// If the Python interpreter is not already initialized, this function
/// will initialize it with disabled signal handling
/// (Python will not raise the `KeyboardInterrupt` exception).
/// Python signal handling depends on the notion of a 'main thread', which must be
/// the thread that initializes the Python interpreter.
///
/// If both the Python interpreter and Python threading are already initialized,
/// this function has no effect.
///
/// # Panic
/// If the Python interpreter is initialized but Python threading is not,
/// a panic occurs.
/// It is not possible to safely access the Python runtime unless the main
/// thread (the thread which originally initialized Python) also initializes
/// threading.
///
/// When writing an extension module, the `py_module_initializer!` macro
/// will ensure that Python threading is initialized.
///
pub fn prepare_freethreaded_python() {
    // Protect against race conditions when Python is not yet initialized
    // and multiple threads concurrently call 'prepare_freethreaded_python()'.
    // Note that we do not protect against concurrent initialization of the Python runtime
    // by other users of the Python C API.
    START.call_once(|| unsafe {
        if ffi::Py_IsInitialized() != 0 {
            // If Python is already initialized, we expect Python threading to also be initialized,
            // as we can't make the existing Python main thread acquire the GIL.
            assert!(ffi::PyEval_ThreadsInitialized() != 0);
        } else {
            // If Python isn't initialized yet, we expect that Python threading isn't initialized either.
            assert!(ffi::PyEval_ThreadsInitialized() == 0);
            // Initialize Python.
            // We use Py_InitializeEx() with initsigs=0 to disable Python signal handling.
            // Signal handling depends on the notion of a 'main thread', which doesn't exist in this case.
            // Note that the 'main thread' notion in Python isn't documented properly;
            // and running Python without one is not officially supported.
            ffi::Py_InitializeEx(0);
            ffi::PyEval_InitThreads();
            // PyEval_InitThreads() will acquire the GIL,
            // but we don't want to hold it at this point
            // (it's not acquired in the other code paths)
            // So immediately release the GIL:
            let _thread_state = ffi::PyEval_SaveThread();
            // Note that the PyThreadState returned by PyEval_SaveThread is also held in TLS by the Python runtime,
            // and will be restored by PyGILState_Ensure.
        }
    });
}

/// RAII type that represents the Global Interpreter Lock acquisition.
///
/// # Example
/// ```
/// use cpython::Python;
///
/// {
///     let gil_guard = Python::acquire_gil();
///     let py = gil_guard.python();
/// } // GIL is released when gil_guard is dropped
/// ```
#[must_use]
pub struct GILGuard {
    gstate: ffi::PyGILState_STATE,
    // hack to opt out of Send on stable rust, which doesn't
    // have negative impls
    no_send: marker::PhantomData<rc::Rc<()>>
}

/// The Drop implementation for GILGuard will release the GIL.
impl Drop for GILGuard {
    fn drop(&mut self) {
        unsafe { ffi::PyGILState_Release(self.gstate) }
    }
}

impl GILGuard {
    /// Acquires the global interpreter lock, which allows access to the Python runtime.
    ///
    /// If the Python runtime is not already initialized, this function will initialize it.
    /// See [prepare_freethreaded_python()](fn.prepare_freethreaded_python.html) for details.
    pub fn acquire() -> GILGuard {
        ::pythonrun::prepare_freethreaded_python();
        let gstate = unsafe { ffi::PyGILState_Ensure() }; // acquire GIL
        GILGuard { gstate: gstate, no_send: marker::PhantomData }
    }

    /// Retrieves the marker type that proves that the GIL was acquired.
    #[inline]
    pub fn python<'p>(&'p self) -> Python<'p> {
        unsafe { Python::assume_gil_acquired() }
    }
}

/// Mutex-like wrapper object for data that is protected by the Python GIL.
///
/// # Example
/// ```
/// use std::cell::Cell;
/// use cpython::{Python, GILProtected};
///
/// let data = GILProtected::new(Cell::new(0));
///
/// {
///     let gil_guard = Python::acquire_gil();
///     let cell = data.get(gil_guard.python());
///     cell.set(cell.get() + 1);
/// }
/// ```
pub struct GILProtected<T> {
    data: T
}

unsafe impl<T: Send> Send for GILProtected<T> { }

/// Because `GILProtected` ensures that the contained data
/// is only accessed while the GIL is acquired,
/// it can implement `Sync` even if the contained data
/// does not.
unsafe impl<T: Send> Sync for GILProtected<T> { }

impl <T> GILProtected<T> {
    /// Creates a new instance of `GILProtected`.
    #[inline]
    #[cfg(feature="nightly")]
    pub const fn new(data: T) -> GILProtected<T> {
        GILProtected { data: data }
    }

    /// Creates a new instance of `GILProtected`.
    #[inline]
    #[cfg(not(feature="nightly"))]
    pub fn new(data: T) -> GILProtected<T> {
        GILProtected { data: data }
    }

    /// Returns a shared reference to the data stored in the `GILProtected`.
    ///
    /// Requires a `Python` instance as proof that the GIL is acquired.
    #[inline]
    pub fn get<'a>(&'a self, _py: Python<'a>) -> &'a T {
        &self.data
    }

    /// Consumes the `GILProtected`, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.data
    }
}

