//! Synchronization mechanisms based on the Python GIL.
//!
//! With the acceptance of [PEP 703] (aka a "freethreaded Python") for Python 3.13, these
//! are likely to undergo significant developments in the future.
//!
//! [PEP 703]: https://peps.python.org/pep-703/
use crate::{
    types::{any::PyAnyMethods, PyString, PyType},
    Bound, Py, PyResult, PyVisit, Python,
};
use std::cell::UnsafeCell;

/// Value with concurrent access protected by the GIL.
///
/// This is a synchronization primitive based on Python's global interpreter lock (GIL).
/// It ensures that only one thread at a time can access the inner value via shared references.
/// It can be combined with interior mutability to obtain mutable references.
///
/// # Example
///
/// Combining `GILProtected` with `RefCell` enables mutable access to static data:
///
/// ```
/// # use pyo3::prelude::*;
/// use pyo3::sync::GILProtected;
/// use std::cell::RefCell;
///
/// static NUMBERS: GILProtected<RefCell<Vec<i32>>> = GILProtected::new(RefCell::new(Vec::new()));
///
/// Python::with_gil(|py| {
///     NUMBERS.get(py).borrow_mut().push(42);
/// });
/// ```
pub struct GILProtected<T> {
    value: T,
}

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

unsafe impl<T> Sync for GILProtected<T> where T: Send {}

/// A write-once cell similar to [`once_cell::OnceCell`](https://docs.rs/once_cell/latest/once_cell/).
///
/// Unlike `once_cell::sync` which blocks threads to achieve thread safety, this implementation
/// uses the Python GIL to mediate concurrent access. This helps in cases where `once_cell` or
/// `lazy_static`'s synchronization strategy can lead to deadlocks when interacting with the Python
/// GIL. For an example, see
#[doc = concat!("[the FAQ section](https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/faq.html)")]
/// of the guide.
///
/// Note that:
///  1) `get_or_init` and `get_or_try_init` do not protect against infinite recursion
///     from reentrant initialization.
///  2) If the initialization function `f` provided to `get_or_init` (or `get_or_try_init`)
///     temporarily releases the GIL (e.g. by calling `Python::import`) then it is possible
///     for a second thread to also begin initializing the `GITOnceCell`. Even when this
///     happens `GILOnceCell` guarantees that only **one** write to the cell ever occurs -
///     this is treated as a race, other threads will discard the value they compute and
///     return the result of the first complete computation.
///
/// # Examples
///
/// The following example shows how to use `GILOnceCell` to share a reference to a Python list
/// between threads:
///
/// ```
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
/// # Python::with_gil(|py| assert_eq!(get_shared_list(py).len(), 0));
/// ```
#[derive(Default)]
pub struct GILOnceCell<T>(UnsafeCell<Option<T>>);

// T: Send is needed for Sync because the thread which drops the GILOnceCell can be different
// to the thread which fills it.
unsafe impl<T: Send + Sync> Sync for GILOnceCell<T> {}
unsafe impl<T: Send> Send for GILOnceCell<T> {}

impl<T> GILOnceCell<T> {
    /// Create a `GILOnceCell` which does not yet contain a value.
    pub const fn new() -> Self {
        Self(UnsafeCell::new(None))
    }

    /// Get a reference to the contained value, or `None` if the cell has not yet been written.
    #[inline]
    pub fn get(&self, _py: Python<'_>) -> Option<&T> {
        // Safe because if the cell has not yet been written, None is returned.
        unsafe { &*self.0.get() }.as_ref()
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
        let value = f()?;
        let _ = self.set(py, value);

        Ok(self.get(py).unwrap())
    }

    /// Get the contents of the cell mutably. This is only possible if the reference to the cell is
    /// unique.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.0.get_mut().as_mut()
    }

    /// Set the value in the cell.
    ///
    /// If the cell has already been written, `Err(value)` will be returned containing the new
    /// value which was not written.
    pub fn set(&self, _py: Python<'_>, value: T) -> Result<(), T> {
        // Safe because GIL is held, so no other thread can be writing to this cell concurrently.
        let inner = unsafe { &mut *self.0.get() };
        if inner.is_some() {
            return Err(value);
        }

        *inner = Some(value);
        Ok(())
    }

    /// Takes the value out of the cell, moving it back to an uninitialized state.
    ///
    /// Has no effect and returns None if the cell has not yet been written.
    pub fn take(&mut self) -> Option<T> {
        self.0.get_mut().take()
    }

    /// Consumes the cell, returning the wrapped value.
    ///
    /// Returns None if the cell has not yet been written.
    pub fn into_inner(self) -> Option<T> {
        self.0.into_inner()
    }
}

impl GILOnceCell<Py<PyType>> {
    /// Get a reference to the contained Python type, initializing it if needed.
    ///
    /// This is a shorthand method for `get_or_init` which imports the type from Python on init.
    pub(crate) fn get_or_try_init_type_ref<'py>(
        &self,
        py: Python<'py>,
        module_name: &str,
        attr_name: &str,
    ) -> PyResult<&Bound<'py, PyType>> {
        self.get_or_try_init(py, || {
            let type_object = py
                .import_bound(module_name)?
                .getattr(attr_name)?
                .downcast_into()?;
            Ok(type_object.unbind())
        })
        .map(|ty| ty.bind(py))
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
/// # Python::with_gil(|py| {
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
pub struct Interned(&'static str, GILOnceCell<Py<PyString>>);

impl Interned {
    /// Creates an empty holder for an interned `str`.
    pub const fn new(value: &'static str) -> Self {
        Interned(value, GILOnceCell::new())
    }

    /// Gets or creates the interned `str` value.
    #[inline]
    pub fn get<'py>(&self, py: Python<'py>) -> &Bound<'py, PyString> {
        self.1
            .get_or_init(py, || PyString::intern(py, self.0).into())
            .bind(py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::{dict::PyDictMethods, PyDict};

    #[test]
    fn test_intern() {
        Python::with_gil(|py| {
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
    fn test_once_cell() {
        Python::with_gil(|py| {
            let mut cell = GILOnceCell::new();

            assert!(cell.get(py).is_none());

            assert_eq!(cell.get_or_try_init(py, || Err(5)), Err(5));
            assert!(cell.get(py).is_none());

            assert_eq!(cell.get_or_try_init(py, || Ok::<_, ()>(2)), Ok(&2));
            assert_eq!(cell.get(py), Some(&2));

            assert_eq!(cell.get_or_try_init(py, || Err(5)), Ok(&2));

            assert_eq!(cell.take(), Some(2));
            assert_eq!(cell.into_inner(), None)
        })
    }
}
