//! Interaction with attachment of the current thread to the Python interpreter.

#[cfg(pyo3_disable_reference_pool)]
use crate::impl_::panic::PanicTrap;
use crate::{ffi, Python};

use std::cell::Cell;
#[cfg(not(pyo3_disable_reference_pool))]
use std::sync::OnceLock;
#[cfg_attr(pyo3_disable_reference_pool, allow(unused_imports))]
use std::{mem, ptr::NonNull, sync};

std::thread_local! {
    /// This is an internal counter in pyo3 monitoring whether this thread is attached to the interpreter.
    ///
    /// It will be incremented whenever an AttachGuard is created, and decremented whenever
    /// they are dropped.
    ///
    /// As a result, if this thread is attached to the interpreter, ATTACH_COUNT is greater than zero.
    ///
    /// Additionally, we sometimes need to prevent safe access to the Python interpreter,
    /// e.g. when implementing `__traverse__`, which is represented by a negative value.
    static ATTACH_COUNT: Cell<isize> = const { Cell::new(0) };
}

const ATTACH_FORBIDDEN_DURING_TRAVERSE: isize = -1;

/// Checks whether the thread is attached to the Python interpreter.
///
/// Note: This uses pyo3's internal count rather than PyGILState_Check for two reasons:
///  1) for performance
///  2) PyGILState_Check always returns 1 if the sub-interpreter APIs have ever been called,
///     which could lead to incorrect conclusions that the thread is attached.
#[inline(always)]
fn thread_is_attached() -> bool {
    ATTACH_COUNT.try_with(|c| c.get() > 0).unwrap_or(false)
}

/// RAII type that represents thread attachment to the interpreter.
pub(crate) enum AttachGuard {
    /// Indicates the thread was already attached when this AttachGuard was acquired.
    Assumed,
    /// Indicates that we attached when this AttachGuard was acquired
    Ensured { gstate: ffi::PyGILState_STATE },
}

/// Possible error when calling `try_attach()`
pub(crate) enum AttachError {
    /// Forbidden during GC traversal.
    ForbiddenDuringTraverse,
    /// The interpreter is not initialized.
    NotInitialized,
    #[cfg(Py_3_13)]
    /// The interpreter is finalizing.
    Finalizing,
}

impl AttachGuard {
    /// PyO3 internal API for attaching to the Python interpreter. The public API is Python::attach.
    ///
    /// If the thread was already attached via PyO3, this returns
    /// `AttachGuard::Assumed`. Otherwise, the thread will attach now and
    /// `AttachGuard::Ensured` will be returned.
    pub(crate) fn attach() -> Self {
        match Self::try_attach() {
            Ok(guard) => guard,
            Err(AttachError::ForbiddenDuringTraverse) => {
                panic!("{}", ForbidAttaching::FORBIDDEN_DURING_TRAVERSE)
            }
            Err(AttachError::NotInitialized) => {
                // try to initialize the interpreter and try again
                crate::interpreter_lifecycle::ensure_initialized();
                unsafe { Self::do_attach_unchecked() }
            }
            #[cfg(Py_3_13)]
            Err(AttachError::Finalizing) => {
                panic!("Cannot attach to the Python interpreter while it is finalizing.");
            }
        }
    }

    /// Variant of the above which will will return gracefully if the interpreter cannot be attached to.
    pub(crate) fn try_attach() -> Result<Self, AttachError> {
        match ATTACH_COUNT.try_with(|c| c.get()) {
            Ok(i) if i > 0 => {
                // SAFETY: We just checked that the thread is already attached.
                return Ok(unsafe { Self::assume() });
            }
            // Cannot attach during GC traversal.
            Ok(ATTACH_FORBIDDEN_DURING_TRAVERSE) => {
                return Err(AttachError::ForbiddenDuringTraverse)
            }
            // other cases handled below
            _ => {}
        }

        // SAFETY: always safe to call this
        if unsafe { ffi::Py_IsInitialized() } == 0 {
            return Err(AttachError::NotInitialized);
        }

        // Calling `PyGILState_Ensure` while finalizing may crash CPython in unpredictable
        // ways, we'll make a best effort attempt here to avoid that. (There's a time of
        // check to time-of-use issue, but it's better than nothing.)
        //
        // SAFETY: always safe to call this
        #[cfg(Py_3_13)]
        if unsafe { ffi::Py_IsFinalizing() } != 0 {
            // If the interpreter is not initialized, we cannot attach.
            return Err(AttachError::Finalizing);
        }

        // SAFETY: We have done everything reasonable to ensure we're in a safe state to
        // attach to the Python interpreter.
        Ok(unsafe { Self::do_attach_unchecked() })
    }

    /// Acquires the `AttachGuard` without performing any state checking.
    ///
    /// This can be called in "unsafe" contexts where the normal interpreter state
    /// checking performed by `AttachGuard::try_attach` may fail. This includes calling
    /// as part of multi-phase interpreter initialization.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the Python interpreter is sufficiently initialized
    /// for a thread to be able to attach to it.
    pub(crate) unsafe fn attach_unchecked() -> Self {
        if thread_is_attached() {
            return unsafe { Self::assume() };
        }

        unsafe { Self::do_attach_unchecked() }
    }

    /// Attach to the interpreter, without a fast-path to check if the thread is already attached.
    #[cold]
    unsafe fn do_attach_unchecked() -> Self {
        // SAFETY: interpreter is sufficiently initialized to attach a thread.
        let gstate = unsafe { ffi::PyGILState_Ensure() };
        increment_attach_count();
        // SAFETY: just attached to the interpreter
        drop_deferred_references(unsafe { Python::assume_attached() });
        AttachGuard::Ensured { gstate }
    }

    /// Acquires the `AttachGuard` while assuming that the thread is already attached
    /// to the interpreter.
    pub(crate) unsafe fn assume() -> Self {
        increment_attach_count();
        // SAFETY: invariant of calling this function
        drop_deferred_references(unsafe { Python::assume_attached() });
        AttachGuard::Assumed
    }

    /// Gets the Python token associated with this [`AttachGuard`].
    #[inline]
    pub(crate) fn python(&self) -> Python<'_> {
        // SAFETY: this guard guarantees the thread is attached
        unsafe { Python::assume_attached() }
    }
}

/// The Drop implementation for `AttachGuard` will decrement the attach count (and potentially detach).
impl Drop for AttachGuard {
    fn drop(&mut self) {
        match self {
            AttachGuard::Assumed => {}
            AttachGuard::Ensured { gstate } => unsafe {
                // Drop the objects in the pool before attempting to release the thread state
                ffi::PyGILState_Release(*gstate);
            },
        }
        decrement_attach_count();
    }
}

#[cfg(not(pyo3_disable_reference_pool))]
type PyObjVec = Vec<NonNull<ffi::PyObject>>;

#[cfg(not(pyo3_disable_reference_pool))]
/// Thread-safe storage for objects which were dec_ref while not attached.
struct ReferencePool {
    pending_decrefs: sync::Mutex<PyObjVec>,
}

#[cfg(not(pyo3_disable_reference_pool))]
impl ReferencePool {
    const fn new() -> Self {
        Self {
            pending_decrefs: sync::Mutex::new(Vec::new()),
        }
    }

    fn register_decref(&self, obj: NonNull<ffi::PyObject>) {
        self.pending_decrefs.lock().unwrap().push(obj);
    }

    fn drop_deferred_references(&self, _py: Python<'_>) {
        let mut pending_decrefs = self.pending_decrefs.lock().unwrap();
        if pending_decrefs.is_empty() {
            return;
        }

        let decrefs = mem::take(&mut *pending_decrefs);
        drop(pending_decrefs);

        for ptr in decrefs {
            unsafe { ffi::Py_DECREF(ptr.as_ptr()) };
        }
    }
}

#[cfg(not(pyo3_disable_reference_pool))]
unsafe impl Send for ReferencePool {}

#[cfg(not(pyo3_disable_reference_pool))]
unsafe impl Sync for ReferencePool {}

#[cfg(not(pyo3_disable_reference_pool))]
static POOL: OnceLock<ReferencePool> = OnceLock::new();

#[cfg(not(pyo3_disable_reference_pool))]
fn get_pool() -> &'static ReferencePool {
    POOL.get_or_init(ReferencePool::new)
}

#[cfg_attr(pyo3_disable_reference_pool, inline(always))]
#[cfg_attr(pyo3_disable_reference_pool, allow(unused_variables))]
fn drop_deferred_references(py: Python<'_>) {
    #[cfg(not(pyo3_disable_reference_pool))]
    if let Some(pool) = POOL.get() {
        pool.drop_deferred_references(py);
    }
}

/// A guard which can be used to temporarily detach from the interpreter and restore on `Drop`.
pub(crate) struct SuspendAttach {
    count: isize,
    tstate: *mut ffi::PyThreadState,
}

impl SuspendAttach {
    pub(crate) unsafe fn new() -> Self {
        let count = ATTACH_COUNT.with(|c| c.replace(0));
        let tstate = unsafe { ffi::PyEval_SaveThread() };

        Self { count, tstate }
    }
}

impl Drop for SuspendAttach {
    fn drop(&mut self) {
        ATTACH_COUNT.with(|c| c.set(self.count));
        unsafe {
            ffi::PyEval_RestoreThread(self.tstate);

            // Update counts of `Py<T>` that were dropped while not attached.
            #[cfg(not(pyo3_disable_reference_pool))]
            if let Some(pool) = POOL.get() {
                pool.drop_deferred_references(Python::assume_attached());
            }
        }
    }
}

/// Used to lock safe access to the interpreter
pub(crate) struct ForbidAttaching {
    count: isize,
}

impl ForbidAttaching {
    const FORBIDDEN_DURING_TRAVERSE: &'static str = "Attaching a thread to the interpreter is prohibited while a __traverse__ implementation is running.";

    /// Lock access to the interpreter while an implementation of `__traverse__` is running
    pub fn during_traverse() -> Self {
        Self::new(ATTACH_FORBIDDEN_DURING_TRAVERSE)
    }

    fn new(reason: isize) -> Self {
        let count = ATTACH_COUNT.with(|c| c.replace(reason));

        Self { count }
    }

    #[cold]
    fn bail(current: isize) {
        match current {
            ATTACH_FORBIDDEN_DURING_TRAVERSE => panic!("{}", Self::FORBIDDEN_DURING_TRAVERSE),
            _ => panic!("Attaching a thread to the interpreter is currently prohibited."),
        }
    }
}

impl Drop for ForbidAttaching {
    fn drop(&mut self) {
        ATTACH_COUNT.with(|c| c.set(self.count));
    }
}

/// Increments the reference count of a Python object if the thread is attached. If
/// the thread is not attached, this function will panic.
///
/// # Safety
/// The object must be an owned Python reference.
#[cfg(feature = "py-clone")]
#[track_caller]
pub unsafe fn register_incref(obj: NonNull<ffi::PyObject>) {
    if thread_is_attached() {
        unsafe { ffi::Py_INCREF(obj.as_ptr()) }
    } else {
        panic!("Cannot clone pointer into Python heap without the thread being attached.");
    }
}

/// Registers a Python object pointer inside the release pool, to have its reference count decreased
/// the next time the thread is attached in pyo3.
///
/// If the thread is attached, the reference count will be decreased immediately instead of being queued
/// for later.
///
/// # Safety
/// The object must be an owned Python reference.
#[track_caller]
pub unsafe fn register_decref(obj: NonNull<ffi::PyObject>) {
    if thread_is_attached() {
        unsafe { ffi::Py_DECREF(obj.as_ptr()) }
    } else {
        #[cfg(not(pyo3_disable_reference_pool))]
        get_pool().register_decref(obj);
        #[cfg(all(
            pyo3_disable_reference_pool,
            not(pyo3_leak_on_drop_without_reference_pool)
        ))]
        {
            let _trap = PanicTrap::new("Aborting the process to avoid panic-from-drop.");
            panic!("Cannot drop pointer into Python heap without the thread being attached.");
        }
    }
}

/// Private helper function to check if we are currently in a GC traversal (as detected by PyO3).
#[cfg(any(not(Py_LIMITED_API), Py_3_11))]
pub(crate) fn is_in_gc_traversal() -> bool {
    ATTACH_COUNT
        .try_with(|c| c.get() == ATTACH_FORBIDDEN_DURING_TRAVERSE)
        .unwrap_or(false)
}

/// Increments pyo3's internal attach count - to be called whenever an AttachGuard is created.
#[inline(always)]
fn increment_attach_count() {
    // Ignores the error in case this function called from `atexit`.
    let _ = ATTACH_COUNT.try_with(|c| {
        let current = c.get();
        if current < 0 {
            ForbidAttaching::bail(current);
        }
        c.set(current + 1);
    });
}

/// Decrements pyo3's internal attach count - to be called whenever AttachGuard is dropped.
#[inline(always)]
fn decrement_attach_count() {
    // Ignores the error in case this function called from `atexit`.
    let _ = ATTACH_COUNT.try_with(|c| {
        let current = c.get();
        debug_assert!(
            current > 0,
            "Negative attach count detected. Please report this error to the PyO3 repo as a bug."
        );
        c.set(current - 1);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{ffi, types::PyAnyMethods, Py, PyAny, Python};

    fn get_object(py: Python<'_>) -> Py<PyAny> {
        py.eval(ffi::c_str!("object()"), None, None)
            .unwrap()
            .unbind()
    }

    #[cfg(not(pyo3_disable_reference_pool))]
    fn pool_dec_refs_does_not_contain(obj: &Py<PyAny>) -> bool {
        !get_pool()
            .pending_decrefs
            .lock()
            .unwrap()
            .contains(&unsafe { NonNull::new_unchecked(obj.as_ptr()) })
    }

    // With free-threading, threads can empty the POOL at any time, so this
    // function does not test anything meaningful
    #[cfg(not(any(pyo3_disable_reference_pool, Py_GIL_DISABLED)))]
    fn pool_dec_refs_contains(obj: &Py<PyAny>) -> bool {
        get_pool()
            .pending_decrefs
            .lock()
            .unwrap()
            .contains(&unsafe { NonNull::new_unchecked(obj.as_ptr()) })
    }

    #[test]
    fn test_pyobject_drop_attached_decreases_refcnt() {
        Python::attach(|py| {
            let obj = get_object(py);

            // Create a reference to drop while attached.
            let reference = obj.clone_ref(py);

            assert_eq!(obj.get_refcnt(py), 2);
            #[cfg(not(pyo3_disable_reference_pool))]
            assert!(pool_dec_refs_does_not_contain(&obj));

            // While attached, reference count will be decreased immediately.
            drop(reference);

            assert_eq!(obj.get_refcnt(py), 1);
            #[cfg(not(any(pyo3_disable_reference_pool)))]
            assert!(pool_dec_refs_does_not_contain(&obj));
        });
    }

    #[test]
    #[cfg(all(not(pyo3_disable_reference_pool), not(target_arch = "wasm32")))] // We are building wasm Python with pthreads disabled
    fn test_pyobject_drop_detached_doesnt_decrease_refcnt() {
        let obj = Python::attach(|py| {
            let obj = get_object(py);
            // Create a reference to drop while detached.
            let reference = obj.clone_ref(py);

            assert_eq!(obj.get_refcnt(py), 2);
            assert!(pool_dec_refs_does_not_contain(&obj));

            // Drop reference in a separate (detached) thread.
            std::thread::spawn(move || drop(reference)).join().unwrap();

            // The reference count should not have changed, it is remembered
            // to release later.
            assert_eq!(obj.get_refcnt(py), 2);
            #[cfg(not(Py_GIL_DISABLED))]
            assert!(pool_dec_refs_contains(&obj));
            obj
        });

        // On next attach, the reference is released
        #[allow(unused)]
        Python::attach(|py| {
            // With free-threading, another thread could still be processing
            // DECREFs after releasing the lock on the POOL, so the
            // refcnt could still be 2 when this assert happens
            #[cfg(not(Py_GIL_DISABLED))]
            assert_eq!(obj.get_refcnt(py), 1);
            assert!(pool_dec_refs_does_not_contain(&obj));
        });
    }

    #[test]
    #[allow(deprecated)]
    fn test_attach_counts() {
        // Check `attach` and AttachGuard both increase counts correctly
        let get_attach_count = || ATTACH_COUNT.with(|c| c.get());

        assert_eq!(get_attach_count(), 0);
        Python::attach(|_| {
            assert_eq!(get_attach_count(), 1);

            let pool = unsafe { AttachGuard::assume() };
            assert_eq!(get_attach_count(), 2);

            let pool2 = unsafe { AttachGuard::assume() };
            assert_eq!(get_attach_count(), 3);

            drop(pool);
            assert_eq!(get_attach_count(), 2);

            Python::attach(|_| {
                // nested `attach` updates attach count
                assert_eq!(get_attach_count(), 3);
            });
            assert_eq!(get_attach_count(), 2);

            drop(pool2);
            assert_eq!(get_attach_count(), 1);
        });
        assert_eq!(get_attach_count(), 0);
    }

    #[test]
    fn test_detach() {
        assert!(!thread_is_attached());

        Python::attach(|py| {
            assert!(thread_is_attached());

            py.detach(move || {
                assert!(!thread_is_attached());

                Python::attach(|_| assert!(thread_is_attached()));

                assert!(!thread_is_attached());
            });

            assert!(thread_is_attached());
        });

        assert!(!thread_is_attached());
    }

    #[cfg(feature = "py-clone")]
    #[test]
    #[should_panic]
    fn test_detach_updates_refcounts() {
        Python::attach(|py| {
            // Make a simple object with 1 reference
            let obj = get_object(py);
            assert!(obj.get_refcnt(py) == 1);
            // Cloning the object when detached should panic
            py.detach(|| obj.clone());
        });
    }

    #[test]
    fn recursive_attach_ok() {
        Python::attach(|py| {
            let obj = Python::attach(|_| py.eval(ffi::c_str!("object()"), None, None).unwrap());
            assert_eq!(obj.get_refcnt(), 1);
        })
    }

    #[cfg(feature = "py-clone")]
    #[test]
    fn test_clone_attached() {
        Python::attach(|py| {
            let obj = get_object(py);
            let count = obj.get_refcnt(py);

            // Cloning when attached should increase reference count immediately
            #[allow(clippy::redundant_clone)]
            let c = obj.clone();
            assert_eq!(count + 1, c.get_refcnt(py));
        })
    }

    #[test]
    #[cfg(not(pyo3_disable_reference_pool))]
    fn test_drop_deferred_references_does_not_deadlock() {
        // drop_deferred_references can run arbitrary Python code during Py_DECREF.
        // if the locking is implemented incorrectly, it will deadlock.

        use crate::ffi;

        Python::attach(|py| {
            let obj = get_object(py);

            unsafe extern "C" fn capsule_drop(capsule: *mut ffi::PyObject) {
                // This line will implicitly call drop_deferred_references
                // -> and so cause deadlock if drop_deferred_references is not handling recursion correctly.
                let pool = unsafe { AttachGuard::assume() };

                // Rebuild obj so that it can be dropped
                unsafe {
                    Py::<PyAny>::from_owned_ptr(
                        pool.python(),
                        ffi::PyCapsule_GetPointer(capsule, std::ptr::null()) as _,
                    )
                };
            }

            let ptr = obj.into_ptr();

            let capsule =
                unsafe { ffi::PyCapsule_New(ptr as _, std::ptr::null(), Some(capsule_drop)) };

            get_pool().register_decref(NonNull::new(capsule).unwrap());

            // Updating the counts will call decref on the capsule, which calls capsule_drop
            get_pool().drop_deferred_references(py);
        })
    }

    #[test]
    #[cfg(not(pyo3_disable_reference_pool))]
    fn test_attach_guard_drop_deferred_references() {
        Python::attach(|py| {
            let obj = get_object(py);

            // For AttachGuard::attach

            get_pool().register_decref(NonNull::new(obj.clone_ref(py).into_ptr()).unwrap());
            #[cfg(not(Py_GIL_DISABLED))]
            assert!(pool_dec_refs_contains(&obj));
            let _guard = AttachGuard::attach();
            assert!(pool_dec_refs_does_not_contain(&obj));

            // For AttachGuard::assume

            get_pool().register_decref(NonNull::new(obj.clone_ref(py).into_ptr()).unwrap());
            #[cfg(not(Py_GIL_DISABLED))]
            assert!(pool_dec_refs_contains(&obj));
            let _guard2 = unsafe { AttachGuard::assume() };
            assert!(pool_dec_refs_does_not_contain(&obj));
        })
    }
}
