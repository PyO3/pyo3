// Copyright (c) 2017-present PyO3 Project and Contributors

//! Interaction with Python's global interpreter lock

use crate::{ffi, internal_tricks::Unsendable, Python};
use parking_lot::{const_mutex, Mutex, Once};
use std::cell::{Cell, RefCell};
use std::{
    mem::{self, ManuallyDrop},
    ptr::NonNull,
    sync::atomic,
};

static START: Once = Once::new();

thread_local! {
    /// This is an internal counter in pyo3 monitoring whether this thread has the GIL.
    ///
    /// It will be incremented whenever a GILGuard or GILPool is created, and decremented whenever
    /// they are dropped.
    ///
    /// As a result, if this thread has the GIL, GIL_COUNT is greater than zero.
    ///
    /// pub(crate) because it is manipulated temporarily by `Python::allow_threads`.
    pub(crate) static GIL_COUNT: Cell<usize> = Cell::new(0);

    /// Temporarily hold objects that will be released when the GILPool drops.
    static OWNED_OBJECTS: RefCell<Vec<NonNull<ffi::PyObject>>> = RefCell::new(Vec::with_capacity(256));
}

/// Checks whether the GIL is acquired.
///
/// Note: This uses pyo3's internal count rather than PyGILState_Check for two reasons:
///  1) for performance
///  2) PyGILState_Check always returns 1 if the sub-interpreter APIs have ever been called,
///     which could lead to incorrect conclusions that the GIL is held.
pub(crate) fn gil_is_acquired() -> bool {
    GIL_COUNT.try_with(|c| c.get() > 0).unwrap_or(false)
}

/// Prepares the use of Python in a free-threaded context.
///
/// If the Python interpreter is not already initialized, this function will initialize it with
/// signal handling disabled (Python will not raise the `KeyboardInterrupt` exception). Python
/// signal handling depends on the notion of a 'main thread', which must be the thread that
/// initializes the Python interpreter.
///
/// If both the Python interpreter and Python threading are already initialized, this function has
/// no effect.
///
/// This function is unavailable under PyPy because PyPy cannot be embedded in Rust (or any other
/// software). Support for this is tracked on the
/// [PyPy issue tracker](https://foss.heptapod.net/pypy/pypy/-/issues/3286).
///
/// Python 3.6 only: If the Python interpreter is initialized but Python threading is not, this
/// function will initialize Python threading.
///
/// # Panics
/// - Python 3.6 only: If this function needs to initialize Python threading but the calling thread
///   is not the thread which initialized Python, this function will panic.
///
/// # Examples
/// ```rust
/// use pyo3::prelude::*;
///
/// # fn main() -> PyResult<()>{
/// pyo3::prepare_freethreaded_python();
/// Python::with_gil(|py| py.run("print('Hello World')", None, None))
/// # }
/// ```
#[cfg(not(PyPy))]
#[allow(clippy::collapsible_if)] // for if cfg!
pub fn prepare_freethreaded_python() {
    // Protect against race conditions when Python is not yet initialized and multiple threads
    // concurrently call 'prepare_freethreaded_python()'. Note that we do not protect against
    // concurrent initialization of the Python runtime by other users of the Python C API.
    START.call_once_force(|_| unsafe {
        if cfg!(not(Py_3_7)) {
            // Use call_once_force because if initialization panics, it's okay to try again.
            if ffi::Py_IsInitialized() != 0 {
                if ffi::PyEval_ThreadsInitialized() == 0 {
                    // We can only safely initialize threads if this thread holds the GIL.
                    assert!(
                        !ffi::PyGILState_GetThisThreadState().is_null(),
                        "Python threading is not initialized and cannot be initialized by this \
                         thread, because it is not the thread which initialized Python."
                    );
                    ffi::PyEval_InitThreads();
                }
            } else {
                ffi::Py_InitializeEx(0);
                ffi::PyEval_InitThreads();

                // Release the GIL.
                ffi::PyEval_SaveThread();
            }
        } else if ffi::Py_IsInitialized() == 0 {
            // In Python 3.7 and up PyEval_InitThreads is irrelevant.
            ffi::Py_InitializeEx(0);

            // Release the GIL.
            ffi::PyEval_SaveThread();
        }
    });
}

/// Executes the provided closure with an embedded Python interpreter.
///
/// This function initializes the Python interpreter, executes the provided closure, and then
/// finalizes the Python interpreter.
///
/// After execution all Python resources are cleaned up, and no further Python APIs can be called.
/// Because many Python modules implemented in C do not support multiple Python interpreters in a
/// single process, it is not safe to call this function more than once. (Many such modules will not
/// initialize correctly on the second run.)
///
/// # Panics
/// - If the Python interpreter is already initialized before calling this function.
///
/// # Safety
/// - This function should only ever be called once per process (usually as part of the `main`
///   function). It is also not thread-safe.
/// - No Python APIs can be used after this function has finished executing.
/// - The return value of the closure must not contain any Python value, _including_ `PyResult`.
///
/// # Examples
/// ```rust
/// use pyo3::prelude::*;
///
/// # fn main() -> PyResult<()>{
/// unsafe {
///     pyo3::with_embedded_python_interpreter(|py| py.run("print('Hello World')", None, None))
/// }
/// # }
/// ```
#[cfg(not(PyPy))]
#[allow(clippy::collapsible_if)] // for if cfg!
pub unsafe fn with_embedded_python_interpreter<F, R>(f: F) -> R
where
    F: for<'p> FnOnce(Python<'p>) -> R,
{
    assert_eq!(
        ffi::Py_IsInitialized(),
        0,
        "called `with_embedded_python_interpreter` but a Python interpreter is already running."
    );

    ffi::Py_InitializeEx(0);

    // Changed in version 3.7: This function is now called by Py_Initialize(), so you don’t have to
    // call it yourself anymore.
    if cfg!(not(Py_3_7)) {
        if ffi::PyEval_ThreadsInitialized() == 0 {
            ffi::PyEval_InitThreads();
        }
    }

    // Safe: the GIL is already held because of the Py_IntializeEx call.
    let pool = GILPool::new();

    // Import the threading module - this ensures that it will associate this thread as the "main"
    // thread, which is important to avoid an `AssertionError` at finalization.
    pool.python().import("threading").unwrap();

    // Execute the closure.
    let result = f(pool.python());

    // Drop the pool before finalizing.
    drop(pool);

    // Finalize the Python interpreter.
    ffi::Py_Finalize();

    result
}

/// RAII type that represents the Global Interpreter Lock acquisition.
///
/// Users are strongly encouraged to use [`Python::with_gil`](struct.Python.html#method.with_gil)
/// instead of directly constructing this type.
/// See [`Python::acquire_gil`](struct.Python.html#method.acquire_gil) for more.
///
/// # Examples
/// ```
/// use pyo3::Python;
///
/// {
///     let gil_guard = Python::acquire_gil();
///     let py = gil_guard.python();
/// } // GIL is released when gil_guard is dropped
/// ```
#[allow(clippy::upper_case_acronyms)]
#[must_use]
pub struct GILGuard {
    gstate: ffi::PyGILState_STATE,
    pool: ManuallyDrop<Option<GILPool>>,
}

impl GILGuard {
    /// Retrieves the marker type that proves that the GIL was acquired.
    #[inline]
    pub fn python(&self) -> Python {
        unsafe { Python::assume_gil_acquired() }
    }

    /// PyO3 internal API for acquiring the GIL. The public API is Python::acquire_gil.
    ///
    /// If PyO3 does not yet have a `GILPool` for tracking owned PyObject references, then this new
    /// `GILGuard` will also contain a `GILPool`.
    pub(crate) fn acquire() -> GILGuard {
        // Maybe auto-initialize the GIL:
        //  - If auto-initialize feature set and supported, try to initalize the interpreter.
        //  - If the auto-initialize feature is set but unsupported, emit hard errors only when the
        //    extension-module feature is not activated - extension modules don't care about
        //    auto-initialize so this avoids breaking existing builds.
        //  - Otherwise, just check the GIL is initialized.
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "auto-initialize", not(PyPy)))] {
                prepare_freethreaded_python();
            } else {
                START.call_once_force(|_| unsafe {
                    // Use call_once_force because if there is a panic because the interpreter is
                    // not initialized, it's fine for the user to initialize the interpreter and
                    // retry.
                    assert_ne!(
                        ffi::Py_IsInitialized(),
                        0,
                        "The Python interpreter is not initalized and the `auto-initialize` \
                         feature is not enabled.\n\n\
                         Consider calling `pyo3::prepare_freethreaded_python()` before attempting \
                         to use Python APIs."
                    );
                    assert_ne!(
                        ffi::PyEval_ThreadsInitialized(),
                        0,
                        "Python threading is not initalized and the `auto-initialize` feature is \
                         not enabled.\n\n\
                         Consider calling `pyo3::prepare_freethreaded_python()` before attempting \
                         to use Python APIs."
                    );
                });
            }
        }

        Self::acquire_unchecked()
    }

    /// Acquires the `GILGuard` without performing any state checking.
    ///
    /// This can be called in "unsafe" contexts where the normal interpreter state
    /// checking performed by `GILGuard::acquire` may fail. This includes calling
    /// as part of multi-phase interpreter initialization.
    pub(crate) fn acquire_unchecked() -> GILGuard {
        let gstate = unsafe { ffi::PyGILState_Ensure() }; // acquire GIL

        // If there's already a GILPool, we should not create another or this could lead to
        // incorrect dangling references in safe code (see #864).
        let pool = if !gil_is_acquired() {
            Some(unsafe { GILPool::new() })
        } else {
            // As no GILPool was created, need to update the gil count manually.
            increment_gil_count();
            None
        };

        GILGuard {
            gstate,
            pool: ManuallyDrop::new(pool),
        }
    }
}

/// The Drop implementation for `GILGuard` will release the GIL.
impl Drop for GILGuard {
    fn drop(&mut self) {
        // First up, try to detect if the order of destruction is correct.
        let _ = GIL_COUNT.try_with(|c| {
            if self.gstate == ffi::PyGILState_STATE::PyGILState_UNLOCKED && c.get() != 1 {
                // XXX: this panic commits to leaking all objects in the pool as well as
                // potentially meaning the GIL never releases. Perhaps should be an abort?
                // Unfortunately abort UX is much worse than panic.
                panic!("The first GILGuard acquired must be the last one dropped.");
            }
        });

        // If this GILGuard doesn't own a pool, then need to decrease the count after dropping
        // all objects from the pool.
        let should_decrement = self.pool.is_none();

        // Drop the objects in the pool before attempting to release the thread state
        unsafe {
            ManuallyDrop::drop(&mut self.pool);
        }

        if should_decrement {
            decrement_gil_count();
        }

        unsafe {
            ffi::PyGILState_Release(self.gstate);
        }
    }
}

// Vector of PyObject(
type PyObjVec = Vec<NonNull<ffi::PyObject>>;

/// Thread-safe storage for objects which were inc_ref / dec_ref while the GIL was not held.
struct ReferencePool {
    dirty: atomic::AtomicBool,
    // .0 is INCREFs, .1 is DECREFs
    pointer_ops: Mutex<(PyObjVec, PyObjVec)>,
}

impl ReferencePool {
    const fn new() -> Self {
        Self {
            dirty: atomic::AtomicBool::new(false),
            pointer_ops: const_mutex((Vec::new(), Vec::new())),
        }
    }

    fn register_incref(&self, obj: NonNull<ffi::PyObject>) {
        self.pointer_ops.lock().0.push(obj);
        self.dirty.store(true, atomic::Ordering::Release);
    }

    fn register_decref(&self, obj: NonNull<ffi::PyObject>) {
        self.pointer_ops.lock().1.push(obj);
        self.dirty.store(true, atomic::Ordering::Release);
    }

    fn update_counts(&self, _py: Python) {
        let prev = self.dirty.swap(false, atomic::Ordering::Acquire);
        if !prev {
            return;
        }

        let mut ops = self.pointer_ops.lock();
        let (increfs, decrefs) = mem::take(&mut *ops);
        drop(ops);
        // Always increase reference counts first - as otherwise objects which have a
        // nonzero total reference count might be incorrectly dropped by Python during
        // this update.
        for ptr in increfs {
            unsafe { ffi::Py_INCREF(ptr.as_ptr()) };
        }

        for ptr in decrefs {
            unsafe { ffi::Py_DECREF(ptr.as_ptr()) };
        }
    }
}

unsafe impl Sync for ReferencePool {}

static POOL: ReferencePool = ReferencePool::new();

/// A RAII pool which PyO3 uses to store owned Python references.
///
/// See the [Memory Management] chapter of the guide for more information about how PyO3 uses
/// [`GILPool`] to manage memory.

///
/// [Memory Management]: https://pyo3.rs/main/memory.html#gil-bound-memory
#[allow(clippy::upper_case_acronyms)]
pub struct GILPool {
    /// Initial length of owned objects and anys.
    /// `Option` is used since TSL can be broken when `new` is called from `atexit`.
    start: Option<usize>,
    no_send: Unsendable,
}

impl GILPool {
    /// Creates a new [`GILPool`]. This function should only ever be called with the GIL held.
    ///
    /// It is recommended not to use this API directly, but instead to use [`Python::new_pool`], as
    /// that guarantees the GIL is held.
    ///
    /// # Safety
    ///
    /// As well as requiring the GIL, see the safety notes on [`Python::new_pool`].
    #[inline]
    pub unsafe fn new() -> GILPool {
        increment_gil_count();
        // Update counts of PyObjects / Py that have been cloned or dropped since last acquisition
        POOL.update_counts(Python::assume_gil_acquired());
        GILPool {
            start: OWNED_OBJECTS.try_with(|o| o.borrow().len()).ok(),
            no_send: Unsendable::default(),
        }
    }

    /// Gets the Python token associated with this [`GILPool`].
    pub fn python(&self) -> Python {
        unsafe { Python::assume_gil_acquired() }
    }
}

impl Drop for GILPool {
    fn drop(&mut self) {
        if let Some(obj_len_start) = self.start {
            let dropping_obj = OWNED_OBJECTS.with(|holder| {
                // `holder` must be dropped before calling Py_DECREF, or Py_DECREF may call
                // `GILPool::drop` recursively, resulting in invalid borrowing.
                let mut holder = holder.borrow_mut();
                if obj_len_start < holder.len() {
                    holder.split_off(obj_len_start)
                } else {
                    Vec::new()
                }
            });
            for obj in dropping_obj {
                unsafe {
                    ffi::Py_DECREF(obj.as_ptr());
                }
            }
        }
        decrement_gil_count();
    }
}

/// Registers a Python object pointer inside the release pool, to have its reference count increased
/// the next time the GIL is acquired in pyo3.
///
/// If the GIL is held, the reference count will be increased immediately instead of being queued
/// for later.
///
/// # Safety
/// The object must be an owned Python reference.
pub unsafe fn register_incref(obj: NonNull<ffi::PyObject>) {
    if gil_is_acquired() {
        ffi::Py_INCREF(obj.as_ptr())
    } else {
        POOL.register_incref(obj);
    }
}

/// Registers a Python object pointer inside the release pool, to have its reference count decreased
/// the next time the GIL is acquired in pyo3.
///
/// If the GIL is held, the reference count will be decreased immediately instead of being queued
/// for later.
///
/// # Safety
/// The object must be an owned Python reference.
pub unsafe fn register_decref(obj: NonNull<ffi::PyObject>) {
    if gil_is_acquired() {
        ffi::Py_DECREF(obj.as_ptr())
    } else {
        POOL.register_decref(obj);
    }
}

/// Registers an owned object inside the GILPool, to be released when the GILPool drops.
///
/// # Safety
/// The object must be an owned Python reference.
pub unsafe fn register_owned(_py: Python, obj: NonNull<ffi::PyObject>) {
    debug_assert!(gil_is_acquired());
    // Ignores the error in case this function called from `atexit`.
    let _ = OWNED_OBJECTS.try_with(|holder| holder.borrow_mut().push(obj));
}

/// Increments pyo3's internal GIL count - to be called whenever GILPool or GILGuard is created.
#[inline(always)]
fn increment_gil_count() {
    // Ignores the error in case this function called from `atexit`.
    let _ = GIL_COUNT.try_with(|c| c.set(c.get() + 1));
}

/// Decrements pyo3's internal GIL count - to be called whenever GILPool or GILGuard is dropped.
#[inline(always)]
fn decrement_gil_count() {
    // Ignores the error in case this function called from `atexit`.
    let _ = GIL_COUNT.try_with(|c| {
        let current = c.get();
        debug_assert!(
            current > 0,
            "Negative GIL count detected. Please report this error to the PyO3 repo as a bug."
        );
        c.set(current - 1);
    });
}

/// Ensures the GIL is held, used in the implementation of `Python::with_gil`.
pub(crate) fn ensure_gil() -> EnsureGIL {
    if gil_is_acquired() {
        EnsureGIL(None)
    } else {
        EnsureGIL(Some(GILGuard::acquire()))
    }
}

/// Ensures the GIL is held, without interpreter state checking.
///
/// This bypasses interpreter state checking that would normally be performed
/// before acquiring the GIL.
pub(crate) fn ensure_gil_unchecked() -> EnsureGIL {
    if gil_is_acquired() {
        EnsureGIL(None)
    } else {
        EnsureGIL(Some(GILGuard::acquire_unchecked()))
    }
}

/// Struct used internally which avoids acquiring the GIL where it's not necessary.
#[allow(clippy::upper_case_acronyms)]
pub(crate) struct EnsureGIL(Option<GILGuard>);

impl EnsureGIL {
    /// Get the GIL token.
    ///
    /// # Safety
    /// If `self.0` is `None`, then this calls [Python::assume_gil_acquired].
    /// Thus this method could be used to get access to a GIL token while the GIL is not held.
    /// Care should be taken to only use the returned Python in contexts where it is certain the
    /// GIL continues to be held.
    pub unsafe fn python(&self) -> Python {
        match &self.0 {
            Some(gil) => gil.python(),
            None => Python::assume_gil_acquired(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{gil_is_acquired, GILPool, GIL_COUNT, OWNED_OBJECTS, POOL};
    use crate::{ffi, gil, AsPyPointer, IntoPyPointer, PyObject, Python, ToPyObject};
    use std::ptr::NonNull;

    fn get_object(py: Python) -> PyObject {
        // Convenience function for getting a single unique object, using `new_pool` so as to leave
        // the original pool state unchanged.
        let pool = unsafe { py.new_pool() };
        let py = pool.python();

        let obj = py.eval("object()", None, None).unwrap();
        obj.to_object(py)
    }

    fn owned_object_count() -> usize {
        OWNED_OBJECTS.with(|holder| holder.borrow().len())
    }

    #[test]
    fn test_owned() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = get_object(py);
        let obj_ptr = obj.as_ptr();
        // Ensure that obj does not get freed
        let _ref = obj.clone_ref(py);

        unsafe {
            {
                let pool = py.new_pool();
                gil::register_owned(pool.python(), NonNull::new_unchecked(obj.into_ptr()));

                assert_eq!(owned_object_count(), 1);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 2);
            }
            {
                let _pool = py.new_pool();
                assert_eq!(owned_object_count(), 0);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 1);
            }
        }
    }

    #[test]
    fn test_owned_nested() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = get_object(py);
        // Ensure that obj does not get freed
        let _ref = obj.clone_ref(py);
        let obj_ptr = obj.as_ptr();

        unsafe {
            {
                let _pool = py.new_pool();
                assert_eq!(owned_object_count(), 0);

                gil::register_owned(py, NonNull::new_unchecked(obj.into_ptr()));

                assert_eq!(owned_object_count(), 1);
                assert_eq!(ffi::Py_REFCNT(obj_ptr), 2);
                {
                    let _pool = py.new_pool();
                    let obj = get_object(py);
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
        let obj = get_object(py);
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
        let obj = get_object(py);
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
        // allow_threads should temporarily release GIL in PyO3's internal tracking too.
        let gil = Python::acquire_gil();
        let py = gil.python();

        assert!(gil_is_acquired());

        py.allow_threads(move || {
            assert!(!gil_is_acquired());

            let gil = Python::acquire_gil();
            assert!(gil_is_acquired());

            drop(gil);
            assert!(!gil_is_acquired());
        });

        assert!(gil_is_acquired());
    }

    #[allow(clippy::needless_late_init)]
    #[test]
    fn dropping_gil_does_not_invalidate_references() {
        // Acquiring GIL for the second time should be safe - see #864
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj;

        let gil2 = Python::acquire_gil();
        obj = py.eval("object()", None, None).unwrap();
        drop(gil2);

        // After gil2 drops, obj should still have a reference count of one
        assert_eq!(obj.get_refcnt(), 1);
    }

    #[test]
    fn test_clone_with_gil() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let obj = get_object(py);
        let count = obj.get_refcnt(py);

        // Cloning with the GIL should increase reference count immediately
        #[allow(clippy::redundant_clone)]
        let c = obj.clone();
        assert_eq!(count + 1, c.get_refcnt(py));
    }

    #[test]
    fn test_clone_without_gil() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = get_object(py);
        let count = obj.get_refcnt(py);

        // Cloning without GIL should not update reference count
        drop(gil);
        let c = obj.clone();
        assert_eq!(
            count,
            obj.get_refcnt(unsafe { Python::assume_gil_acquired() })
        );

        // Acquring GIL will clear this pending change
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Total reference count should be one higher
        assert_eq!(count + 1, obj.get_refcnt(py));

        // Clone dropped
        drop(c);

        // Overall count is now back to the original, and should be no pending change
        assert_eq!(count, obj.get_refcnt(py));
    }

    #[test]
    fn test_clone_in_other_thread() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = get_object(py);
        let count = obj.get_refcnt(py);

        // Move obj to a thread which does not have the GIL, and clone it
        let t = std::thread::spawn(move || {
            // Cloning without GIL should not update reference count
            #[allow(clippy::redundant_clone)]
            let _ = obj.clone();
            assert_eq!(
                count,
                obj.get_refcnt(unsafe { Python::assume_gil_acquired() })
            );

            // Return obj so original thread can continue to use
            obj
        });

        let obj = t.join().unwrap();
        let ptr = NonNull::new(obj.as_ptr()).unwrap();

        // The pointer should appear once in the incref pool, and once in the
        // decref pool (for the clone being created and also dropped)
        assert_eq!(&POOL.pointer_ops.lock().0, &vec![ptr]);
        assert_eq!(&POOL.pointer_ops.lock().1, &vec![ptr]);

        // Re-acquring GIL will clear these pending changes
        drop(gil);
        let gil = Python::acquire_gil();

        assert!(POOL.pointer_ops.lock().0.is_empty());
        assert!(POOL.pointer_ops.lock().1.is_empty());

        // Overall count is still unchanged
        assert_eq!(count, obj.get_refcnt(gil.python()));
    }

    #[test]
    fn test_update_counts_does_not_deadlock() {
        // update_counts can run arbitrary Python code during Py_DECREF.
        // if the locking is implemented incorrectly, it will deadlock.

        let gil = Python::acquire_gil();
        let obj = get_object(gil.python());

        unsafe {
            unsafe extern "C" fn capsule_drop(capsule: *mut ffi::PyObject) {
                // This line will implicitly call update_counts
                // -> and so cause deadlock if update_counts is not handling recursion correctly.
                let pool = GILPool::new();

                // Rebuild obj so that it can be dropped
                PyObject::from_owned_ptr(
                    pool.python(),
                    ffi::PyCapsule_GetPointer(capsule, std::ptr::null()) as _,
                );
            }

            let ptr = obj.into_ptr();
            let capsule = ffi::PyCapsule_New(ptr as _, std::ptr::null(), Some(capsule_drop));

            POOL.register_decref(NonNull::new(capsule).unwrap());

            // Updating the counts will call decref on the capsule, which calls capsule_drop
            POOL.update_counts(gil.python())
        }
    }
}
