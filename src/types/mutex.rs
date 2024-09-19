use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

/// Wrapper for [`PyMutex`](https://docs.python.org/3/c-api/init.html#c.PyMutex), exposing an RAII guard interface.
///
/// Comapred with `std::sync::Mutex` or `parking_lot::Mutex`, this is a very
/// stripped-down locking primitive that only supports blocking lock and unlock
/// operations.

/// `PyMutex` is hooked into CPython's garbage collector and the GIL in GIL-enabled
/// builds. If a thread is blocked on aquiring the mutex and holds the GIL or would
/// prevent Python from entering garbage collection, then Python will release the
/// thread state, allowing garbage collection or other threads blocked by the GIL to
/// proceed. This means it is impossible for PyMutex to deadlock with the GIL.
pub struct PyMutex<T: ?Sized> {
    mutex: UnsafeCell<crate::ffi::PyMutex>,
    data: UnsafeCell<T>,
}

/// RAII guard to handle releasing a PyMutex lock.
///
/// The lock is released when `PyMutexGuard` is dropped.
pub struct PyMutexGuard<'a, T> {
    inner: &'a PyMutex<T>,
}

impl<T> PyMutex<T> {
    /// Acquire the mutex, blocking the current thread until it is able to do so.
    pub fn lock(&self) -> PyMutexGuard<'_, T> {
        unsafe { crate::ffi::PyMutex_Lock(UnsafeCell::raw_get(&self.mutex)) };
        PyMutexGuard { inner: self }
    }

    /// Create a new mutex in an unlocked state ready for use.
    pub fn new(value: T) -> Self {
        Self {
            mutex: UnsafeCell::new(crate::ffi::PyMutex::new()),
            data: UnsafeCell::new(value),
        }
    }
}

// safety: PyMutex serializes access
unsafe impl<T: Send> Sync for PyMutex<T> {}

impl<'a, T> Drop for PyMutexGuard<'a, T> {
    fn drop(&mut self) {
        unsafe { crate::ffi::PyMutex_Unlock(UnsafeCell::raw_get(&self.inner.mutex)) };
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
    use std::sync::{mpsc::sync_channel, OnceLock};

    use super::*;
    use crate::types::{PyAnyMethods, PyDict, PyDictMethods, PyNone};
    use crate::Py;
    use crate::Python;

    #[test]
    fn test_pymutex() {
        let mutex = Python::with_gil(|py| -> PyMutex<Py<PyDict>> {
            let d = PyDict::new(py);
            PyMutex::new(d.unbind())
        });

        Python::with_gil(|py| {
            let mutex = py.allow_threads(|| -> PyMutex<Py<PyDict>> {
                std::thread::spawn(|| {
                    let dict_guard = mutex.lock();
                    Python::with_gil(|py| {
                        let dict = dict_guard.bind(py);
                        dict.set_item(PyNone::get(py), PyNone::get(py)).unwrap();
                    });
                    drop(dict_guard);
                    mutex
                })
                .join()
                .unwrap()
            });

            let dict_guard = mutex.lock();
            let d = dict_guard.bind(py);

            assert!(d
                .get_item(PyNone::get(py))
                .unwrap()
                .unwrap()
                .eq(PyNone::get(py))
                .unwrap());
        });
    }

    #[test]
    fn test_pymutex_blocks() {
        let mutex = OnceLock::<PyMutex<()>>::new();
        let first_thread_locked_once = OnceLock::<bool>::new();
        let second_thread_locked_once = OnceLock::<bool>::new();
        let finished = OnceLock::<bool>::new();
        let (sender, receiver) = sync_channel::<bool>(0);

        mutex.get_or_init(|| PyMutex::new(()));

        std::thread::scope(|s| {
            s.spawn(|| {
                let guard = mutex.get().unwrap().lock();
                first_thread_locked_once.set(true).unwrap();
                while finished.get().is_none() {
                    if second_thread_locked_once.get().is_some() {
                        // Wait a little to guard against the unlikely event that
                        // the other thread isn't blocked on acquiring the mutex yet.
                        // If PyMutex had a try_lock implementation this would be
                        // unnecessary
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        // block (and hold the mutex) until the receiver actually receives something
                        sender.send(true).unwrap();
                        finished.set(true).unwrap();
                    }
                }
                drop(guard);
            });

            s.spawn(|| {
                while first_thread_locked_once.get().is_none() {}
                let mutex = mutex.get().unwrap();
                second_thread_locked_once.set(true).unwrap();
                let guard = mutex.lock();
                assert!(finished.get().unwrap());
                drop(guard);
            });

            // threads are blocked until we receive
            receiver.recv().unwrap();
        });
    }
}
