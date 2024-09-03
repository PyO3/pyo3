use std::ops::Deref;

/// Wrapper for [`PyMutex`](https://docs.python.org/3/c-api/init.html#c.PyMutex) exposing an RAII interface.
#[derive(Debug)]
pub struct PyMutex<T> {
    _mutex: crate::ffi::PyMutex,
    data: T,
}

/// RAII guard to handle releasing a PyMutex lock.
#[derive(Debug)]
pub struct PyMutexGuard<'a, T> {
    _mutex: &'a mut crate::ffi::PyMutex,
    data: &'a T,
}

impl<T> PyMutex<T> {
    /// Acquire the mutex, blocking the current thread until it is able to do so.
    pub fn lock(&mut self) -> PyMutexGuard<'_, T> {
        unsafe { crate::ffi::PyMutex_Lock(&mut self._mutex) };
        PyMutexGuard {
            _mutex: &mut self._mutex,
            data: &self.data,
        }
    }

    /// Create a new mutex in an unlocked state ready for use.
    pub fn new(value: T) -> Self {
        Self {
            _mutex: crate::ffi::PyMutex::new(),
            data: value,
        }
    }
}

impl<'a, T> Drop for PyMutexGuard<'a, T> {
    fn drop(&mut self) {
        unsafe { crate::ffi::PyMutex_Unlock(self._mutex) };
    }
}

impl<'a, T> Deref for PyMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyNone};
    use crate::Python;

    #[test]
    fn test_pymutex() {
        Python::with_gil(|py| {
            let d = PyDict::new(py);
            let mut mutex = PyMutex::new(&d);

            let list = Python::with_gil(|py| PyList::new(py, vec!["foo", "bar"]).unbind());
            let dict_guard = mutex.lock();

            py.allow_threads(|| {
                std::thread::spawn(move || {
                    drop(list);
                })
                .join()
                .unwrap();
            });

            dict_guard
                .set_item(PyNone::get(py), PyNone::get(py))
                .unwrap();
            drop(dict_guard);

            assert!(d
                .get_item(PyNone::get(py))
                .unwrap()
                .unwrap()
                .eq(PyNone::get(py))
                .unwrap());
        });
    }
}
