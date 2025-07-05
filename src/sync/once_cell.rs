use crate::{gil::SuspendGIL, Python};

mod once_lock_ext_sealed {
    pub trait Sealed {}
    impl<T> Sealed for once_cell::sync::OnceCell<T> {}
}

/// Extension trait for [`once_cell::sync::OnceCell`] which helps avoid deadlocks between the Python
/// interpreter and initialization with the `OnceCell`.
pub trait OnceCellExt<T>: once_lock_ext_sealed::Sealed {
    /// Initializes this `OnceCell` with the given closure if it has not been initialized yet.
    ///
    /// If this function would block, this function detaches from the Python interpreter and
    /// reattaches before calling `f`. This avoids deadlocks between the Python interpreter and
    /// the `OnceCell` in cases where `f` can call arbitrary Python code, as calling arbitrary
    /// Python code can lead to `f` itself blocking on the Python interpreter.
    ///
    /// By detaching from the Python interpreter before blocking, this ensures that if `f` blocks
    /// then the Python interpreter cannot be blocked by `f` itself.
    fn get_or_init_py_attached<F>(&self, py: Python<'_>, f: F) -> &T
    where
        F: FnOnce() -> T;

    /// Attempts to initialize this `OnceCell` with the given fallible closure if it has not been
    /// initialized yet.
    ///
    /// If this function would block, this function detaches from the Python interpreter and
    /// reattaches before calling `f`. This avoids deadlocks between the Python interpreter and
    /// the `OnceCell` in cases where `f` can call arbitrary Python code, as calling arbitrary
    /// Python code can lead to `f` itself blocking on the Python interpreter.
    ///
    /// By detaching from the Python interpreter before blocking, this ensures that if `f` blocks
    /// then the Python interpreter cannot be blocked by `f` itself.
    fn get_or_try_init_py_attached<F, E>(&self, py: Python<'_>, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>;
}

impl<T> OnceCellExt<T> for once_cell::sync::OnceCell<T> {
    fn get_or_init_py_attached<F>(&self, py: Python<'_>, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.get()
            .unwrap_or_else(|| init_once_cell_py_attached(self, py, f))
    }

    fn get_or_try_init_py_attached<F, E>(&self, py: Python<'_>, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        self.get()
            .map_or_else(|| try_init_once_cell_py_attached(self, py, f), Ok)
    }
}

#[cold]
fn init_once_cell_py_attached<'a, F, T>(
    cell: &'a once_cell::sync::OnceCell<T>,
    _py: Python<'_>,
    f: F,
) -> &'a T
where
    F: FnOnce() -> T,
{
    // SAFETY: detach from the runtime right before a possibly blocking call
    // then reattach when the blocking call completes and before calling
    // into the C API.
    let ts_guard = unsafe { SuspendGIL::new() };

    // By having detached here, we guarantee that `.get_or_init` cannot deadlock with
    // the Python interpreter
    cell.get_or_init(move || {
        drop(ts_guard);
        f()
    })
}

#[cold]
fn try_init_once_cell_py_attached<'a, F, T, E>(
    cell: &'a once_cell::sync::OnceCell<T>,
    _py: Python<'_>,
    f: F,
) -> Result<&'a T, E>
where
    F: FnOnce() -> Result<T, E>,
{
    // SAFETY: detach from the runtime right before a possibly blocking call
    // then reattach when the blocking call completes and before calling
    // into the C API.
    let ts_guard = unsafe { SuspendGIL::new() };

    // By having detached here, we guarantee that `.get_or_init` cannot deadlock with
    // the Python interpreter
    cell.get_or_try_init(move || {
        drop(ts_guard);
        f()
    })
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
mod tests {
    use super::*;

    #[test]
    fn test_once_lock_ext() {
        let cell = once_cell::sync::OnceCell::new();
        std::thread::scope(|s| {
            assert!(cell.get().is_none());

            s.spawn(|| {
                Python::attach(|py| {
                    assert_eq!(*cell.get_or_init_py_attached(py, || 12345), 12345);

                    // initializing again should not change the value
                    assert_eq!(*cell.get_or_init_py_attached(py, || 23456), 12345);
                });
            });
        });
        assert_eq!(cell.get(), Some(&12345));
    }

    #[test]
    fn test_once_lock_ext_try_init() {
        type R = Result<i32, i32>;

        let cell = once_cell::sync::OnceCell::new();
        std::thread::scope(|s| {
            assert!(cell.get().is_none());

            s.spawn(|| {
                Python::attach(|py| {
                    // initializing error
                    assert_eq!(
                        cell.get_or_try_init_py_attached(py, || Err(12345)),
                        Err(12345)
                    );

                    assert!(cell.get().is_none());

                    // successful initialization
                    assert_eq!(
                        cell.get_or_try_init_py_attached(py, || R::Ok(12345)),
                        Ok(&12345)
                    );

                    // initializing again should not change the value
                    assert_eq!(
                        cell.get_or_try_init_py_attached(py, || R::Ok(23456)),
                        Ok(&12345)
                    );

                    // even if re-init would fail
                    assert_eq!(
                        cell.get_or_try_init_py_attached(py, || Err(23456)),
                        Ok(&12345)
                    );
                });
            });
        });
        assert_eq!(cell.get(), Some(&12345));
    }
}
