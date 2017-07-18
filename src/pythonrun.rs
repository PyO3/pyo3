// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::{sync, rc, marker, mem};
use spin;

use ffi;
use python::Python;
use objects::PyObjectRef;

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
            assert_ne!(ffi::PyEval_ThreadsInitialized(), 0);
        } else {
            // If Python isn't initialized yet, we expect that Python threading isn't initialized either.
            assert_eq!(ffi::PyEval_ThreadsInitialized(), 0);
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
        POINTERS = Box::into_raw(Box::new(Pointers::new()));
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
    owned: usize,
    borrowed: usize,
    gstate: ffi::PyGILState_STATE,
    // hack to opt out of Send on stable rust, which doesn't
    // have negative impls
    no_send: marker::PhantomData<rc::Rc<()>>
}

/// The Drop implementation for `GILGuard` will release the GIL.
impl Drop for GILGuard {
    fn drop(&mut self) {
        unsafe {
            let pool: &'static mut Pointers = &mut *POINTERS;
            pool.drain(self.owned, self.borrowed, true);

            ffi::PyGILState_Release(self.gstate);
        }
    }
}


struct Pointers {
    owned: Vec<*mut ffi::PyObject>,
    borrowed: Vec<*mut ffi::PyObject>,
    pointers: *mut Vec<*mut ffi::PyObject>,
    p: spin::Mutex<*mut Vec<*mut ffi::PyObject>>,
}

impl Pointers {
    fn new() -> Pointers {
        Pointers {
            owned: Vec::with_capacity(250),
            borrowed: Vec::with_capacity(250),
            pointers: Box::into_raw(Box::new(Vec::with_capacity(250))),
            p: spin::Mutex::new(Box::into_raw(Box::new(Vec::with_capacity(250)))),
        }
    }

    unsafe fn release_pointers(&mut self) {
        let mut v = self.p.lock();

        // vec of pointers
        let ptr = *v;
        let vec: &'static mut Vec<*mut ffi::PyObject> = &mut *ptr;
        if vec.is_empty() {
            return
        }

        // switch vectors
        *v = self.pointers;
        self.pointers = ptr;
        drop(v);

        // release py objects
        for ptr in vec.iter_mut() {
            ffi::Py_DECREF(*ptr);
        }
        vec.set_len(0);
    }

    pub unsafe fn drain(&mut self, owned: usize, borrowed: usize, pointers: bool) {
        let len = self.owned.len();
        if owned < len {
            for ptr in &mut self.owned[owned..len] {
                ffi::Py_DECREF(*ptr);
            }
            self.owned.set_len(owned);
        }

        let len = self.borrowed.len();
        if borrowed < len {
            self.borrowed.set_len(borrowed);
        }

        if pointers {
            self.release_pointers();
        }
    }
}

static mut POINTERS: *mut Pointers = ::std::ptr::null_mut();

pub struct Pool {
    owned: usize,
    borrowed: usize,
    pointers: bool,
    no_send: marker::PhantomData<rc::Rc<()>>,
}

impl Pool {
    #[inline]
    pub unsafe fn new() -> Pool {
        let p: &'static mut Pointers = &mut *POINTERS;
        Pool {owned: p.owned.len(),
              borrowed: p.borrowed.len(),
              pointers: true,
              no_send: marker::PhantomData}
    }
    #[inline]
    pub unsafe fn new_no_pointers() -> Pool {
        let p: &'static mut Pointers = &mut *POINTERS;
        Pool {owned: p.owned.len(),
              borrowed: p.borrowed.len(),
              pointers: false,
              no_send: marker::PhantomData}
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        unsafe {
            let pool: &'static mut Pointers = &mut *POINTERS;
            pool.drain(self.owned, self.borrowed, self.pointers);
        }
    }
}


pub unsafe fn register_pointer(obj: *mut ffi::PyObject)
{
    let pool: &'static mut Pointers = &mut *POINTERS;

    let mut v = pool.p.lock();
    let pool: &'static mut Vec<*mut ffi::PyObject> = &mut *(*v);
    pool.push(obj);
}

pub unsafe fn register_owned(_py: Python, obj: *mut ffi::PyObject) -> &PyObjectRef
{
    let pool: &'static mut Pointers = &mut *POINTERS;
    pool.owned.push(obj);
    mem::transmute(&pool.owned[pool.owned.len()-1])
}

pub unsafe fn register_borrowed(_py: Python, obj: *mut ffi::PyObject) -> &PyObjectRef
{
    let pool: &'static mut Pointers = &mut *POINTERS;
    pool.borrowed.push(obj);
    mem::transmute(&pool.borrowed[pool.borrowed.len()-1])
}

impl GILGuard {
    /// Acquires the global interpreter lock, which allows access to the Python runtime.
    ///
    /// If the Python runtime is not already initialized, this function will initialize it.
    /// See [prepare_freethreaded_python()](fn.prepare_freethreaded_python.html) for details.
    pub fn acquire() -> GILGuard {
        prepare_freethreaded_python();

        unsafe {
            let gstate = ffi::PyGILState_Ensure(); // acquire GIL
            let pool: &'static mut Pointers = &mut *POINTERS;
            GILGuard { owned: pool.owned.len(),
                       borrowed: pool.borrowed.len(),
                       gstate: gstate,
                       no_send: marker::PhantomData }
        }
    }

    /// Retrieves the marker type that proves that the GIL was acquired.
    #[inline]
    pub fn python(&self) -> Python {
        unsafe { Python::assume_gil_acquired() }
    }
}

#[cfg(test)]
mod test {
    use std;
    use {ffi, pythonrun};
    use python::Python;
    use object::PyObject;
    use super::{Pool, Pointers, POINTERS};

    #[test]
    fn test_owned() {
        pythonrun::prepare_pyo3_library();

        unsafe {
            let p: &'static mut Pointers = std::mem::transmute(POINTERS);

            let cnt;
            let empty;
            {
                let gil = Python::acquire_gil();
                let py = gil.python();

                empty = ffi::PyTuple_New(0);
                cnt = ffi::Py_REFCNT(empty) - 1;
                let _ = pythonrun::register_owned(py, empty);

                assert_eq!(p.owned.len(), 1);
            }
            {
                let _gil = Python::acquire_gil();
                assert_eq!(p.owned.len(), 0);
                assert_eq!(cnt, ffi::Py_REFCNT(empty));
            }
        }
    }

    #[test]
    fn test_owned_nested() {
        pythonrun::prepare_pyo3_library();

        unsafe {
            let p: &'static mut Pointers = std::mem::transmute(POINTERS);

            let cnt;
            let empty;
            {
                let gil = Python::acquire_gil();
                let py = gil.python();
                assert_eq!(p.owned.len(), 0);

                // empty tuple is singleton
                empty = ffi::PyTuple_New(0);
                cnt = ffi::Py_REFCNT(empty) - 1;
                let _ = pythonrun::register_owned(py, empty);

                assert_eq!(p.owned.len(), 1);

                {
                    let _pool = Pool::new();
                    let empty = ffi::PyTuple_New(0);
                    let _ = pythonrun::register_owned(py, empty);
                    assert_eq!(p.owned.len(), 2);
                }
                assert_eq!(p.owned.len(), 1);
            }
            {
                let _gil = Python::acquire_gil();
                assert_eq!(p.owned.len(), 0);
                assert_eq!(cnt, ffi::Py_REFCNT(empty));
            }
        }
    }

    #[test]
    fn test_borrowed() {
        pythonrun::prepare_pyo3_library();

        unsafe {
            let p: &'static mut Pointers = std::mem::transmute(POINTERS);

            let cnt;
            {
                let gil = Python::acquire_gil();
                let py = gil.python();
                assert_eq!(p.borrowed.len(), 0);

                cnt = ffi::Py_REFCNT(ffi::Py_True());
                pythonrun::register_borrowed(py, ffi::Py_True());

                assert_eq!(p.borrowed.len(), 1);
                assert_eq!(ffi::Py_REFCNT(ffi::Py_True()), cnt);
            }
            {
                let _gil = Python::acquire_gil();
                assert_eq!(p.borrowed.len(), 0);
                assert_eq!(ffi::Py_REFCNT(ffi::Py_True()), cnt);
            }
        }
    }

    #[test]
    fn test_borrowed_nested() {
        pythonrun::prepare_pyo3_library();

        unsafe {
            let p: &'static mut Pointers = std::mem::transmute(POINTERS);

            let cnt;
            {
                let gil = Python::acquire_gil();
                let py = gil.python();
                assert_eq!(p.borrowed.len(), 0);

                cnt = ffi::Py_REFCNT(ffi::Py_True());
                pythonrun::register_borrowed(py, ffi::Py_True());

                assert_eq!(p.borrowed.len(), 1);
                assert_eq!(ffi::Py_REFCNT(ffi::Py_True()), cnt);

                {
                    let _pool = Pool::new();
                    assert_eq!(p.borrowed.len(), 1);
                    pythonrun::register_borrowed(py, ffi::Py_True());
                    assert_eq!(p.borrowed.len(), 2);
                }

                assert_eq!(p.borrowed.len(), 1);
                assert_eq!(ffi::Py_REFCNT(ffi::Py_True()), cnt);
            }
            {
                let _gil = Python::acquire_gil();
                assert_eq!(p.borrowed.len(), 0);
                assert_eq!(ffi::Py_REFCNT(ffi::Py_True()), cnt);
            }
        }
    }

    #[test]
    fn test_pyobject_drop() {
        pythonrun::prepare_pyo3_library();

        unsafe {
            let p: &'static mut Pointers = std::mem::transmute(POINTERS);

            let ob;
            let cnt;
            let empty;
            {
                let gil = Python::acquire_gil();
                let py = gil.python();
                assert_eq!(p.owned.len(), 0);

                // empty tuple is singleton
                empty = ffi::PyTuple_New(0);
                cnt = ffi::Py_REFCNT(empty);
                ob = PyObject::from_owned_ptr(py, empty);
            }
            drop(ob);
            assert_eq!(cnt, ffi::Py_REFCNT(empty));

            {
                let _gil = Python::acquire_gil();
            }
            assert_eq!(cnt - 1, ffi::Py_REFCNT(empty));
        }
    }
}
