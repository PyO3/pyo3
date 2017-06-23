// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::{sync, rc, marker, mem};
use ffi;
use python::{Python, ToPyPointer};
use pointer::PyObject;
use objects::PyInstance;

static START: sync::Once = sync::ONCE_INIT;
static START_PYO3: sync::Once = sync::ONCE_INIT;

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

        prepare_pyo3_library();
    });
}

#[doc(hidden)]
pub fn prepare_pyo3_library() {
    START_PYO3.call_once(|| unsafe {
        // initialize release pool
        POOL = Box::into_raw(Box::new(Vec::with_capacity(250)));
    });
}

/// RAII type that represents the Global Interpreter Lock acquisition.
///
/// # Example
/// ```
/// use pyo3::Python;
///
/// {
///     let gil_guard = Python::acquire_gil();
///     let py = gil_guard.python();
/// } // GIL is released when gil_guard is dropped
/// ```
#[must_use]
pub struct GILGuard {
    pos: usize,
    gstate: ffi::PyGILState_STATE,
    // hack to opt out of Send on stable rust, which doesn't
    // have negative impls
    no_send: marker::PhantomData<rc::Rc<()>>
}

/// The Drop implementation for `GILGuard` will release the GIL.
impl Drop for GILGuard {
    fn drop(&mut self) {
        unsafe {
            drain(self.pos);

            ffi::PyGILState_Release(self.gstate);
        }
    }
}

static mut POOL: *mut Vec<PyObject> = 0 as *mut _;

pub struct Pool {
    pos: usize,
    no_send: marker::PhantomData<rc::Rc<()>>,
}

impl Pool {
    #[inline]
    pub unsafe fn new() -> Pool {
        let pool: &'static mut Vec<PyInstance> = mem::transmute(POOL);
        Pool{ pos: pool.len(), no_send: marker::PhantomData }
    }
    // /// Retrieves the marker type that proves that the GIL was acquired.
    // #[inline]
    // pub fn python<'p>(&'p self) -> Python<'p> {
    //    unsafe { Python::assume_gil_acquired() }
    //}
}

impl Drop for Pool {
    fn drop(&mut self) {
        unsafe {
            drain(self.pos);
        }
    }
}

pub unsafe fn register<'p>(_py: Python<'p>, obj: PyObject) -> &'p PyInstance {
    let pool: &'static mut Vec<PyObject> = mem::transmute(POOL);
    pool.push(obj);
    mem::transmute(&pool[pool.len()-1])
}

pub unsafe fn drain(pos: usize) {
    let pool: &'static mut Vec<PyObject> = mem::transmute(POOL);

    let len = pool.len();
    if pos < len {
        for ob in &mut pool[pos..len] {
            ffi::Py_DECREF(ob.as_ptr());
        }
        pool.set_len(pos);
    }
}


impl GILGuard {
    /// Acquires the global interpreter lock, which allows access to the Python runtime.
    ///
    /// If the Python runtime is not already initialized, this function will initialize it.
    /// See [prepare_freethreaded_python()](fn.prepare_freethreaded_python.html) for details.
    pub fn acquire() -> GILGuard {
        ::pythonrun::prepare_freethreaded_python();

        unsafe {
            let gstate = ffi::PyGILState_Ensure(); // acquire GIL
            let pool: &'static mut Vec<PyInstance> = mem::transmute(POOL);

            GILGuard { pos: pool.len(), gstate: gstate, no_send: marker::PhantomData }
        }
    }

    /// Retrieves the marker type that proves that the GIL was acquired.
    #[inline]
    pub fn python<'p>(&'p self) -> Python<'p> {
        unsafe { Python::assume_gil_acquired() }
    }
}
