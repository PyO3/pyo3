// Copyright (c) 2017-present PyO3 Project and Contributors

//! Interaction with python's global interpreter lock

use crate::{ffi, internal_tricks::Unsendable, Python};
use std::cell::{Cell, RefCell, UnsafeCell};
use std::sync::atomic::{spin_loop_hint, AtomicBool, Ordering};
use std::{any, mem::ManuallyDrop, ptr::NonNull, sync};

static START: sync::Once = sync::Once::new();

thread_local! {
    /// This is a internal counter in pyo3 monitoring whether this thread has the GIL.
    ///
    /// It will be incremented whenever a GIL-holding RAII struct is created, and decremented
    /// whenever they are dropped.
    ///
    /// As a result, if this thread has the GIL, GIL_COUNT is greater than zero.
    ///
    /// pub(crate) because it is manipulated temporarily by Python::allow_threads
    pub(crate) static GIL_COUNT: Cell<u32> = Cell::new(0);

    /// These are objects owned by the current thread, to be released when the GILPool drops.
    static OWNED_OBJECTS: RefCell<Vec<NonNull<ffi::PyObject>>> = RefCell::new(Vec::with_capacity(256));

    /// These are non-python objects such as (String) owned by the current thread, to be released
    /// when the GILPool drops.
    static OWNED_ANYS: RefCell<Vec<Box<dyn any::Any>>> = RefCell::new(Vec::with_capacity(256));
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
            // Must drop the objects in the pool before releasing the GILGuard
            ManuallyDrop::drop(&mut self.pool);
            ffi::PyGILState_Release(self.gstate);
        }
    }
}

/// Thread-safe storage for objects which were dropped while the GIL was not held.
struct ReleasePool {
    locked: AtomicBool,
    pointers_to_drop: UnsafeCell<Vec<NonNull<ffi::PyObject>>>,
}

struct Lock<'a> {
    lock: &'a AtomicBool,
}

impl<'a> Lock<'a> {
    fn new(lock: &'a AtomicBool) -> Self {
        while lock.compare_and_swap(false, true, Ordering::Acquire) {
            spin_loop_hint();
        }
        Self { lock }
    }
}

impl<'a> Drop for Lock<'a> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}

impl ReleasePool {
    const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            pointers_to_drop: UnsafeCell::new(Vec::new()),
        }
    }

    fn register_pointer(&self, obj: NonNull<ffi::PyObject>) {
        let _lock = Lock::new(&self.locked);
        let v = self.pointers_to_drop.get();
        unsafe { (*v).push(obj) };
    }

    fn release_pointers(&self, _py: Python) {
        let _lock = Lock::new(&self.locked);
        let v = self.pointers_to_drop.get();
        unsafe {
            for ptr in &(*v) {
                ffi::Py_DECREF(ptr.as_ptr());
            }
            (*v).clear();
        }
    }
}

unsafe impl Sync for ReleasePool {}

static POOL: ReleasePool = ReleasePool::new();

#[doc(hidden)]
pub struct GILPool {
    owned_objects_start: usize,
    owned_anys_start: usize,
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
        POOL.release_pointers(Python::assume_gil_acquired());
        GILPool {
            owned_objects_start: OWNED_OBJECTS.with(|o| o.borrow().len()),
            owned_anys_start: OWNED_ANYS.with(|o| o.borrow().len()),
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
            OWNED_OBJECTS.with(|owned_objects| {
                // Note: inside this closure we must be careful to not hold a borrow too long, because
                // while calling Py_DECREF we may cause other callbacks to run which will need to
                // register objects into the GILPool.
                let len = owned_objects.borrow().len();
                if self.owned_objects_start < len {
                    let rest = owned_objects
                        .borrow_mut()
                        .split_off(self.owned_objects_start);
                    for obj in rest {
                        ffi::Py_DECREF(obj.as_ptr());
                    }
                }
            });

            OWNED_ANYS.with(|owned_anys| owned_anys.borrow_mut().truncate(self.owned_anys_start));
        }
        decrement_gil_count();
    }
}

/// Register a Python object pointer inside the release pool, to have reference count decreased
/// next time the GIL is acquired in pyo3.
///
/// # Safety
/// The object must be an owned Python reference.
pub unsafe fn register_pointer(obj: NonNull<ffi::PyObject>) {
    if gil_is_acquired() {
        ffi::Py_DECREF(obj.as_ptr())
    } else {
        POOL.register_pointer(obj);
    }
}

/// Register an owned object inside the GILPool.
///
/// # Safety
/// The object must be an owned Python reference.
pub unsafe fn register_owned(_py: Python, obj: NonNull<ffi::PyObject>) {
    debug_assert!(gil_is_acquired());
    OWNED_OBJECTS.with(|objs| objs.borrow_mut().push(obj));
}

/// Register any value inside the GILPool.
///
/// # Safety
/// It is the caller's responsibility to ensure that the inferred lifetime 'p is not inferred by
/// the Rust compiler to outlast the current GILPool.
pub unsafe fn register_any<'p, T: 'static>(obj: T) -> &'p T {
    debug_assert!(gil_is_acquired());
    OWNED_ANYS.with(|owned_anys| {
        let boxed = Box::new(obj);
        let value_ref: &T = &*boxed;

        // Sneaky - extend the lifetime of the reference so that the box can be moved
        let value_ref_extended_lifetime = std::mem::transmute(value_ref);

        owned_anys.borrow_mut().push(boxed);
        value_ref_extended_lifetime
    })
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

#[cfg(test)]
mod test {
    use super::{GILPool, GIL_COUNT, OWNED_OBJECTS, POOL};
    use crate::{ffi, gil, AsPyPointer, IntoPyPointer, PyObject, Python, ToPyObject};
    use std::ptr::NonNull;

    fn get_object() -> PyObject {
        // Convenience function for getting a single unique object
        let gil = Python::acquire_gil();
        let py = gil.python();

        let obj = py.eval("object()", None, None).unwrap();

        obj.to_object(py)
    }

    fn owned_object_count() -> usize {
        OWNED_OBJECTS.with(|objs| objs.borrow().len())
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
            {
                let gil = Python::acquire_gil();
                gil::register_owned(gil.python(), NonNull::new_unchecked(obj.into_ptr()));

                assert_eq!(ffi::Py_REFCNT(obj_ptr), 2);
                assert_eq!(owned_object_count(), 1);
            }
            {
                let _gil = Python::acquire_gil();
                assert_eq!(owned_object_count(), 0);
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
            {
                let _pool = GILPool::new();
                assert_eq!(owned_object_count(), 0);

                gil::register_owned(py, NonNull::new_unchecked(obj.into_ptr()));

                assert_eq!(owned_object_count(), 1);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 2);
                {
                    let _pool = GILPool::new();
                    let obj = get_object();
                    gil::register_owned(py, NonNull::new_unchecked(obj.into_ptr()));
                    assert_eq!(owned_object_count(), 2);
                }
                assert_eq!(owned_object_count(), 1);
            }
            {
                assert_eq!(owned_object_count(), 0);
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
            {
                assert_eq!(owned_object_count(), 0);
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
            {
                assert_eq!(owned_object_count(), 0);
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

    #[test]
    fn test_allow_threads() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let object = get_object();

        py.allow_threads(move || {
            // Should be no pointers to drop
            assert!(unsafe { (*POOL.pointers_to_drop.get()).is_empty() });

            // Dropping object without the GIL should put the pointer in the pool
            drop(object);
            let obj_count = unsafe { (*POOL.pointers_to_drop.get()).len() };
            assert_eq!(obj_count, 1);

            // Now repeat dropping an object, with the GIL.
            let gil = Python::acquire_gil();

            // (Acquiring the GIL should have cleared the pool).
            assert!(unsafe { (*POOL.pointers_to_drop.get()).is_empty() });

            let object = get_object();
            drop(object);
            drop(gil);

            // Previous drop should have decreased count immediately instead of put in pool
            assert!(unsafe { (*POOL.pointers_to_drop.get()).is_empty() });
        })
    }
}
