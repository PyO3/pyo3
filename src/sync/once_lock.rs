use crate::{
    internal::state::SuspendAttach, types::any::PyAnyMethods, Bound, Py, PyResult, PyTypeCheck,
    Python,
};

/// An equivalent to [`std::sync::OnceLock`] for initializing objects while attached to
/// the Python interpreter.
///
/// Unlike `OnceLock<T>`, this type will not deadlock with the interpreter.
/// Before blocking calls the cell will detach from the runtime and then
/// re-attach once the cell is unblocked.
///
/// # Re-entrant initialization
///
/// Like `OnceLock<T>`, it is an error to re-entrantly initialize a `PyOnceLock<T>`. The exact
/// behavior in this case is not guaranteed, it may either deadlock or panic.
///
/// # Examples
///
/// The following example shows how to use `PyOnceLock` to share a reference to a Python list
/// between threads:
///
/// ```
/// use pyo3::sync::PyOnceLock;
/// use pyo3::prelude::*;
/// use pyo3::types::PyList;
///
/// static LIST_CELL: PyOnceLock<Py<PyList>> = PyOnceLock::new();
///
/// pub fn get_shared_list(py: Python<'_>) -> &Bound<'_, PyList> {
///     LIST_CELL
///         .get_or_init(py, || PyList::empty(py).unbind())
///         .bind(py)
/// }
/// # Python::attach(|py| assert_eq!(get_shared_list(py).len(), 0));
/// ```
#[derive(Default)]
pub struct PyOnceLock<T> {
    inner: once_cell::sync::OnceCell<T>,
}

impl<T> PyOnceLock<T> {
    /// Create a `PyOnceLock` which does not yet contain a value.
    pub const fn new() -> Self {
        Self {
            inner: once_cell::sync::OnceCell::new(),
        }
    }

    /// Get a reference to the contained value, or `None` if the cell has not yet been written.
    #[inline]
    pub fn get(&self, _py: Python<'_>) -> Option<&T> {
        self.inner.get()
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
        self.inner
            .get()
            .unwrap_or_else(|| init_once_cell_py_attached(&self.inner, py, f))
    }

    /// Like `get_or_init`, but accepts a fallible initialization function. If it fails, the cell
    /// is left uninitialized.
    ///
    /// See the type-level documentation for detail on re-entrancy and concurrent initialization.
    pub fn get_or_try_init<F, E>(&self, py: Python<'_>, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        self.inner
            .get()
            .map_or_else(|| try_init_once_cell_py_attached(&self.inner, py, f), Ok)
    }

    /// Get the contents of the cell mutably. This is only possible if the reference to the cell is
    /// unique.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.inner.get_mut()
    }

    /// Set the value in the cell.
    ///
    /// If the cell has already been written, `Err(value)` will be returned containing the new
    /// value which was not written.
    pub fn set(&self, _py: Python<'_>, value: T) -> Result<(), T> {
        self.inner.set(value)
    }

    /// Takes the value out of the cell, moving it back to an uninitialized state.
    ///
    /// Has no effect and returns None if the cell has not yet been written.
    pub fn take(&mut self) -> Option<T> {
        self.inner.take()
    }

    /// Consumes the cell, returning the wrapped value.
    ///
    /// Returns None if the cell has not yet been written.
    pub fn into_inner(self) -> Option<T> {
        self.inner.into_inner()
    }
}

impl<T> PyOnceLock<Py<T>> {
    /// Creates a new cell that contains a new Python reference to the same contained object.
    ///
    /// Returns an uninitialized cell if `self` has not yet been initialized.
    pub fn clone_ref(&self, py: Python<'_>) -> Self {
        let cloned = PyOnceLock::new();
        if let Some(value) = self.get(py) {
            let _ = cloned.set(py, value.clone_ref(py));
        }
        cloned
    }
}

impl<T> PyOnceLock<Py<T>>
where
    T: PyTypeCheck,
{
    /// This is a shorthand method for `get_or_init` which imports the type from Python on init.
    ///
    /// # Example: Using `PyOnceLock` to store a class in a static variable.
    ///
    /// `PyOnceLock` can be used to avoid importing a class multiple times:
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::sync::PyOnceLock;
    /// # use pyo3::types::{PyDict, PyType};
    /// # use pyo3::intern;
    /// #
    /// #[pyfunction]
    /// fn create_ordered_dict<'py>(py: Python<'py>, dict: Bound<'py, PyDict>) -> PyResult<Bound<'py, PyAny>> {
    ///     // Even if this function is called multiple times,
    ///     // the `OrderedDict` class will be imported only once.
    ///     static ORDERED_DICT: PyOnceLock<Py<PyType>> = PyOnceLock::new();
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
            let type_object = py
                .import(module_name)?
                .getattr(attr_name)?
                .downcast_into()?;
            Ok(type_object.unbind())
        })
        .map(|ty| ty.bind(py))
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
    let ts_guard = unsafe { SuspendAttach::new() };

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
    let ts_guard = unsafe { SuspendAttach::new() };

    // By having detached here, we guarantee that `.get_or_init` cannot deadlock with
    // the Python interpreter
    cell.get_or_try_init(move || {
        drop(ts_guard);
        f()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_once_cell() {
        Python::attach(|py| {
            let mut cell = PyOnceLock::new();

            assert!(cell.get(py).is_none());

            assert_eq!(cell.get_or_try_init(py, || Err(5)), Err(5));
            assert!(cell.get(py).is_none());

            assert_eq!(cell.get_or_try_init(py, || Ok::<_, ()>(2)), Ok(&2));
            assert_eq!(cell.get(py), Some(&2));

            assert_eq!(cell.get_or_try_init(py, || Err(5)), Ok(&2));

            assert_eq!(cell.take(), Some(2));
            assert_eq!(cell.into_inner(), None);

            let cell_py = PyOnceLock::new();
            assert!(cell_py.clone_ref(py).get(py).is_none());
            cell_py.get_or_init(py, || py.None());
            assert!(cell_py.clone_ref(py).get(py).unwrap().is_none(py));
        })
    }

    #[test]
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
            let cell = PyOnceLock::new();
            cell.set(py, RecordDrop(&mut dropped)).unwrap();
            let drop_container = cell.get(py).unwrap();

            assert!(!*drop_container.0);
            drop(cell);
            assert!(dropped);
        });
    }
}
