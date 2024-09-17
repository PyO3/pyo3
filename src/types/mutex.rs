use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

/// Wrapper for [`PyMutex`](https://docs.python.org/3/c-api/init.html#c.PyMutex), exposing an RAII guard interface similar to `std::sync::Mutex`.
#[derive(Debug)]
pub struct PyMutex<T> {
    _mutex: UnsafeCell<crate::ffi::PyMutex>,
    data: UnsafeCell<T>,
}

/// RAII guard to handle releasing a PyMutex lock.
#[derive(Debug)]
pub struct PyMutexGuard<'a, T> {
    mutex: &'a PyMutex<T>,
}

impl<T> PyMutex<T> {
    /// Acquire the mutex, blocking the current thread until it is able to do so.
    pub fn lock(&self) -> PyMutexGuard<'_, T> {
        unsafe { crate::ffi::PyMutex_Lock(UnsafeCell::raw_get(&self._mutex)) };
        PyMutexGuard { mutex: self }
    }

    /// Create a new mutex in an unlocked state ready for use.
    pub fn new(value: T) -> Self {
        Self {
            _mutex: UnsafeCell::new(crate::ffi::PyMutex::new()),
            data: UnsafeCell::new(value),
        }
    }
}

unsafe impl<T: Send> Sync for PyMutex<T> {}

impl<'a, T> Drop for PyMutexGuard<'a, T> {
    fn drop(&mut self) {
        unsafe { crate::ffi::PyMutex_Unlock(UnsafeCell::raw_get(&self.mutex._mutex)) };
    }
}

impl<'a, T> Deref for PyMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // safety: cannot be null pointer because PyMutexGuard::new always
        // creates a valid PyMutex pointer
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> DerefMut for PyMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        // safety: cannot be null pointer because PyMutexGuard::new always
        // creates a valid PyMutex pointer
        unsafe { &mut *self.mutex.data.get() }
    }
}

#[cfg(test)]
mod tests {
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
}
