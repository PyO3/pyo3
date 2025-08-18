use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
#[cfg(panic = "unwind")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LockResult, PoisonError};
#[cfg(panic = "unwind")]
use std::thread;

// See std::sync::poison in the rust standard library.
// This is more-or-less copied from there since it is not public.
// this type detects a panic and poisons the wrapping mutex
struct Flag {
    #[cfg(panic = "unwind")]
    failed: AtomicBool,
}

impl Flag {
    #[inline]
    const fn new() -> Flag {
        Flag {
            #[cfg(panic = "unwind")]
            failed: AtomicBool::new(false),
        }
    }

    /// Checks the flag for an unguarded borrow, where we only care about existing poison.
    #[inline]
    fn borrow(&self) -> LockResult<()> {
        if self.get() {
            Err(PoisonError::new(()))
        } else {
            Ok(())
        }
    }

    /// Checks the flag for a guarded borrow, where we may also set poison when `done`.
    #[inline]
    fn guard(&self) -> LockResult<Guard> {
        let ret = Guard {
            #[cfg(panic = "unwind")]
            panicking: thread::panicking(),
        };
        if self.get() {
            Err(PoisonError::new(ret))
        } else {
            Ok(ret)
        }
    }

    #[inline]
    #[cfg(panic = "unwind")]
    fn done(&self, guard: &Guard) {
        if !guard.panicking && thread::panicking() {
            self.failed.store(true, Ordering::Relaxed);
        }
    }

    #[inline]
    #[cfg(not(panic = "unwind"))]
    fn done(&self, _guard: &Guard) {}

    #[inline]
    #[cfg(panic = "unwind")]
    fn get(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }

    #[inline(always)]
    #[cfg(not(panic = "unwind"))]
    fn get(&self) -> bool {
        false
    }

    #[inline]
    fn clear(&self) {
        #[cfg(panic = "unwind")]
        self.failed.store(false, Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub(crate) struct Guard {
    #[cfg(panic = "unwind")]
    panicking: bool,
}

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
///
/// ## Poisoning
///
/// Like `std::sync::Mutex`, `PyMutex` implements poisoning. A mutex
/// is considered poisoned whenever a thread panics while holding the mutex. Once
/// a mutex is poisoned, all other threads are unable to access the data by
/// default as it is likely to be tainted (some invariant is not being held).
///
/// This means that the `lock` method returns a `Result` which indicated whether
/// the mutex has been poisoned or not. Must usage will simple `unwrap()` these
/// results, propagating panics among threads to ensure a possible invalid
/// invariant is not being observed.
///
/// A poisoned mutex, however, does not prevent all access to the underlying
/// data. The `PoisonError` type has an `into_inner` method which will return
/// the guard that would have otherwise been returned on a successful lock. This
/// allows access to the data, despite the lock being poisoned.
pub struct PyMutex<T: ?Sized> {
    mutex: UnsafeCell<crate::ffi::PyMutex>,
    poison: Flag,
    data: UnsafeCell<T>,
}

/// RAII guard to handle releasing a PyMutex lock.
///
/// The lock is released when `PyMutexGuard` is dropped.
pub struct PyMutexGuard<'a, T: ?Sized> {
    inner: &'a PyMutex<T>,
    poison: Guard,
    // this is equivalent to impl !Send, which we can't do
    // because negative trait bounds aren't supported yet
    _phantom: PhantomData<*const ()>,
}

/// `T` must be `Sync` for a [`PyMutexGuard<T>`] to be `Sync`
/// because it is possible to get a `&T` from `&MutexGuard` (via `Deref`).
unsafe impl<T: ?Sized + Sync> Sync for PyMutexGuard<'_, T> {}

/// `T` must be `Send` for a [`PyMutex`] to be `Send` because it is possible to acquire
/// the owned `T` from the `PyMutex` via [`into_inner`].
///
/// [`into_inner`]: PyMutex::into_inner
unsafe impl<T: ?Sized + Send> Send for PyMutex<T> {}

/// `T` must be `Send` for [`PyMutex`] to be `Sync`.
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
/// [`Rc`]: std::rc::Rc
unsafe impl<T: ?Sized + Send> Sync for PyMutex<T> {}

impl<T> PyMutex<T> {
    /// Acquire the mutex, blocking the current thread until it is able to do so.
    pub fn lock(&self) -> LockResult<PyMutexGuard<'_, T>> {
        unsafe { crate::ffi::PyMutex_Lock(UnsafeCell::raw_get(&self.mutex)) };
        PyMutexGuard::new(self)
    }

    /// Create a new mutex in an unlocked state ready for use.
    pub const fn new(value: T) -> Self {
        Self {
            mutex: UnsafeCell::new(crate::ffi::PyMutex::new()),
            data: UnsafeCell::new(value),
            poison: Flag::new(),
        }
    }

    /// Check if the mutex is locked.
    ///
    /// Note that this is only useful for debugging or test purposes and should
    /// not be used to make concurrency control decisions, as the lock state may
    /// change immediately after the check.
    #[cfg(Py_3_14)]
    pub fn is_locked(&self) -> bool {
        let ret = unsafe { crate::ffi::PyMutex_IsLocked(UnsafeCell::raw_get(&self.mutex)) };
        ret != 0
    }

    /// Consumes this mutex, returning the underlying data.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the underlying data
    /// instead.
    pub fn into_inner(self) -> LockResult<T>
    where
        T: Sized,
    {
        let data = self.data.into_inner();
        map_result(self.poison.borrow(), |()| data)
    }

    /// Clear the poisoned state from a mutex.
    ///
    /// If the mutex is poisoned, it will remain poisoned until this function is called. This
    /// allows recovering from a poisoned state and marking that it has recovered. For example, if
    /// the value is overwritten by a known-good value, then the mutex can be marked as
    /// un-poisoned. Or possibly, the value could be inspected to determine if it is in a
    /// consistent state, and if so the poison is removed.
    pub fn clear_poison(&self) {
        self.poison.clear();
    }
}

#[cfg_attr(not(panic = "unwind"), allow(clippy::unnecessary_wraps))]
fn map_result<T, U, F>(result: LockResult<T>, f: F) -> LockResult<U>
where
    F: FnOnce(T) -> U,
{
    match result {
        Ok(t) => Ok(f(t)),
        #[cfg(panic = "unwind")]
        Err(e) => Err(PoisonError::new(f(e.into_inner()))),
        #[cfg(not(panic = "unwind"))]
        Err(_) => {
            unreachable!();
        }
    }
}

impl<'mutex, T: ?Sized> PyMutexGuard<'mutex, T> {
    fn new(lock: &'mutex PyMutex<T>) -> LockResult<PyMutexGuard<'mutex, T>> {
        map_result(lock.poison.guard(), |guard| PyMutexGuard {
            inner: lock,
            poison: guard,
            _phantom: PhantomData,
        })
    }
}

impl<'a, T: ?Sized> Drop for PyMutexGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            self.inner.poison.done(&self.poison);
            crate::ffi::PyMutex_Unlock(UnsafeCell::raw_get(&self.inner.mutex))
        };
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
    #[cfg(not(target_arch = "wasm32"))]
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Barrier,
    };

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
                    let dict_guard = mutex.lock().unwrap();
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

            let dict_guard = mutex.lock().unwrap();
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
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        // block (and hold the mutex) until the receiver actually receives something
                        barrier.wait();
                        finished.store(true, Ordering::SeqCst);
                    }
                }
                drop(guard);
            });

            s.spawn(|| {
                while !first_thread_locked_once.load(Ordering::SeqCst) {
                    std::hint::spin_loop();
                }
                second_thread_locked_once.store(true, Ordering::SeqCst);
                let guard = mutex.lock();
                assert!(finished.load(Ordering::SeqCst));
                drop(guard);
            });

            barrier.wait();
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_recover_poison() {
        let mutex = Python::attach(|py| -> PyMutex<Py<PyDict>> {
            let d = PyDict::new(py);
            d.set_item("hello", "world").unwrap();
            PyMutex::new(d.unbind())
        });

        let lock = Arc::new(mutex);
        let lock2 = Arc::clone(&lock);

        let _ = thread::spawn(move || {
            let _guard = lock2.lock().unwrap();

            // poison the mutex
            panic!();
        })
        .join();

        // by now the lock is poisoned, use into_inner to recover the value despite that
        let guard = match lock.lock() {
            Ok(_) => {
                unreachable!();
            }
            Err(poisoned) => poisoned.into_inner(),
        };

        Python::attach(|py| {
            assert!(
                (*guard)
                    .bind(py)
                    .get_item("hello")
                    .unwrap()
                    .unwrap()
                    .extract::<&str>()
                    .unwrap()
                    == "world"
            );
        });

        // now test recovering via PyMutex::into_inner
        let mutex = PyMutex::new(0);
        assert_eq!(mutex.into_inner().unwrap(), 0);

        let mutex = PyMutex::new(0);
        let _ = std::thread::scope(|s| {
            s.spawn(|| {
                let _guard = mutex.lock().unwrap();

                // poison the mutex
                panic!();
            })
            .join()
        });

        match mutex.into_inner() {
            Ok(_) => {
                unreachable!()
            }
            Err(e) => {
                assert!(e.into_inner() == 0)
            }
        }

        // now test recovering via PyMutex::clear_poison
        let mutex = PyMutex::new(0);
        let _ = std::thread::scope(|s| {
            s.spawn(|| {
                let _guard = mutex.lock().unwrap();

                // poison the mutex
                panic!();
            })
            .join()
        });
        mutex.clear_poison();
        assert!(*mutex.lock().unwrap() == 0);
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
