//! Interaction with attachment of the current thread to the Python interpreter.

#[cfg(pyo3_disable_reference_pool)]
use crate::impl_::panic::PanicTrap;
use crate::platform::prelude::*;
use crate::{ffi, Py, PyAny, Python};

#[cfg(not(pyo3_disable_reference_pool))]
use crate::platform::sync::non_poison::Mutex;
use core::cell::Cell;
#[cfg_attr(pyo3_disable_reference_pool, allow(unused_imports))]
use core::mem;
#[cfg(not(pyo3_disable_reference_pool))]
use core::sync::atomic::{AtomicBool, Ordering};

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
pub(crate) fn thread_is_attached() -> bool {
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
                // SAFETY: just initialized the interpreter
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

        // Py_IsInitialized() can return 1 while Py_InitializeEx is still
        // running (e.g. importing site.py). Block until any in-progress PyO3
        // initialization has fully completed.
        crate::interpreter_lifecycle::wait_for_initialization();

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
            // SAFETY: just confirmed that current thread is attached
            return unsafe { Self::assume() };
        }

        // SAFETY: requirements upheld by caller
        unsafe { Self::do_attach_unchecked() }
    }

    /// Attach to the interpreter, without a fast-path to check if the thread is already attached.
    /// # Safety
    /// The interpreter must be sufficiently initialized to attach a thread.
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
    ///
    /// # Safety
    /// Current thread must already be attached to the interpreter.
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
            AttachGuard::Ensured { gstate } => {
                // SAFETY: matching call to ensure in constructor
                unsafe {
                    // Drop the objects in the pool before attempting to release the thread state
                    ffi::PyGILState_Release(*gstate);
                }
            }
        }
        decrement_attach_count();
    }
}

#[cfg(not(pyo3_disable_reference_pool))]
type PyObjVec = Vec<Py<PyAny>>;

#[cfg(not(pyo3_disable_reference_pool))]
/// Thread-safe storage for objects which were dec_ref while not attached.
struct ReferencePool {
    // Whether any decrefs are (or may be) pending. The `Mutex` performs
    // synchronization so we can use `Relaxed` ordering for all operations
    // on this flag.
    dirty: AtomicBool,
    pending_decrefs: Mutex<PyObjVec>,
}

#[cfg(not(pyo3_disable_reference_pool))]
impl ReferencePool {
    const fn new() -> Self {
        Self {
            dirty: AtomicBool::new(false),
            pending_decrefs: Mutex::new(Vec::new()),
        }
    }

    fn register_decref(&self, obj: Py<PyAny>) {
        // It is important to set the dirty flag _after_ pushing the object
        // - at worst an existing drainer can take the object immediately and
        // the dirty flag will lead to a false positive drain.
        //
        // If the order is reversed, the draining can run before the object
        // is pushed and it might be leaked.
        self.pending_decrefs.lock().push(obj);
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn drop_deferred_references(&self, py: Python<'_>) {
        // Check the dirty flag first to avoid any possible contention from atomic
        // RMW operation to update the dirty flag on a hit.
        if !self.dirty.load(Ordering::Relaxed) {
            return;
        }

        // dirty flag is set, we _probably_ need to drop references (the flag is
        // not updated under the mutex so false positives are possible but rare)
        self.drop_deferred_references_slow(py);
    }

    #[cold]
    fn drop_deferred_references_slow(&self, py: Python<'_>) {
        // Compare and swap the dirty flag to false avoids multiple threads from having
        // contention on the mutex.
        if self
            .dirty
            .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            // Another thread is already dropping the references, so we can return early.
            return;
        }

        let mut pending_decrefs = self.pending_decrefs.lock();
        if pending_decrefs.is_empty() {
            // We don't set the dirty flag under the mutex so it's possible to reach
            // this case as a false positive. Returning early avoids a store on false
            // positives.
            return;
        }
        let decrefs = mem::take(&mut *pending_decrefs);
        drop(pending_decrefs);

        for obj in decrefs {
            obj.drop_ref(py);
        }
    }
}

#[cfg(not(pyo3_disable_reference_pool))]
static POOL: ReferencePool = ReferencePool::new();

#[cfg_attr(pyo3_disable_reference_pool, inline(always))]
#[cfg_attr(pyo3_disable_reference_pool, allow(unused_variables))]
fn drop_deferred_references(py: Python<'_>) {
    #[cfg(not(pyo3_disable_reference_pool))]
    {
        POOL.drop_deferred_references(py);
    }
}

/// A guard which can be used to temporarily detach from the interpreter and restore on `Drop`.
pub(crate) struct SuspendAttach {
    count: isize,
    tstate: *mut ffi::PyThreadState,
}

impl SuspendAttach {
    /// # Safety
    /// Current thread must be attached
    pub(crate) unsafe fn new() -> Self {
        let count = ATTACH_COUNT.with(|c| c.replace(0));
        // SAFETY: caller uphold requirements
        let tstate = unsafe { ffi::PyEval_SaveThread() };

        Self { count, tstate }
    }
}

impl Drop for SuspendAttach {
    fn drop(&mut self) {
        ATTACH_COUNT.with(|c| c.set(self.count));
        // SAFETY: tstate come from call to PyEval_SaveThread and it was not re-attached yet
        unsafe { ffi::PyEval_RestoreThread(self.tstate) };
        // Update counts of `Py<T>` that were dropped while not attached.
        #[cfg(not(pyo3_disable_reference_pool))]
        {
            // SAFETY: just re-attached
            let py = unsafe { Python::assume_attached() };
            POOL.drop_deferred_references(py);
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

/// Registers a Python object pointer inside the release pool, to have its reference count decreased
/// the next time the thread is attached in pyo3.
#[inline]
pub fn register_decref(obj: Py<PyAny>) {
    #[cfg(not(pyo3_disable_reference_pool))]
    {
        POOL.register_decref(obj);
    }
    #[cfg(all(
        pyo3_disable_reference_pool,
        not(pyo3_leak_on_drop_without_reference_pool)
    ))]
    {
        let _trap = PanicTrap::new("Aborting the process to avoid panic-from-drop.");
        panic!("Cannot drop pointer into Python heap without the thread being attached.");
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

#[allow(clippy::undocumented_unsafe_blocks, reason = "tests")]
#[cfg(test)]
mod tests {
    use super::*;

    use crate::ffi_ptr_ext::FfiPtrExt;
    use crate::{Py, PyAny, Python};

    fn get_object(py: Python<'_>) -> Py<PyAny> {
        py.eval(c"object()", None, None).unwrap().unbind()
    }

    #[cfg(not(pyo3_disable_reference_pool))]
    fn pool_dec_refs_does_not_contain(obj: &Py<PyAny>) -> bool {
        for _ in 0..100 {
            if !POOL
                .pending_decrefs
                .lock()
                .iter()
                .any(|pending| pending.is(obj))
            {
                return true;
            }

            // It is possible for another thread to be about to remove the decref
            // from the pool having already cleared the dirty flag, wait a bit and re-check.
            std::thread::sleep(core::time::Duration::from_millis(5));
        }
        false
    }

    // With free-threading, threads can empty the POOL at any time, so this
    // function does not test anything meaningful
    #[cfg(not(any(pyo3_disable_reference_pool, Py_GIL_DISABLED)))]
    fn pool_dec_refs_contains(obj: &Py<PyAny>) -> bool {
        POOL.pending_decrefs
            .lock()
            .unwrap()
            .iter()
            .any(|pending| pending.is(obj))
    }

    #[test]
    fn test_pyobject_drop_attached_decreases_refcnt() {
        Python::attach(|py| {
            let obj = get_object(py);

            // Create a reference to drop while attached.
            let reference = obj.clone_ref(py);

            assert_eq!(obj._get_refcnt(py), 2);
            #[cfg(not(pyo3_disable_reference_pool))]
            assert!(pool_dec_refs_does_not_contain(&obj));

            // While attached, reference count will be decreased immediately.
            drop(reference);

            assert_eq!(obj._get_refcnt(py), 1);
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

            assert_eq!(obj._get_refcnt(py), 2);
            assert!(pool_dec_refs_does_not_contain(&obj));

            // Drop reference in a separate (detached) thread.
            std::thread::spawn(move || drop(reference)).join().unwrap();

            // The reference count should not have changed, it is remembered
            // to release later.
            assert_eq!(obj._get_refcnt(py), 2);
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
            assert_eq!(obj._get_refcnt(py), 1);
            assert!(pool_dec_refs_does_not_contain(&obj));
        });
    }

    #[test]
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
            assert_eq!(obj._get_refcnt(py), 1);
            // Cloning the object when detached should panic
            py.detach(|| obj.clone());
        });
    }

    #[test]
    fn recursive_attach_ok() {
        Python::attach(|py| {
            let obj = Python::attach(|_| py.eval(c"object()", None, None).unwrap());
            assert_eq!(obj._get_refcnt(), 1);
        })
    }

    #[cfg(feature = "py-clone")]
    #[test]
    fn test_clone_attached() {
        Python::attach(|py| {
            let obj = get_object(py);
            let count = obj._get_refcnt(py);

            // Cloning when attached should increase reference count immediately
            #[expect(clippy::redundant_clone)]
            let c = obj.clone();
            assert_eq!(count + 1, c._get_refcnt(py));
        })
    }

    #[test]
    #[cfg(not(pyo3_disable_reference_pool))]
    fn test_detached_drop_is_collected_on_next_attach() {
        let obj = Python::attach(get_object);
        let obj2 = Python::attach(|py| obj.clone_ref(py));

        POOL.register_decref(obj2);

        Python::attach(|_| {
            assert!(pool_dec_refs_does_not_contain(&obj));
        });
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
                    use crate::Bound;

                    Bound::from_owned_ptr(
                        pool.python(),
                        ffi::PyCapsule_GetPointer(capsule, core::ptr::null()) as _,
                    )
                };
            }

            let ptr = obj.into_ptr();

            let capsule = unsafe {
                ffi::PyCapsule_New(ptr as _, core::ptr::null(), Some(capsule_drop)).assume_owned(py)
            }
            .unbind();

            POOL.register_decref(capsule);

            // Updating the counts will call decref on the capsule, which calls capsule_drop
            POOL.drop_deferred_references(py);
        })
    }

    #[test]
    #[cfg(not(pyo3_disable_reference_pool))]
    fn test_attach_guard_drop_deferred_references() {
        Python::attach(|py| {
            let obj = get_object(py);

            // For AttachGuard::attach

            POOL.register_decref(obj.clone_ref(py));
            #[cfg(not(Py_GIL_DISABLED))]
            assert!(pool_dec_refs_contains(&obj));
            let _guard = AttachGuard::attach();
            assert!(pool_dec_refs_does_not_contain(&obj));

            // For AttachGuard::assume

            POOL.register_decref(obj.clone_ref(py));
            #[cfg(not(Py_GIL_DISABLED))]
            assert!(pool_dec_refs_contains(&obj));
            let _guard2 = unsafe { AttachGuard::assume() };
            assert!(pool_dec_refs_does_not_contain(&obj));
        })
    }
}
