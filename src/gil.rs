// Copyright (c) 2017-present PyO3 Project and Contributors

//! Interaction with python's global interpreter lock

use crate::{ffi, internal_tricks::Unsendable, PyAny, Python};
use std::cell::{Cell, UnsafeCell};
use std::{any, mem::ManuallyDrop, ptr::NonNull, sync};

static START: sync::Once = sync::Once::new();

thread_local! {
    /// This is a internal counter in pyo3 monitoring whether this thread has the GIL.
    ///
    /// It will be incremented whenever a GIL-holding RAII struct is created, and decremented
    /// whenever they are dropped.
    ///
    /// As a result, if this thread has the GIL, GIL_COUNT is greater than zero.
    static GIL_COUNT: Cell<u32> = Cell::new(0);
}

/// Check whether the GIL is acquired.
///
/// Note: This uses pyo3's internal count rather than PyGILState_Check for two reasons:
///  1) for performance
///  2) PyGILState_Check always returns 1 if the sub-interpreter APIs have ever been called,
///     which could lead to incorrect conclusions that the GIL is held.
fn gil_is_acquired() -> bool {
    GIL_COUNT.with(|c| c.get() > 0)
}

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
    gstate: ffi::PyGILState_STATE,
    pool: ManuallyDrop<GILPool>,
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
            GILGuard {
                gstate,
                pool: ManuallyDrop::new(GILPool::new()),
            }
        }
    }

    /// Retrieves the marker type that proves that the GIL was acquired.
    #[inline]
    pub fn python(&self) -> Python {
        unsafe { Python::assume_gil_acquired() }
    }
}

/// The Drop implementation for `GILGuard` will release the GIL.
impl Drop for GILGuard {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.pool);
            ffi::PyGILState_Release(self.gstate);
        }
    }
}

/// Implementation of release pool
struct ReleasePoolImpl {
    owned: ArrayList<NonNull<ffi::PyObject>>,
    borrowed: ArrayList<NonNull<ffi::PyObject>>,
    pointers: *mut Vec<NonNull<ffi::PyObject>>,
    obj: Vec<Box<dyn any::Any>>,
    p: parking_lot::Mutex<*mut Vec<NonNull<ffi::PyObject>>>,
}

impl ReleasePoolImpl {
    fn new() -> Self {
        Self {
            owned: ArrayList::new(),
            borrowed: ArrayList::new(),
            pointers: Box::into_raw(Box::new(Vec::with_capacity(256))),
            obj: Vec::with_capacity(8),
            p: parking_lot::Mutex::new(Box::into_raw(Box::new(Vec::with_capacity(256)))),
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

    pub unsafe fn drain(&mut self, _py: Python, owned: usize, borrowed: usize) {
        // Release owned objects(call decref)
        while owned < self.owned.len() {
            let last = self.owned.pop_back().unwrap();
            ffi::Py_DECREF(last.as_ptr());
        }
        // Release borrowed objects(don't call decref)
        self.borrowed.truncate(borrowed);
        self.release_pointers();
        self.obj.clear();
    }
}

/// Sync wrapper of ReleasePoolImpl
struct ReleasePool {
    value: UnsafeCell<Option<ReleasePoolImpl>>,
}

impl ReleasePool {
    const fn new() -> Self {
        Self {
            value: UnsafeCell::new(None),
        }
    }
    /// # Safety
    /// This function is not thread safe. Thus, the caller has to have GIL.
    #[allow(clippy::mut_from_ref)]
    unsafe fn get_or_init(&self) -> &mut ReleasePoolImpl {
        (*self.value.get()).get_or_insert_with(ReleasePoolImpl::new)
    }
}

static POOL: ReleasePool = ReleasePool::new();

unsafe impl Sync for ReleasePool {}

#[doc(hidden)]
pub struct GILPool {
    owned: usize,
    borrowed: usize,
    // Stable solution for impl !Send
    no_send: Unsendable,
}

impl GILPool {
    /// # Safety
    /// This function requires that GIL is already acquired.
    #[inline]
    pub unsafe fn new() -> GILPool {
        increment_gil_count();
        // Release objects that were dropped since last GIL acquisition
        let pool = POOL.get_or_init();
        pool.release_pointers();
        GILPool {
            owned: pool.owned.len(),
            borrowed: pool.borrowed.len(),
            no_send: Unsendable::default(),
        }
    }
    pub unsafe fn python(&self) -> Python {
        Python::assume_gil_acquired()
    }
}

impl Drop for GILPool {
    fn drop(&mut self) {
        unsafe {
            let pool = POOL.get_or_init();
            pool.drain(self.python(), self.owned, self.borrowed);
        }
        decrement_gil_count();
    }
}

pub unsafe fn register_any<'p, T: 'static>(obj: T) -> &'p T {
    let pool = POOL.get_or_init();

    pool.obj.push(Box::new(obj));
    pool.obj
        .last()
        .unwrap()
        .as_ref()
        .downcast_ref::<T>()
        .unwrap()
}

pub unsafe fn register_pointer(obj: NonNull<ffi::PyObject>) {
    let pool = POOL.get_or_init();
    if gil_is_acquired() {
        ffi::Py_DECREF(obj.as_ptr())
    } else {
        (**pool.p.lock()).push(obj);
    }
}

pub unsafe fn register_owned(_py: Python, obj: NonNull<ffi::PyObject>) -> &PyAny {
    let pool = POOL.get_or_init();
    &*(pool.owned.push_back(obj) as *const _ as *const PyAny)
}

pub unsafe fn register_borrowed(_py: Python, obj: NonNull<ffi::PyObject>) -> &PyAny {
    let pool = POOL.get_or_init();
    &*(pool.borrowed.push_back(obj) as *const _ as *const PyAny)
}

/// Increment pyo3's internal GIL count - to be called whenever GILPool or GILGuard is created.
#[inline(always)]
fn increment_gil_count() {
    GIL_COUNT.with(|c| c.set(c.get() + 1))
}

/// Decrement pyo3's internal GIL count - to be called whenever GILPool or GILGuard is dropped.
#[inline(always)]
fn decrement_gil_count() {
    GIL_COUNT.with(|c| {
        let current = c.get();
        debug_assert!(
            current > 0,
            "Negative GIL count detected. Please report this error to the PyO3 repo as a bug."
        );
        c.set(current - 1);
    })
}

use self::array_list::ArrayList;

mod array_list {
    use std::collections::LinkedList;
    const BLOCK_SIZE: usize = 256;

    /// A container type for Release Pool
    /// See #271 for why this is crated
    pub(super) struct ArrayList<T> {
        inner: LinkedList<[Option<T>; BLOCK_SIZE]>,
        length: usize,
    }

    impl<T: Copy> ArrayList<T> {
        pub fn new() -> Self {
            ArrayList {
                inner: LinkedList::new(),
                length: 0,
            }
        }
        pub fn push_back(&mut self, item: T) -> &T {
            let next_idx = self.next_idx();
            if next_idx == 0 {
                self.inner.push_back([None; BLOCK_SIZE]);
            }
            self.inner.back_mut().unwrap()[next_idx] = Some(item);
            self.length += 1;
            self.inner.back().unwrap()[next_idx].as_ref().unwrap()
        }
        pub fn pop_back(&mut self) -> Option<T> {
            self.length -= 1;
            let current_idx = self.next_idx();
            if current_idx == 0 {
                let last_list = self.inner.pop_back()?;
                return last_list[0];
            }
            self.inner.back().and_then(|arr| arr[current_idx])
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
    use super::{GILPool, NonNull, GIL_COUNT, POOL};
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
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = get_object();
        let obj_ptr = obj.as_ptr();
        // Ensure that obj does not get freed
        let _ref = obj.clone_ref(py);

        unsafe {
            let p = POOL.get_or_init();

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
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = get_object();
        // Ensure that obj does not get freed
        let _ref = obj.clone_ref(py);
        let obj_ptr = obj.as_ptr();

        unsafe {
            let p = POOL.get_or_init();

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
        unsafe {
            let p = POOL.get_or_init();

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
        unsafe {
            let p = POOL.get_or_init();

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
    fn test_pyobject_drop_with_gil_decreases_refcnt() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = get_object();
        // Ensure that obj does not get freed
        let _ref = obj.clone_ref(py);
        let obj_ptr = obj.as_ptr();

        unsafe {
            let p = POOL.get_or_init();

            {
                assert_eq!(p.owned.len(), 0);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 2);
            }

            // With the GIL held, obj can be dropped immediately
            drop(obj);
            assert_eq!(ffi::Py_REFCNT(obj_ptr), 1);
        }
    }

    #[test]
    fn test_pyobject_drop_without_gil_doesnt_decrease_refcnt() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = get_object();
        // Ensure that obj does not get freed
        let _ref = obj.clone_ref(py);
        let obj_ptr = obj.as_ptr();

        unsafe {
            let p = POOL.get_or_init();

            {
                assert_eq!(p.owned.len(), 0);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 2);
            }

            // Without the GIL held, obj cannot be dropped until the next GIL acquire
            drop(gil);
            drop(obj);
            assert_eq!(ffi::Py_REFCNT(obj_ptr), 2);

            {
                // Next time the GIL is acquired, the object is released
                let _gil = Python::acquire_gil();
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 1);
            }
        }
    }

    #[test]
    fn test_gil_counts() {
        // Check GILGuard and GILPool both increase counts correctly
        let get_gil_count = || GIL_COUNT.with(|c| c.get());

        assert_eq!(get_gil_count(), 0);
        let gil = Python::acquire_gil();
        assert_eq!(get_gil_count(), 1);

        assert_eq!(get_gil_count(), 1);
        let pool = unsafe { GILPool::new() };
        assert_eq!(get_gil_count(), 2);

        let pool2 = unsafe { GILPool::new() };
        assert_eq!(get_gil_count(), 3);

        drop(pool);
        assert_eq!(get_gil_count(), 2);

        let gil2 = Python::acquire_gil();
        assert_eq!(get_gil_count(), 3);

        drop(gil2);
        assert_eq!(get_gil_count(), 2);

        drop(pool2);
        assert_eq!(get_gil_count(), 1);

        drop(gil);
        assert_eq!(get_gil_count(), 0);
    }
}
