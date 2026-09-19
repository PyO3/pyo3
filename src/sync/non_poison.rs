use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

/// Wrapper for [`PyMutex`](https://docs.python.org/3/c-api/init.html#c.PyMutex), exposing an RAII guard interface.
///
/// Compared with `std::sync::Mutex` or `parking_lot::Mutex`, this is a very
/// stripped-down locking primitive that only supports blocking lock and unlock
/// operations and does not support `try_lock` or APIs that depend on
/// `try_lock`.  For this reason, it is not possible to avoid the possibility of
/// possibly blocking when calling `lock` and extreme care must be taken to avoid
/// introducing a deadlock.
///
/// This type is most useful when arbitrary Python code might execute while the
/// lock is held. On the GIL-enabled build, PyMutex will release the GIL if the
/// thread is blocked on acquiring the lock. On the free-threaded build, threads
/// blocked on acquiring a PyMutex will not prevent the garbage collector from
/// running.
pub struct PyMutex<T: ?Sized> {
    pub(crate) mutex: UnsafeCell<crate::ffi::PyMutex>,
    pub(crate) data: UnsafeCell<T>,
}

/// RAII guard to handle releasing a PyMutex lock.
///
/// The lock is released when `PyMutexGuard` is dropped.
pub struct PyMutexGuard<'a, T: ?Sized> {
    inner: &'a PyMutex<T>,
    // this is equivalent to impl !Send, which we can't do
    // because negative trait bounds aren't supported yet
    _phantom: PhantomData<*const ()>,
}

/// SAFETY: `T` must be `Sync` for a [`PyMutexGuard<T>`] to be `Sync`
/// because it is possible to get a `&T` from `&MutexGuard` (via `Deref`).
unsafe impl<T: ?Sized + Sync> Sync for PyMutexGuard<'_, T> {}

/// SAFETY: `T` must be `Send` for a [`PyMutex`] to be `Send` because it is possible to acquire
/// the owned `T` from the `PyMutex` via [`into_inner`].
///
/// [`into_inner`]: PyMutex::into_inner
unsafe impl<T: ?Sized + Send> Send for PyMutex<T> {}

/// SAFETY: `T` must be `Send` for [`PyMutex`] to be `Sync`.
/// This ensures that the protected data can be accessed safely from multiple threads
/// without causing data races or other unsafe behavior.
///
/// [`PyMutex<T>`] provides mutable access to `T` to one thread at a time. However, it's essential
/// for `T` to be `Send` because it's not safe for non-`Send` structures to be accessed in
/// this manner. For instance, consider [`Rc`], a non-atomic reference counted smart pointer,
/// which is not `Send`. With `Rc`, we can have multiple copies pointing to the same heap
/// allocation with a non-atomic reference count. If we were to use `Mutex<Rc<_>>`, it would
/// only protect one instance of `Rc` from shared access, leaving other copies vulnerable
/// to potential data races.
///
/// Also note that it is not necessary for `T` to be `Sync` as `&T` is only made available
/// to one thread at a time if `T` is not `Sync`.
///
/// [`Rc`]: alloc::rc::Rc
unsafe impl<T: ?Sized + Send> Sync for PyMutex<T> {}

impl<T> PyMutex<T> {
    /// Acquire the mutex, blocking the current thread until it is able to do so.
    pub fn lock(&self) -> PyMutexGuard<'_, T> {
        // SAFETY: valid pointer to mutex passed to `PyMutex_Lock`
        // and the mutex is not moved while locked
        unsafe { crate::ffi::PyMutex_Lock(self.mutex.get()) };
        PyMutexGuard::new(self)
    }

    /// Create a new mutex in an unlocked state ready for use.
    pub const fn new(value: T) -> Self {
        Self {
            mutex: UnsafeCell::new(crate::ffi::PyMutex::new()),
            data: UnsafeCell::new(value),
        }
    }

    /// Check if the mutex is locked.
    ///
    /// Note that this is only useful for debugging or test purposes and should
    /// not be used to make concurrency control decisions, as the lock state may
    /// change immediately after the check.
    #[cfg(Py_3_14)]
    pub fn is_locked(&self) -> bool {
        // SAFETY: valid pointer to mutex passed to `PyMutex_IsLocked`
        let ret = unsafe { crate::ffi::PyMutex_IsLocked(self.mutex.get()) };
        ret != 0
    }

    /// Consumes this mutex, returning the underlying data.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the underlying data
    /// instead.
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.data.into_inner()
    }
}

impl<'mutex, T: ?Sized> PyMutexGuard<'mutex, T> {
    fn new(lock: &'mutex PyMutex<T>) -> PyMutexGuard<'mutex, T> {
        PyMutexGuard {
            inner: lock,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: ?Sized> Drop for PyMutexGuard<'a, T> {
    fn drop(&mut self) {
        // SAFETY: valid pointer to mutex passed to `PyMutex_Unlock`
        unsafe { crate::ffi::PyMutex_Unlock(self.inner.mutex.get()) };
    }
}

impl<'a, T> Deref for PyMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // safety: cannot be null pointer because PyMutex::new always
        // creates a valid PyMutex pointer
        unsafe { &*self.inner.data.get() }
    }
}

impl<'a, T> DerefMut for PyMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        // safety: cannot be null pointer because PyMutex::new always
        // creates a valid PyMutex pointer
        unsafe { &mut *self.inner.data.get() }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    #[cfg(not(target_arch = "wasm32"))]
    use core::sync::atomic::{AtomicBool, Ordering};
    #[cfg(not(target_arch = "wasm32"))]
    use std::sync::Barrier;

    use super::*;
    #[cfg(not(target_arch = "wasm32"))]
    use crate::types::{PyAnyMethods, PyDict, PyDictMethods, PyNone};
    #[cfg(not(target_arch = "wasm32"))]
    use crate::Py;
    #[cfg(not(target_arch = "wasm32"))]
    use crate::Python;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_pymutex() {
        let mutex = Python::attach(|py| -> PyMutex<Py<PyDict>> {
            let d = PyDict::new(py);
            PyMutex::new(d.unbind())
        });
        #[cfg_attr(not(Py_3_14), allow(unused_variables))]
        let mutex = Python::attach(|py| {
            let mutex = py.detach(|| -> PyMutex<Py<PyDict>> {
                std::thread::spawn(|| {
                    let dict_guard = mutex.lock();
                    Python::attach(|py| {
                        let dict = dict_guard.bind(py);
                        dict.set_item(PyNone::get(py), PyNone::get(py)).unwrap();
                    });
                    #[cfg(Py_3_14)]
                    assert!(mutex.is_locked());
                    drop(dict_guard);
                    #[cfg(Py_3_14)]
                    assert!(!mutex.is_locked());
                    mutex
                })
                .join()
                .unwrap()
            });

            let dict_guard = mutex.lock();
            #[cfg(Py_3_14)]
            assert!(mutex.is_locked());
            let d = dict_guard.bind(py);

            assert!(d
                .get_item(PyNone::get(py))
                .unwrap()
                .unwrap()
                .eq(PyNone::get(py))
                .unwrap());
            #[cfg(Py_3_14)]
            assert!(mutex.is_locked());
            drop(dict_guard);
            #[cfg(Py_3_14)]
            assert!(!mutex.is_locked());
            mutex
        });
        #[cfg(Py_3_14)]
        assert!(!mutex.is_locked());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_pymutex_blocks() {
        let mutex = PyMutex::new(());
        let first_thread_locked_once = AtomicBool::new(false);
        let second_thread_locked_once = AtomicBool::new(false);
        let finished = AtomicBool::new(false);
        let barrier = Barrier::new(2);

        std::thread::scope(|s| {
            s.spawn(|| {
                let guard = mutex.lock();
                first_thread_locked_once.store(true, Ordering::SeqCst);
                while !finished.load(Ordering::SeqCst) {
                    if second_thread_locked_once.load(Ordering::SeqCst) {
                        // Wait a little to guard against the unlikely event that
                        // the other thread isn't blocked on acquiring the mutex yet.
                        // If PyMutex had a try_lock implementation this would be
                        // unnecessary
                        std::thread::sleep(core::time::Duration::from_millis(10));
                        // block (and hold the mutex) until the receiver actually receives something
                        barrier.wait();
                        finished.store(true, Ordering::SeqCst);
                    }
                }
                drop(guard);
            });

            s.spawn(|| {
                while !first_thread_locked_once.load(Ordering::SeqCst) {
                    core::hint::spin_loop();
                }
                second_thread_locked_once.store(true, Ordering::SeqCst);
                let guard = mutex.lock();
                assert!(finished.load(Ordering::SeqCst));
                drop(guard);
            });

            barrier.wait();
        });
    }

    #[test]
    fn test_send_not_send() {
        use crate::impl_::pyclass::{value_of, IsSend, IsSync};

        assert!(!value_of!(IsSend, PyMutexGuard<'_, i32>));
        assert!(value_of!(IsSync, PyMutexGuard<'_, i32>));

        assert!(value_of!(IsSend, PyMutex<i32>));
        assert!(value_of!(IsSync, PyMutex<i32>));
    }
}
