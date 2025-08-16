//! Synchronization mechanisms based on the Python GIL.
//!
//! With the acceptance of [PEP 703] (aka a "freethreaded Python") for Python 3.13, these
//! are likely to undergo significant developments in the future.
//!
//! [PEP 703]: https://peps.python.org/pep-703/
use crate::{
    internal::state::SuspendAttach,
    sealed::Sealed,
    types::{any::PyAnyMethods, PyAny, PyString},
    Bound, Py, PyResult, PyTypeCheck, Python,
};
use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::{Once, OnceState},
};

pub(crate) mod once_lock;

#[cfg(not(Py_GIL_DISABLED))]
use crate::PyVisit;

pub use self::once_lock::PyOnceLock;

/// Value with concurrent access protected by the GIL.
///
/// This is a synchronization primitive based on Python's global interpreter lock (GIL).
/// It ensures that only one thread at a time can access the inner value via shared references.
/// It can be combined with interior mutability to obtain mutable references.
///
/// This type is not defined for extensions built against the free-threaded CPython ABI.
///
/// # Example
///
/// Combining `GILProtected` with `RefCell` enables mutable access to static data:
///
/// ```
/// # #![allow(deprecated)]
/// # use pyo3::prelude::*;
/// use pyo3::sync::GILProtected;
/// use std::cell::RefCell;
///
/// static NUMBERS: GILProtected<RefCell<Vec<i32>>> = GILProtected::new(RefCell::new(Vec::new()));
///
/// Python::attach(|py| {
///     NUMBERS.get(py).borrow_mut().push(42);
/// });
/// ```
#[deprecated(
    since = "0.26.0",
    note = "Prefer an interior mutability primitive compatible with free-threaded Python, such as `Mutex` in combination with the `MutexExt` trait"
)]
#[cfg(not(Py_GIL_DISABLED))]
pub struct GILProtected<T> {
    value: T,
}

#[allow(deprecated)]
#[cfg(not(Py_GIL_DISABLED))]
impl<T> GILProtected<T> {
    /// Place the given value under the protection of the GIL.
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    /// Gain access to the inner value by giving proof of having acquired the GIL.
    pub fn get<'py>(&'py self, _py: Python<'py>) -> &'py T {
        &self.value
    }

    /// Gain access to the inner value by giving proof that garbage collection is happening.
    pub fn traverse<'py>(&'py self, _visit: PyVisit<'py>) -> &'py T {
        &self.value
    }
}

#[allow(deprecated)]
#[cfg(not(Py_GIL_DISABLED))]
unsafe impl<T> Sync for GILProtected<T> where T: Send {}

/// A write-once primitive similar to [`std::sync::OnceLock<T>`].
///
/// Unlike `OnceLock<T>` which blocks threads to achieve thread safety, `GilOnceCell<T>`
/// allows calls to [`get_or_init`][GILOnceCell::get_or_init] and
/// [`get_or_try_init`][GILOnceCell::get_or_try_init] to race to create an initialized value.
/// (It is still guaranteed that only one thread will ever write to the cell.)
///
/// On Python versions that run with the Global Interpreter Lock (GIL), this helps to avoid
/// deadlocks between initialization and the GIL. For an example of such a deadlock, see
#[doc = concat!(
    "[the FAQ section](https://pyo3.rs/v",
    env!("CARGO_PKG_VERSION"),
    "/faq.html#im-experiencing-deadlocks-using-pyo3-with-stdsynconcelock-stdsynclazylock-lazy_static-and-once_cell)"
)]
/// of the guide.
///
/// Note that because the GIL blocks concurrent execution, in practice the means that
/// [`get_or_init`][GILOnceCell::get_or_init] and
/// [`get_or_try_init`][GILOnceCell::get_or_try_init] may race if the initialization
/// function leads to the GIL being released and a thread context switch. This can
/// happen when importing or calling any Python code, as long as it releases the
/// GIL at some point. On free-threaded Python without any GIL, the race is
/// more likely since there is no GIL to prevent races. In the future, PyO3 may change
/// the semantics of GILOnceCell to behave more like the GIL build in the future.
///
/// # Re-entrant initialization
///
/// [`get_or_init`][GILOnceCell::get_or_init] and
/// [`get_or_try_init`][GILOnceCell::get_or_try_init] do not protect against infinite recursion
/// from reentrant initialization.
///
/// # Examples
///
/// The following example shows how to use `GILOnceCell` to share a reference to a Python list
/// between threads:
///
/// ```
/// #![allow(deprecated)]
/// use pyo3::sync::GILOnceCell;
/// use pyo3::prelude::*;
/// use pyo3::types::PyList;
///
/// static LIST_CELL: GILOnceCell<Py<PyList>> = GILOnceCell::new();
///
/// pub fn get_shared_list(py: Python<'_>) -> &Bound<'_, PyList> {
///     LIST_CELL
///         .get_or_init(py, || PyList::empty(py).unbind())
///         .bind(py)
/// }
/// # Python::attach(|py| assert_eq!(get_shared_list(py).len(), 0));
/// ```
#[deprecated(
    since = "0.26.0",
    note = "Prefer `pyo3::sync::PyOnceLock`, which avoids the possibility of racing during initialization."
)]
pub struct GILOnceCell<T> {
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

    /// Get a reference to the contained value, initializing it if needed using the provided
    /// closure.
    ///
    /// See the type-level documentation for detail on re-entrancy and concurrent initialization.
    #[inline]
    pub fn get_or_init<F>(&self, py: Python<'_>, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if let Some(value) = self.get(py) {
            return value;
        }

        // .unwrap() will never panic because the result is always Ok
        self.init(py, || Ok::<T, std::convert::Infallible>(f()))
            .unwrap()
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

    /// Get the contents of the cell mutably. This is only possible if the reference to the cell is
    /// unique.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.once.is_completed() {
            // SAFETY: the cell has been written.
            Some(unsafe { (*self.data.get()).assume_init_mut() })
        } else {
            None
        }
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

    /// Takes the value out of the cell, moving it back to an uninitialized state.
    ///
    /// Has no effect and returns None if the cell has not yet been written.
    pub fn take(&mut self) -> Option<T> {
        if self.once.is_completed() {
            // Reset the cell to its default state so that it won't try to
            // drop the value again.
            self.once = Once::new();
            // SAFETY: the cell has been written. `self.once` has been reset,
            // so when `self` is dropped the value won't be read again.
            Some(unsafe { self.data.get_mut().assume_init_read() })
        } else {
            None
        }
    }

    /// Consumes the cell, returning the wrapped value.
    ///
    /// Returns None if the cell has not yet been written.
    pub fn into_inner(mut self) -> Option<T> {
        self.take()
    }
}

#[allow(deprecated)]
impl<T> GILOnceCell<Py<T>> {
    /// Creates a new cell that contains a new Python reference to the same contained object.
    ///
    /// Returns an uninitialized cell if `self` has not yet been initialized.
    pub fn clone_ref(&self, py: Python<'_>) -> Self {
        let cloned = Self {
            once: Once::new(),
            data: UnsafeCell::new(MaybeUninit::uninit()),
            _marker: PhantomData,
        };
        if let Some(value) = self.get(py) {
            let _ = cloned.set(py, value.clone_ref(py));
        }
        cloned
    }
}

#[allow(deprecated)]
impl<T> GILOnceCell<Py<T>>
where
    T: PyTypeCheck,
{
    /// Get a reference to the contained Python type, initializing the cell if needed.
    ///
    /// This is a shorthand method for `get_or_init` which imports the type from Python on init.
    ///
    /// # Example: Using `GILOnceCell` to store a class in a static variable.
    ///
    /// `GILOnceCell` can be used to avoid importing a class multiple times:
    /// ```
    /// #![allow(deprecated)]
    /// # use pyo3::prelude::*;
    /// # use pyo3::sync::GILOnceCell;
    /// # use pyo3::types::{PyDict, PyType};
    /// # use pyo3::intern;
    /// #
    /// #[pyfunction]
    /// fn create_ordered_dict<'py>(py: Python<'py>, dict: Bound<'py, PyDict>) -> PyResult<Bound<'py, PyAny>> {
    ///     // Even if this function is called multiple times,
    ///     // the `OrderedDict` class will be imported only once.
    ///     static ORDERED_DICT: GILOnceCell<Py<PyType>> = GILOnceCell::new();
    ///     ORDERED_DICT
    ///         .import(py, "collections", "OrderedDict")?
    ///         .call1((dict,))
    /// }
    ///
    /// # Python::attach(|py| {
    /// #     let dict = PyDict::new(py);
    /// #     dict.set_item(intern!(py, "foo"), 42).unwrap();
    /// #     let fun = wrap_pyfunction!(create_ordered_dict, py).unwrap();
    /// #     let ordered_dict = fun.call1((&dict,)).unwrap();
    /// #     assert!(dict.eq(ordered_dict).unwrap());
    /// # });
    /// ```
    pub fn import<'py>(
        &self,
        py: Python<'py>,
        module_name: &str,
        attr_name: &str,
    ) -> PyResult<&Bound<'py, T>> {
        self.get_or_try_init(py, || {
            let type_object = py.import(module_name)?.getattr(attr_name)?.cast_into()?;
            Ok(type_object.unbind())
        })
        .map(|ty| ty.bind(py))
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

/// Executes a closure with a Python critical section held on an object.
///
/// Acquires the per-object lock for the object `op` that is held
/// until the closure `f` is finished.
///
/// This is structurally equivalent to the use of the paired
/// Py_BEGIN_CRITICAL_SECTION and Py_END_CRITICAL_SECTION C-API macros.
///
/// A no-op on GIL-enabled builds, where the critical section API is exposed as
/// a no-op by the Python C API.
///
/// Provides weaker locking guarantees than traditional locks, but can in some
/// cases be used to provide guarantees similar to the GIL without the risk of
/// deadlocks associated with traditional locks.
///
/// Many CPython C API functions do not acquire the per-object lock on objects
/// passed to Python. You should not expect critical sections applied to
/// built-in types to prevent concurrent modification. This API is most useful
/// for user-defined types with full control over how the internal state for the
/// type is managed.
#[cfg_attr(not(Py_GIL_DISABLED), allow(unused_variables))]
pub fn with_critical_section<F, R>(object: &Bound<'_, PyAny>, f: F) -> R
where
    F: FnOnce() -> R,
{
    #[cfg(Py_GIL_DISABLED)]
    {
        struct Guard(crate::ffi::PyCriticalSection);

        impl Drop for Guard {
            fn drop(&mut self) {
                unsafe {
                    crate::ffi::PyCriticalSection_End(&mut self.0);
                }
            }
        }

        let mut guard = Guard(unsafe { std::mem::zeroed() });
        unsafe { crate::ffi::PyCriticalSection_Begin(&mut guard.0, object.as_ptr()) };
        f()
    }
    #[cfg(not(Py_GIL_DISABLED))]
    {
        f()
    }
}

/// Executes a closure with a Python critical section held on two objects.
///
/// Acquires the per-object lock for the objects `a` and `b` that are held
/// until the closure `f` is finished.
///
/// This is structurally equivalent to the use of the paired
/// Py_BEGIN_CRITICAL_SECTION2 and Py_END_CRITICAL_SECTION2 C-API macros.
///
/// A no-op on GIL-enabled builds, where the critical section API is exposed as
/// a no-op by the Python C API.
///
/// Provides weaker locking guarantees than traditional locks, but can in some
/// cases be used to provide guarantees similar to the GIL without the risk of
/// deadlocks associated with traditional locks.
///
/// Many CPython C API functions do not acquire the per-object lock on objects
/// passed to Python. You should not expect critical sections applied to
/// built-in types to prevent concurrent modification. This API is most useful
/// for user-defined types with full control over how the internal state for the
/// type is managed.
#[cfg_attr(not(Py_GIL_DISABLED), allow(unused_variables))]
pub fn with_critical_section2<F, R>(a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>, f: F) -> R
where
    F: FnOnce() -> R,
{
    #[cfg(Py_GIL_DISABLED)]
    {
        struct Guard(crate::ffi::PyCriticalSection2);

        impl Drop for Guard {
            fn drop(&mut self) {
                unsafe {
                    crate::ffi::PyCriticalSection2_End(&mut self.0);
                }
            }
        }

        let mut guard = Guard(unsafe { std::mem::zeroed() });
        unsafe { crate::ffi::PyCriticalSection2_Begin(&mut guard.0, a.as_ptr(), b.as_ptr()) };
        f()
    }
    #[cfg(not(Py_GIL_DISABLED))]
    {
        f()
    }
}

mod once_lock_ext_sealed {
    pub trait Sealed {}
    impl<T> Sealed for std::sync::OnceLock<T> {}
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::{PyDict, PyDictMethods};
    #[cfg(not(target_arch = "wasm32"))]
    use std::sync::Mutex;
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "macros")]
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Barrier,
    };

    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "macros")]
    #[crate::pyclass(crate = "crate")]
    struct BoolWrapper(AtomicBool);

    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "macros")]
    #[crate::pyclass(crate = "crate")]
    struct VecWrapper(Vec<isize>);

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
            let mut cell = GILOnceCell::new();

            assert!(cell.get(py).is_none());

            assert_eq!(cell.get_or_try_init(py, || Err(5)), Err(5));
            assert!(cell.get(py).is_none());

            assert_eq!(cell.get_or_try_init(py, || Ok::<_, ()>(2)), Ok(&2));
            assert_eq!(cell.get(py), Some(&2));

            assert_eq!(cell.get_or_try_init(py, || Err(5)), Ok(&2));

            assert_eq!(cell.take(), Some(2));
            assert_eq!(cell.into_inner(), None);

            let cell_py = GILOnceCell::new();
            assert!(cell_py.clone_ref(py).get(py).is_none());
            cell_py.get_or_init(py, || py.None());
            assert!(cell_py.clone_ref(py).get(py).unwrap().is_none(py));
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

    #[cfg(feature = "macros")]
    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    #[test]
    fn test_critical_section() {
        let barrier = Barrier::new(2);

        let bool_wrapper = Python::attach(|py| -> Py<BoolWrapper> {
            Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap()
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    let b = bool_wrapper.bind(py);
                    with_critical_section(b, || {
                        barrier.wait();
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        b.borrow().0.store(true, Ordering::Release);
                    })
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    let b = bool_wrapper.bind(py);
                    // this blocks until the other thread's critical section finishes
                    with_critical_section(b, || {
                        assert!(b.borrow().0.load(Ordering::Acquire));
                    });
                });
            });
        });
    }

    #[cfg(feature = "macros")]
    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    #[test]
    fn test_critical_section2() {
        let barrier = Barrier::new(3);

        let (bool_wrapper1, bool_wrapper2) = Python::attach(|py| {
            (
                Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap(),
                Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap(),
            )
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    let b1 = bool_wrapper1.bind(py);
                    let b2 = bool_wrapper2.bind(py);
                    with_critical_section2(b1, b2, || {
                        barrier.wait();
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        b1.borrow().0.store(true, Ordering::Release);
                        b2.borrow().0.store(true, Ordering::Release);
                    })
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    let b1 = bool_wrapper1.bind(py);
                    // this blocks until the other thread's critical section finishes
                    with_critical_section(b1, || {
                        assert!(b1.borrow().0.load(Ordering::Acquire));
                    });
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    let b2 = bool_wrapper2.bind(py);
                    // this blocks until the other thread's critical section finishes
                    with_critical_section(b2, || {
                        assert!(b2.borrow().0.load(Ordering::Acquire));
                    });
                });
            });
        });
    }

    #[cfg(feature = "macros")]
    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    #[test]
    fn test_critical_section2_same_object_no_deadlock() {
        let barrier = Barrier::new(2);

        let bool_wrapper = Python::attach(|py| -> Py<BoolWrapper> {
            Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap()
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    let b = bool_wrapper.bind(py);
                    with_critical_section2(b, b, || {
                        barrier.wait();
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        b.borrow().0.store(true, Ordering::Release);
                    })
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    let b = bool_wrapper.bind(py);
                    // this blocks until the other thread's critical section finishes
                    with_critical_section(b, || {
                        assert!(b.borrow().0.load(Ordering::Acquire));
                    });
                });
            });
        });
    }

    #[cfg(feature = "macros")]
    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    #[test]
    fn test_critical_section2_two_containers() {
        let (vec1, vec2) = Python::attach(|py| {
            (
                Py::new(py, VecWrapper(vec![1, 2, 3])).unwrap(),
                Py::new(py, VecWrapper(vec![4, 5])).unwrap(),
            )
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    let v1 = vec1.bind(py);
                    let v2 = vec2.bind(py);
                    with_critical_section2(v1, v2, || {
                        // v2.extend(v1)
                        v2.borrow_mut().0.extend(v1.borrow().0.iter());
                    })
                });
            });
            s.spawn(|| {
                Python::attach(|py| {
                    let v1 = vec1.bind(py);
                    let v2 = vec2.bind(py);
                    with_critical_section2(v1, v2, || {
                        // v1.extend(v2)
                        v1.borrow_mut().0.extend(v2.borrow().0.iter());
                    })
                });
            });
        });

        Python::attach(|py| {
            let v1 = vec1.bind(py);
            let v2 = vec2.bind(py);
            // execution order is not guaranteed, so we need to check both
            // NB: extend should be atomic, items must not be interleaved
            // v1.extend(v2)
            // v2.extend(v1)
            let expected1_vec1 = vec![1, 2, 3, 4, 5];
            let expected1_vec2 = vec![4, 5, 1, 2, 3, 4, 5];
            // v2.extend(v1)
            // v1.extend(v2)
            let expected2_vec1 = vec![1, 2, 3, 4, 5, 1, 2, 3];
            let expected2_vec2 = vec![4, 5, 1, 2, 3];

            assert!(
                (v1.borrow().0.eq(&expected1_vec1) && v2.borrow().0.eq(&expected1_vec2))
                    || (v1.borrow().0.eq(&expected2_vec1) && v2.borrow().0.eq(&expected2_vec2))
            );
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
        assert!(*guard == 42);
    }
}
