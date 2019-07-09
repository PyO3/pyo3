// Copyright (c) 2017-present PyO3 Project and Contributors

//! Interaction with python's global interpreter lock

use crate::ffi;
use crate::types::PyAny;
use crate::Python;
use spin;
use std::ptr::NonNull;
use std::{any, marker, rc, sync};

static START: sync::Once = sync::Once::new();
static START_PYO3: sync::Once = sync::Once::new();

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
/// When writing an extension module, the `#[pymodule]` macro
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
            #[cfg(not(Py_3_7))]
            assert_ne!(ffi::PyEval_ThreadsInitialized(), 0);
        } else {
            // If Python isn't initialized yet, we expect that Python threading
            // isn't initialized either.
            #[cfg(not(Py_3_7))]
            assert_eq!(ffi::PyEval_ThreadsInitialized(), 0);
            // Initialize Python.
            // We use Py_InitializeEx() with initsigs=0 to disable Python signal handling.
            // Signal handling depends on the notion of a 'main thread', which doesn't exist in this case.
            // Note that the 'main thread' notion in Python isn't documented properly;
            // and running Python without one is not officially supported.

            // PyPy does not support the embedding API
            #[cfg(not(PyPy))]
            ffi::Py_InitializeEx(0);

            // > Changed in version 3.7: This function is now called by Py_Initialize(), so you don’t have
            // > to call it yourself anymore.
            #[cfg(not(Py_3_7))]
            ffi::PyEval_InitThreads();
            // PyEval_InitThreads() will acquire the GIL,
            // but we don't want to hold it at this point
            // (it's not acquired in the other code paths)
            // So immediately release the GIL:
            #[cfg(not(PyPy))]
            let _thread_state = ffi::PyEval_SaveThread();
            // Note that the PyThreadState returned by PyEval_SaveThread is also held in TLS by the Python runtime,
            // and will be restored by PyGILState_Ensure.
        }

        init_once();
    });
}

#[doc(hidden)]
pub fn init_once() {
    START_PYO3.call_once(|| unsafe {
        // initialize release pool
        POOL = Box::into_raw(Box::new(ReleasePool::new()));
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
    no_send: marker::PhantomData<rc::Rc<()>>,
}

/// The Drop implementation for `GILGuard` will release the GIL.
impl Drop for GILGuard {
    fn drop(&mut self) {
        unsafe {
            let pool: &'static mut ReleasePool = &mut *POOL;
            pool.drain(self.owned, self.borrowed, true);

            ffi::PyGILState_Release(self.gstate);
        }
    }
}

/// Release pool
struct ReleasePool {
    owned: ArrayList<NonNull<ffi::PyObject>>,
    borrowed: ArrayList<NonNull<ffi::PyObject>>,
    pointers: *mut Vec<NonNull<ffi::PyObject>>,
    obj: Vec<Box<dyn any::Any>>,
    p: spin::Mutex<*mut Vec<NonNull<ffi::PyObject>>>,
}

impl ReleasePool {
    fn new() -> ReleasePool {
        ReleasePool {
            owned: ArrayList::new(),
            borrowed: ArrayList::new(),
            pointers: Box::into_raw(Box::new(Vec::with_capacity(256))),
            obj: Vec::with_capacity(8),
            p: spin::Mutex::new(Box::into_raw(Box::new(Vec::with_capacity(256)))),
        }
    }

    unsafe fn release_pointers(&mut self) {
        let mut v = self.p.lock();
        let vec = &mut **v;
        if vec.is_empty() {
            return;
        }

        // switch vectors
        std::mem::swap(&mut self.pointers, &mut *v);
        drop(v);

        // release PyObjects
        for ptr in vec.iter_mut() {
            ffi::Py_DECREF(ptr.as_ptr());
        }
        vec.set_len(0);
    }

    pub unsafe fn drain(&mut self, owned: usize, borrowed: usize, pointers: bool) {
        // Release owned objects(call decref)
        while owned < self.owned.len() {
            let last = self.owned.pop_back().unwrap();
            ffi::Py_DECREF(last.as_ptr());
        }
        // Release borrowed objects(don't call decref)
        self.borrowed.truncate(borrowed);

        if pointers {
            self.release_pointers();
        }

        self.obj.clear();
    }
}

static mut POOL: *mut ReleasePool = ::std::ptr::null_mut();

#[doc(hidden)]
pub struct GILPool {
    owned: usize,
    borrowed: usize,
    pointers: bool,
    no_send: marker::PhantomData<rc::Rc<()>>,
}

impl Default for GILPool {
    #[inline]
    fn default() -> GILPool {
        let p: &'static mut ReleasePool = unsafe { &mut *POOL };
        GILPool {
            owned: p.owned.len(),
            borrowed: p.borrowed.len(),
            pointers: true,
            no_send: marker::PhantomData,
        }
    }
}

impl GILPool {
    #[inline]
    pub fn new() -> GILPool {
        GILPool::default()
    }
    #[inline]
    pub fn new_no_pointers() -> GILPool {
        let p: &'static mut ReleasePool = unsafe { &mut *POOL };
        GILPool {
            owned: p.owned.len(),
            borrowed: p.borrowed.len(),
            pointers: false,
            no_send: marker::PhantomData,
        }
    }
}

impl Drop for GILPool {
    fn drop(&mut self) {
        unsafe {
            let pool: &'static mut ReleasePool = &mut *POOL;
            pool.drain(self.owned, self.borrowed, self.pointers);
        }
    }
}

pub unsafe fn register_any<'p, T: 'static>(obj: T) -> &'p T {
    let pool: &'static mut ReleasePool = &mut *POOL;

    pool.obj.push(Box::new(obj));
    pool.obj
        .last()
        .unwrap()
        .as_ref()
        .downcast_ref::<T>()
        .unwrap()
}

pub unsafe fn register_pointer(obj: NonNull<ffi::PyObject>) {
    let pool = &mut *POOL;
    (**pool.p.lock()).push(obj);
}

pub unsafe fn register_owned(_py: Python, obj: NonNull<ffi::PyObject>) -> &PyAny {
    let pool = &mut *POOL;
    &*(pool.owned.push_back(obj) as *const _ as *const PyAny)
}

pub unsafe fn register_borrowed(_py: Python, obj: NonNull<ffi::PyObject>) -> &PyAny {
    let pool = &mut *POOL;
    &*(pool.borrowed.push_back(obj) as *const _ as *const PyAny)
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
            let pool: &'static mut ReleasePool = &mut *POOL;
            GILGuard {
                owned: pool.owned.len(),
                borrowed: pool.borrowed.len(),
                gstate,
                no_send: marker::PhantomData,
            }
        }
    }

    /// Retrieves the marker type that proves that the GIL was acquired.
    #[inline]
    pub fn python(&self) -> Python {
        unsafe { Python::assume_gil_acquired() }
    }
}

use self::array_list::ArrayList;

mod array_list {
    use std::collections::LinkedList;
    use std::mem;

    const BLOCK_SIZE: usize = 256;

    /// A container type for Release Pool
    /// See #271 for why this is crated
    pub(super) struct ArrayList<T> {
        inner: LinkedList<[T; BLOCK_SIZE]>,
        length: usize,
    }

    impl<T: Clone> ArrayList<T> {
        pub fn new() -> Self {
            ArrayList {
                inner: LinkedList::new(),
                length: 0,
            }
        }
        pub fn push_back(&mut self, item: T) -> &T {
            let next_idx = self.next_idx();
            if next_idx == 0 {
                self.inner
                    .push_back(unsafe { mem::MaybeUninit::uninit().assume_init() });
            }
            self.inner.back_mut().unwrap()[next_idx] = item;
            self.length += 1;
            &self.inner.back().unwrap()[next_idx]
        }
        pub fn pop_back(&mut self) -> Option<T> {
            self.length -= 1;
            let current_idx = self.next_idx();
            if current_idx == 0 {
                let last_list = self.inner.pop_back()?;
                return Some(last_list[0].clone());
            }
            self.inner.back().map(|arr| arr[current_idx].clone())
        }
        pub fn len(&self) -> usize {
            self.length
        }
        pub fn truncate(&mut self, new_len: usize) {
            if self.length <= new_len {
                return;
            }
            while self.inner.len() > (new_len + BLOCK_SIZE - 1) / BLOCK_SIZE {
                self.inner.pop_back();
            }
            self.length = new_len;
        }
        fn next_idx(&self) -> usize {
            self.length % BLOCK_SIZE
        }
    }
}

#[cfg(test)]
mod test {
    use super::{GILPool, NonNull, ReleasePool, POOL};
    use crate::object::PyObject;
    use crate::AsPyPointer;
    use crate::Python;
    use crate::ToPyObject;
    use crate::{ffi, gil};

    fn get_object() -> PyObject {
        // Convenience function for getting a single unique object
        let gil = Python::acquire_gil();
        let py = gil.python();

        let obj = py.eval("object()", None, None).unwrap();

        obj.to_object(py)
    }

    #[test]
    fn test_owned() {
        gil::init_once();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = get_object();
        let obj_ptr = obj.as_ptr();
        // Ensure that obj does not get freed
        let _ref = obj.clone_ref(py);

        unsafe {
            let p: &'static mut ReleasePool = &mut *POOL;

            {
                let gil = Python::acquire_gil();
                let py = gil.python();
                let _ = gil::register_owned(py, obj.into_nonnull());

                assert_eq!(ffi::Py_REFCNT(obj_ptr), 2);
                assert_eq!(p.owned.len(), 1);
            }
            {
                let _gil = Python::acquire_gil();
                assert_eq!(p.owned.len(), 0);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 1);
            }
        }
    }

    #[test]
    fn test_owned_nested() {
        gil::init_once();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = get_object();
        // Ensure that obj does not get freed
        let _ref = obj.clone_ref(py);
        let obj_ptr = obj.as_ptr();

        unsafe {
            let p: &'static mut ReleasePool = &mut *POOL;

            {
                let _pool = GILPool::new();
                assert_eq!(p.owned.len(), 0);

                let _ = gil::register_owned(py, obj.into_nonnull());

                assert_eq!(p.owned.len(), 1);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 2);
                {
                    let _pool = GILPool::new();
                    let obj = get_object();
                    let _ = gil::register_owned(py, obj.into_nonnull());
                    assert_eq!(p.owned.len(), 2);
                }
                assert_eq!(p.owned.len(), 1);
            }
            {
                assert_eq!(p.owned.len(), 0);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 1);
            }
        }
    }

    #[test]
    fn test_borrowed() {
        gil::init_once();

        unsafe {
            let p: &'static mut ReleasePool = &mut *POOL;

            let obj = get_object();
            let obj_ptr = obj.as_ptr();
            {
                let gil = Python::acquire_gil();
                let py = gil.python();
                assert_eq!(p.borrowed.len(), 0);

                gil::register_borrowed(py, NonNull::new(obj_ptr).unwrap());

                assert_eq!(p.borrowed.len(), 1);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 1);
            }
            {
                let _gil = Python::acquire_gil();
                assert_eq!(p.borrowed.len(), 0);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 1);
            }
        }
    }

    #[test]
    fn test_borrowed_nested() {
        gil::init_once();

        unsafe {
            let p: &'static mut ReleasePool = &mut *POOL;

            let obj = get_object();
            let obj_ptr = obj.as_ptr();
            {
                let gil = Python::acquire_gil();
                let py = gil.python();
                assert_eq!(p.borrowed.len(), 0);

                gil::register_borrowed(py, NonNull::new(obj_ptr).unwrap());

                assert_eq!(p.borrowed.len(), 1);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 1);

                {
                    let _pool = GILPool::new();
                    assert_eq!(p.borrowed.len(), 1);
                    gil::register_borrowed(py, NonNull::new(obj_ptr).unwrap());
                    assert_eq!(p.borrowed.len(), 2);
                }

                assert_eq!(p.borrowed.len(), 1);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 1);
            }
            {
                let _gil = Python::acquire_gil();
                assert_eq!(p.borrowed.len(), 0);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 1);
            }
        }
    }

    #[test]
    fn test_pyobject_drop() {
        gil::init_once();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = get_object();
        // Ensure that obj does not get freed
        let _ref = obj.clone_ref(py);
        let obj_ptr = obj.as_ptr();

        unsafe {
            let p: &'static mut ReleasePool = &mut *POOL;

            {
                assert_eq!(p.owned.len(), 0);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 2);
            }
            drop(obj);
            assert_eq!(ffi::Py_REFCNT(obj_ptr), 2);

            {
                let _gil = Python::acquire_gil();
            }
            assert_eq!(ffi::Py_REFCNT(obj_ptr), 1);
        }
    }
}
