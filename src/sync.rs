//! Synchronization mechanisms which are aware of the existence of the Python interpreter.
//!
//! The Python interpreter has multiple "stop the world" situations which may block threads, such as
//! - The Python global interpreter lock (GIL), on GIL-enabled builds of Python, or
//! - The Python garbage collector (GC), which pauses attached threads during collection.
//!
//! To avoid deadlocks in these cases, threads should take care to be detached from the Python interpreter
//! before performing operations which might block waiting for other threads attached to the Python
//! interpreter.
//!
//! This module provides synchronization primitives which are able to synchronize under these conditions.
use crate::{
    internal::state::SuspendAttach,
    sealed::Sealed,
    types::{PyAny, PyString},
    Bound, Py, Python,
};
use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::{Once, OnceState},
};

pub mod critical_section;
pub(crate) mod once_lock;

/// Deprecated alias for [`pyo3::sync::critical_section::with_critical_section`][crate::sync::critical_section::with_critical_section]
#[deprecated(
    since = "0.28.0",
    note = "use pyo3::sync::critical_section::with_critical_section instead"
)]
pub fn with_critical_section<F, R>(object: &Bound<'_, PyAny>, f: F) -> R
where
    F: FnOnce() -> R,
{
    crate::sync::critical_section::with_critical_section(object, f)
}

/// Deprecated alias for [`pyo3::sync::critical_section::with_critical_section2`][crate::sync::critical_section::with_critical_section2]
#[deprecated(
    since = "0.28.0",
    note = "use pyo3::sync::critical_section::with_critical_section2 instead"
)]
pub fn with_critical_section2<F, R>(a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>, f: F) -> R
where
    F: FnOnce() -> R,
{
    crate::sync::critical_section::with_critical_section2(a, b, f)
}
pub use self::once_lock::PyOnceLock;

#[deprecated(
    since = "0.26.0",
    note = "Now internal only, to be removed after https://github.com/PyO3/pyo3/pull/5341"
)]
pub(crate) struct GILOnceCell<T> {
    once: Once,
    data: UnsafeCell<MaybeUninit<T>>,

    /// (Copied from std::sync::OnceLock)
    ///
    /// `PhantomData` to make sure dropck understands we're dropping T in our Drop impl.
    ///
    /// ```compile_error,E0597
    /// #![allow(deprecated)]
    /// use pyo3::Python;
    /// use pyo3::sync::GILOnceCell;
    ///
    /// struct A<'a>(#[allow(dead_code)] &'a str);
    ///
    /// impl<'a> Drop for A<'a> {
    ///     fn drop(&mut self) {}
    /// }
    ///
    /// let cell = GILOnceCell::new();
    /// {
    ///     let s = String::new();
    ///     let _ = Python::attach(|py| cell.set(py,A(&s)));
    /// }
    /// ```
    _marker: PhantomData<T>,
}

#[allow(deprecated)]
impl<T> Default for GILOnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

// T: Send is needed for Sync because the thread which drops the GILOnceCell can be different
// to the thread which fills it. (e.g. think scoped thread which fills the cell and then exits,
// leaving the cell to be dropped by the main thread).
#[allow(deprecated)]
unsafe impl<T: Send + Sync> Sync for GILOnceCell<T> {}
#[allow(deprecated)]
unsafe impl<T: Send> Send for GILOnceCell<T> {}

#[allow(deprecated)]
impl<T> GILOnceCell<T> {
    /// Create a `GILOnceCell` which does not yet contain a value.
    pub const fn new() -> Self {
        Self {
            once: Once::new(),
            data: UnsafeCell::new(MaybeUninit::uninit()),
            _marker: PhantomData,
        }
    }

    /// Get a reference to the contained value, or `None` if the cell has not yet been written.
    #[inline]
    pub fn get(&self, _py: Python<'_>) -> Option<&T> {
        if self.once.is_completed() {
            // SAFETY: the cell has been written.
            Some(unsafe { (*self.data.get()).assume_init_ref() })
        } else {
            None
        }
    }

    /// Like `get_or_init`, but accepts a fallible initialization function. If it fails, the cell
    /// is left uninitialized.
    ///
    /// See the type-level documentation for detail on re-entrancy and concurrent initialization.
    #[inline]
    pub fn get_or_try_init<F, E>(&self, py: Python<'_>, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if let Some(value) = self.get(py) {
            return Ok(value);
        }

        self.init(py, f)
    }

    #[cold]
    fn init<F, E>(&self, py: Python<'_>, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // Note that f() could temporarily release the GIL, so it's possible that another thread
        // writes to this GILOnceCell before f() finishes. That's fine; we'll just have to discard
        // the value computed here and accept a bit of wasted computation.

        // TODO: on the freethreaded build, consider wrapping this pair of operations in a
        // critical section (requires a critical section API which can use a PyMutex without
        // an object.)
        let value = f()?;
        let _ = self.set(py, value);

        Ok(self.get(py).unwrap())
    }

    /// Set the value in the cell.
    ///
    /// If the cell has already been written, `Err(value)` will be returned containing the new
    /// value which was not written.
    pub fn set(&self, _py: Python<'_>, value: T) -> Result<(), T> {
        let mut value = Some(value);
        // NB this can block, but since this is only writing a single value and
        // does not call arbitrary python code, we don't need to worry about
        // deadlocks with the GIL.
        self.once.call_once_force(|_| {
            // SAFETY: no other threads can be writing this value, because we are
            // inside the `call_once_force` closure.
            unsafe {
                // `.take().unwrap()` will never panic
                (*self.data.get()).write(value.take().unwrap());
            }
        });

        match value {
            // Some other thread wrote to the cell first
            Some(value) => Err(value),
            None => Ok(()),
        }
    }
}

#[allow(deprecated)]
impl<T> Drop for GILOnceCell<T> {
    fn drop(&mut self) {
        if self.once.is_completed() {
            // SAFETY: the cell has been written.
            unsafe { MaybeUninit::assume_init_drop(self.data.get_mut()) }
        }
    }
}

/// Interns `text` as a Python string and stores a reference to it in static storage.
///
/// A reference to the same Python string is returned on each invocation.
///
/// # Example: Using `intern!` to avoid needlessly recreating the same Python string
///
/// ```
/// use pyo3::intern;
/// # use pyo3::{prelude::*, types::PyDict};
///
/// #[pyfunction]
/// fn create_dict(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
///     let dict = PyDict::new(py);
///     //             👇 A new `PyString` is created
///     //                for every call of this function.
///     dict.set_item("foo", 42)?;
///     Ok(dict)
/// }
///
/// #[pyfunction]
/// fn create_dict_faster(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
///     let dict = PyDict::new(py);
///     //               👇 A `PyString` is created once and reused
///     //                  for the lifetime of the program.
///     dict.set_item(intern!(py, "foo"), 42)?;
///     Ok(dict)
/// }
/// #
/// # Python::attach(|py| {
/// #     let fun_slow = wrap_pyfunction!(create_dict, py).unwrap();
/// #     let dict = fun_slow.call0().unwrap();
/// #     assert!(dict.contains("foo").unwrap());
/// #     let fun = wrap_pyfunction!(create_dict_faster, py).unwrap();
/// #     let dict = fun.call0().unwrap();
/// #     assert!(dict.contains("foo").unwrap());
/// # });
/// ```
#[macro_export]
macro_rules! intern {
    ($py: expr, $text: expr) => {{
        static INTERNED: $crate::sync::Interned = $crate::sync::Interned::new($text);
        INTERNED.get($py)
    }};
}

/// Implementation detail for `intern!` macro.
#[doc(hidden)]
pub struct Interned(&'static str, PyOnceLock<Py<PyString>>);

impl Interned {
    /// Creates an empty holder for an interned `str`.
    pub const fn new(value: &'static str) -> Self {
        Interned(value, PyOnceLock::new())
    }

    /// Gets or creates the interned `str` value.
    #[inline]
    pub fn get<'py>(&self, py: Python<'py>) -> &Bound<'py, PyString> {
        self.1
            .get_or_init(py, || PyString::intern(py, self.0).into())
            .bind(py)
    }
}

/// Extension trait for [`Once`] to help avoid deadlocking when using a [`Once`] when attached to a
/// Python thread.
pub trait OnceExt: Sealed {
    ///The state of `Once`
    type OnceState;

    /// Similar to [`call_once`][Once::call_once], but releases the Python GIL temporarily
    /// if blocking on another thread currently calling this `Once`.
    fn call_once_py_attached(&self, py: Python<'_>, f: impl FnOnce());

    /// Similar to [`call_once_force`][Once::call_once_force], but releases the Python GIL
    /// temporarily if blocking on another thread currently calling this `Once`.
    fn call_once_force_py_attached(&self, py: Python<'_>, f: impl FnOnce(&Self::OnceState));
}

/// Extension trait for [`std::sync::OnceLock`] which helps avoid deadlocks between the Python
/// interpreter and initialization with the `OnceLock`.
pub trait OnceLockExt<T>: once_lock_ext_sealed::Sealed {
    /// Initializes this `OnceLock` with the given closure if it has not been initialized yet.
    ///
    /// If this function would block, this function detaches from the Python interpreter and
    /// reattaches before calling `f`. This avoids deadlocks between the Python interpreter and
    /// the `OnceLock` in cases where `f` can call arbitrary Python code, as calling arbitrary
    /// Python code can lead to `f` itself blocking on the Python interpreter.
    ///
    /// By detaching from the Python interpreter before blocking, this ensures that if `f` blocks
    /// then the Python interpreter cannot be blocked by `f` itself.
    fn get_or_init_py_attached<F>(&self, py: Python<'_>, f: F) -> &T
    where
        F: FnOnce() -> T;
}

/// Extension trait for [`std::sync::Mutex`] which helps avoid deadlocks between
/// the Python interpreter and acquiring the `Mutex`.
pub trait MutexExt<T>: Sealed {
    /// The result type returned by the `lock_py_attached` method.
    type LockResult<'a>
    where
        Self: 'a;

    /// Lock this `Mutex` in a manner that cannot deadlock with the Python interpreter.
    ///
    /// Before attempting to lock the mutex, this function detaches from the
    /// Python runtime. When the lock is acquired, it re-attaches to the Python
    /// runtime before returning the `LockResult`. This avoids deadlocks between
    /// the GIL and other global synchronization events triggered by the Python
    /// interpreter.
    fn lock_py_attached(&self, py: Python<'_>) -> Self::LockResult<'_>;
}

/// Extension trait for [`std::sync::RwLock`] which helps avoid deadlocks between
/// the Python interpreter and acquiring the `RwLock`.
pub trait RwLockExt<T>: rwlock_ext_sealed::Sealed {
    /// The result type returned by the `read_py_attached` method.
    type ReadLockResult<'a>
    where
        Self: 'a;

    /// The result type returned by the `write_py_attached` method.
    type WriteLockResult<'a>
    where
        Self: 'a;

    /// Lock this `RwLock` for reading in a manner that cannot deadlock with
    /// the Python interpreter.
    ///
    /// Before attempting to lock the rwlock, this function detaches from the
    /// Python runtime. When the lock is acquired, it re-attaches to the Python
    /// runtime before returning the `ReadLockResult`. This avoids deadlocks between
    /// the GIL and other global synchronization events triggered by the Python
    /// interpreter.
    fn read_py_attached(&self, py: Python<'_>) -> Self::ReadLockResult<'_>;

    /// Lock this `RwLock` for writing in a manner that cannot deadlock with
    /// the Python interpreter.
    ///
    /// Before attempting to lock the rwlock, this function detaches from the
    /// Python runtime. When the lock is acquired, it re-attaches to the Python
    /// runtime before returning the `WriteLockResult`. This avoids deadlocks between
    /// the GIL and other global synchronization events triggered by the Python
    /// interpreter.
    fn write_py_attached(&self, py: Python<'_>) -> Self::WriteLockResult<'_>;
}

impl OnceExt for Once {
    type OnceState = OnceState;

    fn call_once_py_attached(&self, py: Python<'_>, f: impl FnOnce()) {
        if self.is_completed() {
            return;
        }

        init_once_py_attached(self, py, f)
    }

    fn call_once_force_py_attached(&self, py: Python<'_>, f: impl FnOnce(&OnceState)) {
        if self.is_completed() {
            return;
        }

        init_once_force_py_attached(self, py, f);
    }
}

#[cfg(feature = "parking_lot")]
impl OnceExt for parking_lot::Once {
    type OnceState = parking_lot::OnceState;

    fn call_once_py_attached(&self, _py: Python<'_>, f: impl FnOnce()) {
        if self.state().done() {
            return;
        }

        let ts_guard = unsafe { SuspendAttach::new() };

        self.call_once(move || {
            drop(ts_guard);
            f();
        });
    }

    fn call_once_force_py_attached(
        &self,
        _py: Python<'_>,
        f: impl FnOnce(&parking_lot::OnceState),
    ) {
        if self.state().done() {
            return;
        }

        let ts_guard = unsafe { SuspendAttach::new() };

        self.call_once_force(move |state| {
            drop(ts_guard);
            f(&state);
        });
    }
}

impl<T> OnceLockExt<T> for std::sync::OnceLock<T> {
    fn get_or_init_py_attached<F>(&self, py: Python<'_>, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        // Use self.get() first to create a fast path when initialized
        self.get()
            .unwrap_or_else(|| init_once_lock_py_attached(self, py, f))
    }
}

impl<T> MutexExt<T> for std::sync::Mutex<T> {
    type LockResult<'a>
        = std::sync::LockResult<std::sync::MutexGuard<'a, T>>
    where
        Self: 'a;

    fn lock_py_attached(
        &self,
        _py: Python<'_>,
    ) -> std::sync::LockResult<std::sync::MutexGuard<'_, T>> {
        // If try_lock is successful or returns a poisoned mutex, return them so
        // the caller can deal with them. Otherwise we need to use blocking
        // lock, which requires detaching from the Python runtime to avoid
        // possible deadlocks.
        match self.try_lock() {
            Ok(inner) => return Ok(inner),
            Err(std::sync::TryLockError::Poisoned(inner)) => {
                return std::sync::LockResult::Err(inner)
            }
            Err(std::sync::TryLockError::WouldBlock) => {}
        }
        // SAFETY: detach from the runtime right before a possibly blocking call
        // then reattach when the blocking call completes and before calling
        // into the C API.
        let ts_guard = unsafe { SuspendAttach::new() };
        let res = self.lock();
        drop(ts_guard);
        res
    }
}

#[cfg(feature = "lock_api")]
impl<R: lock_api::RawMutex, T> MutexExt<T> for lock_api::Mutex<R, T> {
    type LockResult<'a>
        = lock_api::MutexGuard<'a, R, T>
    where
        Self: 'a;

    fn lock_py_attached(&self, _py: Python<'_>) -> lock_api::MutexGuard<'_, R, T> {
        if let Some(guard) = self.try_lock() {
            return guard;
        }

        let ts_guard = unsafe { SuspendAttach::new() };
        let res = self.lock();
        drop(ts_guard);
        res
    }
}

#[cfg(feature = "arc_lock")]
impl<R, T> MutexExt<T> for std::sync::Arc<lock_api::Mutex<R, T>>
where
    R: lock_api::RawMutex,
{
    type LockResult<'a>
        = lock_api::ArcMutexGuard<R, T>
    where
        Self: 'a;

    fn lock_py_attached(&self, _py: Python<'_>) -> lock_api::ArcMutexGuard<R, T> {
        if let Some(guard) = self.try_lock_arc() {
            return guard;
        }

        let ts_guard = unsafe { SuspendAttach::new() };
        let res = self.lock_arc();
        drop(ts_guard);
        res
    }
}

#[cfg(feature = "lock_api")]
impl<R, G, T> MutexExt<T> for lock_api::ReentrantMutex<R, G, T>
where
    R: lock_api::RawMutex,
    G: lock_api::GetThreadId,
{
    type LockResult<'a>
        = lock_api::ReentrantMutexGuard<'a, R, G, T>
    where
        Self: 'a;

    fn lock_py_attached(&self, _py: Python<'_>) -> lock_api::ReentrantMutexGuard<'_, R, G, T> {
        if let Some(guard) = self.try_lock() {
            return guard;
        }

        let ts_guard = unsafe { SuspendAttach::new() };
        let res = self.lock();
        drop(ts_guard);
        res
    }
}

#[cfg(feature = "arc_lock")]
impl<R, G, T> MutexExt<T> for std::sync::Arc<lock_api::ReentrantMutex<R, G, T>>
where
    R: lock_api::RawMutex,
    G: lock_api::GetThreadId,
{
    type LockResult<'a>
        = lock_api::ArcReentrantMutexGuard<R, G, T>
    where
        Self: 'a;

    fn lock_py_attached(&self, _py: Python<'_>) -> lock_api::ArcReentrantMutexGuard<R, G, T> {
        if let Some(guard) = self.try_lock_arc() {
            return guard;
        }

        let ts_guard = unsafe { SuspendAttach::new() };
        let res = self.lock_arc();
        drop(ts_guard);
        res
    }
}

impl<T> RwLockExt<T> for std::sync::RwLock<T> {
    type ReadLockResult<'a>
        = std::sync::LockResult<std::sync::RwLockReadGuard<'a, T>>
    where
        Self: 'a;

    type WriteLockResult<'a>
        = std::sync::LockResult<std::sync::RwLockWriteGuard<'a, T>>
    where
        Self: 'a;

    fn read_py_attached(&self, _py: Python<'_>) -> Self::ReadLockResult<'_> {
        // If try_read is successful or returns a poisoned rwlock, return them so
        // the caller can deal with them. Otherwise we need to use blocking
        // read lock, which requires detaching from the Python runtime to avoid
        // possible deadlocks.
        match self.try_read() {
            Ok(inner) => return Ok(inner),
            Err(std::sync::TryLockError::Poisoned(inner)) => {
                return std::sync::LockResult::Err(inner)
            }
            Err(std::sync::TryLockError::WouldBlock) => {}
        }

        // SAFETY: detach from the runtime right before a possibly blocking call
        // then reattach when the blocking call completes and before calling
        // into the C API.
        let ts_guard = unsafe { SuspendAttach::new() };

        let res = self.read();
        drop(ts_guard);
        res
    }

    fn write_py_attached(&self, _py: Python<'_>) -> Self::WriteLockResult<'_> {
        // If try_write is successful or returns a poisoned rwlock, return them so
        // the caller can deal with them. Otherwise we need to use blocking
        // write lock, which requires detaching from the Python runtime to avoid
        // possible deadlocks.
        match self.try_write() {
            Ok(inner) => return Ok(inner),
            Err(std::sync::TryLockError::Poisoned(inner)) => {
                return std::sync::LockResult::Err(inner)
            }
            Err(std::sync::TryLockError::WouldBlock) => {}
        }

        // SAFETY: detach from the runtime right before a possibly blocking call
        // then reattach when the blocking call completes and before calling
        // into the C API.
        let ts_guard = unsafe { SuspendAttach::new() };

        let res = self.write();
        drop(ts_guard);
        res
    }
}

#[cfg(feature = "lock_api")]
impl<R: lock_api::RawRwLock, T> RwLockExt<T> for lock_api::RwLock<R, T> {
    type ReadLockResult<'a>
        = lock_api::RwLockReadGuard<'a, R, T>
    where
        Self: 'a;

    type WriteLockResult<'a>
        = lock_api::RwLockWriteGuard<'a, R, T>
    where
        Self: 'a;

    fn read_py_attached(&self, _py: Python<'_>) -> Self::ReadLockResult<'_> {
        if let Some(guard) = self.try_read() {
            return guard;
        }

        let ts_guard = unsafe { SuspendAttach::new() };
        let res = self.read();
        drop(ts_guard);
        res
    }

    fn write_py_attached(&self, _py: Python<'_>) -> Self::WriteLockResult<'_> {
        if let Some(guard) = self.try_write() {
            return guard;
        }

        let ts_guard = unsafe { SuspendAttach::new() };
        let res = self.write();
        drop(ts_guard);
        res
    }
}

#[cfg(feature = "arc_lock")]
impl<R, T> RwLockExt<T> for std::sync::Arc<lock_api::RwLock<R, T>>
where
    R: lock_api::RawRwLock,
{
    type ReadLockResult<'a>
        = lock_api::ArcRwLockReadGuard<R, T>
    where
        Self: 'a;

    type WriteLockResult<'a>
        = lock_api::ArcRwLockWriteGuard<R, T>
    where
        Self: 'a;

    fn read_py_attached(&self, _py: Python<'_>) -> Self::ReadLockResult<'_> {
        if let Some(guard) = self.try_read_arc() {
            return guard;
        }

        let ts_guard = unsafe { SuspendAttach::new() };
        let res = self.read_arc();
        drop(ts_guard);
        res
    }

    fn write_py_attached(&self, _py: Python<'_>) -> Self::WriteLockResult<'_> {
        if let Some(guard) = self.try_write_arc() {
            return guard;
        }

        let ts_guard = unsafe { SuspendAttach::new() };
        let res = self.write_arc();
        drop(ts_guard);
        res
    }
}

#[cold]
fn init_once_py_attached<F, T>(once: &Once, _py: Python<'_>, f: F)
where
    F: FnOnce() -> T,
{
    // SAFETY: detach from the runtime right before a possibly blocking call
    // then reattach when the blocking call completes and before calling
    // into the C API.
    let ts_guard = unsafe { SuspendAttach::new() };

    once.call_once(move || {
        drop(ts_guard);
        f();
    });
}

#[cold]
fn init_once_force_py_attached<F, T>(once: &Once, _py: Python<'_>, f: F)
where
    F: FnOnce(&OnceState) -> T,
{
    // SAFETY: detach from the runtime right before a possibly blocking call
    // then reattach when the blocking call completes and before calling
    // into the C API.
    let ts_guard = unsafe { SuspendAttach::new() };

    once.call_once_force(move |state| {
        drop(ts_guard);
        f(state);
    });
}

#[cold]
fn init_once_lock_py_attached<'a, F, T>(
    lock: &'a std::sync::OnceLock<T>,
    _py: Python<'_>,
    f: F,
) -> &'a T
where
    F: FnOnce() -> T,
{
    // SAFETY: detach from the runtime right before a possibly blocking call
    // then reattach when the blocking call completes and before calling
    // into the C API.
    let ts_guard = unsafe { SuspendAttach::new() };

    // By having detached here, we guarantee that `.get_or_init` cannot deadlock with
    // the Python interpreter
    let value = lock.get_or_init(move || {
        drop(ts_guard);
        f()
    });

    value
}

mod once_lock_ext_sealed {
    pub trait Sealed {}
    impl<T> Sealed for std::sync::OnceLock<T> {}
}

mod rwlock_ext_sealed {
    pub trait Sealed {}
    impl<T> Sealed for std::sync::RwLock<T> {}
    #[cfg(feature = "lock_api")]
    impl<R, T> Sealed for lock_api::RwLock<R, T> {}
    #[cfg(feature = "arc_lock")]
    impl<R, T> Sealed for std::sync::Arc<lock_api::RwLock<R, T>> {}
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::{PyAnyMethods, PyDict, PyDictMethods};
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "macros")]
    use std::sync::atomic::{AtomicBool, Ordering};
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "macros")]
    use std::sync::Barrier;
    #[cfg(not(target_arch = "wasm32"))]
    use std::sync::Mutex;

    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "macros")]
    #[crate::pyclass(crate = "crate")]
    struct BoolWrapper(AtomicBool);

    #[test]
    fn test_intern() {
        Python::attach(|py| {
            let foo1 = "foo";
            let foo2 = intern!(py, "foo");
            let foo3 = intern!(py, stringify!(foo));

            let dict = PyDict::new(py);
            dict.set_item(foo1, 42_usize).unwrap();
            assert!(dict.contains(foo2).unwrap());
            assert_eq!(
                dict.get_item(foo3)
                    .unwrap()
                    .unwrap()
                    .extract::<usize>()
                    .unwrap(),
                42
            );
        });
    }

    #[test]
    #[allow(deprecated)]
    fn test_once_cell() {
        Python::attach(|py| {
            let cell = GILOnceCell::new();

            assert!(cell.get(py).is_none());

            assert_eq!(cell.get_or_try_init(py, || Err(5)), Err(5));
            assert!(cell.get(py).is_none());

            assert_eq!(cell.get_or_try_init(py, || Ok::<_, ()>(2)), Ok(&2));
            assert_eq!(cell.get(py), Some(&2));

            assert_eq!(cell.get_or_try_init(py, || Err(5)), Ok(&2));
        })
    }

    #[test]
    #[allow(deprecated)]
    fn test_once_cell_drop() {
        #[derive(Debug)]
        struct RecordDrop<'a>(&'a mut bool);

        impl Drop for RecordDrop<'_> {
            fn drop(&mut self) {
                *self.0 = true;
            }
        }

        Python::attach(|py| {
            let mut dropped = false;
            let cell = GILOnceCell::new();
            cell.set(py, RecordDrop(&mut dropped)).unwrap();
            let drop_container = cell.get(py).unwrap();

            assert!(!*drop_container.0);
            drop(cell);
            assert!(dropped);
        });
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    fn test_once_ext() {
        macro_rules! test_once {
            ($once:expr, $is_poisoned:expr) => {{
                // adapted from the example in the docs for Once::try_once_force
                let init = $once;
                std::thread::scope(|s| {
                    // poison the once
                    let handle = s.spawn(|| {
                        Python::attach(|py| {
                            init.call_once_py_attached(py, || panic!());
                        })
                    });
                    assert!(handle.join().is_err());

                    // poisoning propagates
                    let handle = s.spawn(|| {
                        Python::attach(|py| {
                            init.call_once_py_attached(py, || {});
                        });
                    });

                    assert!(handle.join().is_err());

                    // call_once_force will still run and reset the poisoned state
                    Python::attach(|py| {
                        init.call_once_force_py_attached(py, |state| {
                            assert!($is_poisoned(state.clone()));
                        });

                        // once any success happens, we stop propagating the poison
                        init.call_once_py_attached(py, || {});
                    });

                    // calling call_once_force should return immediately without calling the closure
                    Python::attach(|py| init.call_once_force_py_attached(py, |_| panic!()));
                });
            }};
        }

        test_once!(Once::new(), OnceState::is_poisoned);
        #[cfg(feature = "parking_lot")]
        test_once!(parking_lot::Once::new(), parking_lot::OnceState::poisoned);
    }

    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    #[test]
    fn test_once_lock_ext() {
        let cell = std::sync::OnceLock::new();
        std::thread::scope(|s| {
            assert!(cell.get().is_none());

            s.spawn(|| {
                Python::attach(|py| {
                    assert_eq!(*cell.get_or_init_py_attached(py, || 12345), 12345);
                });
            });
        });
        assert_eq!(cell.get(), Some(&12345));
    }

    #[cfg(feature = "macros")]
    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    #[test]
    fn test_mutex_ext() {
        let barrier = Barrier::new(2);

        let mutex = Python::attach(|py| -> Mutex<Py<BoolWrapper>> {
            Mutex::new(Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap())
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    let b = mutex.lock_py_attached(py).unwrap();
                    barrier.wait();
                    // sleep to ensure the other thread actually blocks
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    (*b).bind(py).borrow().0.store(true, Ordering::Release);
                    drop(b);
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    // blocks until the other thread releases the lock
                    let b = mutex.lock_py_attached(py).unwrap();
                    assert!((*b).bind(py).borrow().0.load(Ordering::Acquire));
                });
            });
        });
    }

    #[cfg(feature = "macros")]
    #[cfg(all(
        any(feature = "parking_lot", feature = "lock_api"),
        not(target_arch = "wasm32") // We are building wasm Python with pthreads disabled
    ))]
    #[test]
    fn test_parking_lot_mutex_ext() {
        macro_rules! test_mutex {
            ($guard:ty ,$mutex:stmt) => {{
                let barrier = Barrier::new(2);

                let mutex = Python::attach({ $mutex });

                std::thread::scope(|s| {
                    s.spawn(|| {
                        Python::attach(|py| {
                            let b: $guard = mutex.lock_py_attached(py);
                            barrier.wait();
                            // sleep to ensure the other thread actually blocks
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            (*b).bind(py).borrow().0.store(true, Ordering::Release);
                            drop(b);
                        });
                    });
                    s.spawn(|| {
                        barrier.wait();
                        Python::attach(|py| {
                            // blocks until the other thread releases the lock
                            let b: $guard = mutex.lock_py_attached(py);
                            assert!((*b).bind(py).borrow().0.load(Ordering::Acquire));
                        });
                    });
                });
            }};
        }

        test_mutex!(parking_lot::MutexGuard<'_, _>, |py| {
            parking_lot::Mutex::new(Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap())
        });

        test_mutex!(parking_lot::ReentrantMutexGuard<'_, _>, |py| {
            parking_lot::ReentrantMutex::new(
                Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap(),
            )
        });

        #[cfg(feature = "arc_lock")]
        test_mutex!(parking_lot::ArcMutexGuard<_, _>, |py| {
            let mutex =
                parking_lot::Mutex::new(Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap());
            std::sync::Arc::new(mutex)
        });

        #[cfg(feature = "arc_lock")]
        test_mutex!(parking_lot::ArcReentrantMutexGuard<_, _, _>, |py| {
            let mutex =
                parking_lot::ReentrantMutex::new(Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap());
            std::sync::Arc::new(mutex)
        });
    }

    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    #[test]
    fn test_mutex_ext_poison() {
        let mutex = Mutex::new(42);

        std::thread::scope(|s| {
            let lock_result = s.spawn(|| {
                Python::attach(|py| {
                    let _unused = mutex.lock_py_attached(py);
                    panic!();
                });
            });
            assert!(lock_result.join().is_err());
            assert!(mutex.is_poisoned());
        });
        let guard = Python::attach(|py| {
            // recover from the poisoning
            match mutex.lock_py_attached(py) {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            }
        });
        assert_eq!(*guard, 42);
    }

    #[cfg(feature = "macros")]
    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    #[test]
    fn test_rwlock_ext_writer_blocks_reader() {
        use std::sync::RwLock;

        let barrier = Barrier::new(2);

        let rwlock = Python::attach(|py| -> RwLock<Py<BoolWrapper>> {
            RwLock::new(Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap())
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    let b = rwlock.write_py_attached(py).unwrap();
                    barrier.wait();
                    // sleep to ensure the other thread actually blocks
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    (*b).bind(py).borrow().0.store(true, Ordering::Release);
                    drop(b);
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    // blocks until the other thread releases the lock
                    let b = rwlock.read_py_attached(py).unwrap();
                    assert!((*b).bind(py).borrow().0.load(Ordering::Acquire));
                });
            });
        });
    }

    #[cfg(feature = "macros")]
    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    #[test]
    fn test_rwlock_ext_reader_blocks_writer() {
        use std::sync::RwLock;

        let barrier = Barrier::new(2);

        let rwlock = Python::attach(|py| -> RwLock<Py<BoolWrapper>> {
            RwLock::new(Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap())
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    let b = rwlock.read_py_attached(py).unwrap();
                    barrier.wait();

                    // sleep to ensure the other thread actually blocks
                    std::thread::sleep(std::time::Duration::from_millis(10));

                    // The bool must still be false (i.e., the writer did not actually write the
                    // value yet).
                    assert!(!(*b).bind(py).borrow().0.load(Ordering::Acquire));
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    // blocks until the other thread releases the lock
                    let b = rwlock.write_py_attached(py).unwrap();
                    (*b).bind(py).borrow().0.store(true, Ordering::Release);
                    drop(b);
                });
            });
        });

        // Confirm that the writer did in fact run and write the expected `true` value.
        Python::attach(|py| {
            let b = rwlock.read_py_attached(py).unwrap();
            assert!((*b).bind(py).borrow().0.load(Ordering::Acquire));
            drop(b);
        });
    }

    #[cfg(feature = "macros")]
    #[cfg(all(
        any(feature = "parking_lot", feature = "lock_api"),
        not(target_arch = "wasm32") // We are building wasm Python with pthreads disabled
    ))]
    #[test]
    fn test_parking_lot_rwlock_ext_writer_blocks_reader() {
        macro_rules! test_rwlock {
            ($write_guard:ty, $read_guard:ty, $rwlock:stmt) => {{
                let barrier = Barrier::new(2);

                let rwlock = Python::attach({ $rwlock });

                std::thread::scope(|s| {
                    s.spawn(|| {
                        Python::attach(|py| {
                            let b: $write_guard = rwlock.write_py_attached(py);
                            barrier.wait();
                            // sleep to ensure the other thread actually blocks
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            (*b).bind(py).borrow().0.store(true, Ordering::Release);
                            drop(b);
                        });
                    });
                    s.spawn(|| {
                        barrier.wait();
                        Python::attach(|py| {
                            // blocks until the other thread releases the lock
                            let b: $read_guard = rwlock.read_py_attached(py);
                            assert!((*b).bind(py).borrow().0.load(Ordering::Acquire));
                        });
                    });
                });
            }};
        }

        test_rwlock!(
            parking_lot::RwLockWriteGuard<'_, _>,
            parking_lot::RwLockReadGuard<'_, _>,
            |py| {
                parking_lot::RwLock::new(Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap())
            }
        );

        #[cfg(feature = "arc_lock")]
        test_rwlock!(
            parking_lot::ArcRwLockWriteGuard<_, _>,
            parking_lot::ArcRwLockReadGuard<_, _>,
            |py| {
                let rwlock = parking_lot::RwLock::new(
                    Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap(),
                );
                std::sync::Arc::new(rwlock)
            }
        );
    }

    #[cfg(feature = "macros")]
    #[cfg(all(
        any(feature = "parking_lot", feature = "lock_api"),
        not(target_arch = "wasm32") // We are building wasm Python with pthreads disabled
    ))]
    #[test]
    fn test_parking_lot_rwlock_ext_reader_blocks_writer() {
        macro_rules! test_rwlock {
            ($write_guard:ty, $read_guard:ty, $rwlock:stmt) => {{
                let barrier = Barrier::new(2);

                let rwlock = Python::attach({ $rwlock });

                std::thread::scope(|s| {
                    s.spawn(|| {
                        Python::attach(|py| {
                            let b: $read_guard = rwlock.read_py_attached(py);
                            barrier.wait();

                            // sleep to ensure the other thread actually blocks
                            std::thread::sleep(std::time::Duration::from_millis(10));

                            // The bool must still be false (i.e., the writer did not actually write the
                            // value yet).
                            assert!(!(*b).bind(py).borrow().0.load(Ordering::Acquire));                            (*b).bind(py).borrow().0.store(true, Ordering::Release);

                            drop(b);
                        });
                    });
                    s.spawn(|| {
                        barrier.wait();
                        Python::attach(|py| {
                            // blocks until the other thread releases the lock
                            let b: $write_guard = rwlock.write_py_attached(py);
                            (*b).bind(py).borrow().0.store(true, Ordering::Release);
                        });
                    });
                });

                // Confirm that the writer did in fact run and write the expected `true` value.
                Python::attach(|py| {
                    let b: $read_guard = rwlock.read_py_attached(py);
                    assert!((*b).bind(py).borrow().0.load(Ordering::Acquire));
                    drop(b);
                });
            }};
        }

        test_rwlock!(
            parking_lot::RwLockWriteGuard<'_, _>,
            parking_lot::RwLockReadGuard<'_, _>,
            |py| {
                parking_lot::RwLock::new(Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap())
            }
        );

        #[cfg(feature = "arc_lock")]
        test_rwlock!(
            parking_lot::ArcRwLockWriteGuard<_, _>,
            parking_lot::ArcRwLockReadGuard<_, _>,
            |py| {
                let rwlock = parking_lot::RwLock::new(
                    Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap(),
                );
                std::sync::Arc::new(rwlock)
            }
        );
    }

    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    #[test]
    fn test_rwlock_ext_poison() {
        use std::sync::RwLock;

        let rwlock = RwLock::new(42);

        std::thread::scope(|s| {
            let lock_result = s.spawn(|| {
                Python::attach(|py| {
                    let _unused = rwlock.write_py_attached(py);
                    panic!();
                });
            });
            assert!(lock_result.join().is_err());
            assert!(rwlock.is_poisoned());
            Python::attach(|py| {
                assert!(rwlock.read_py_attached(py).is_err());
                assert!(rwlock.write_py_attached(py).is_err());
            });
        });
        Python::attach(|py| {
            // recover from the poisoning
            let guard = rwlock.write_py_attached(py).unwrap_err().into_inner();
            assert_eq!(*guard, 42);
        });
    }
}
