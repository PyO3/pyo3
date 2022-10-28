//! A write-once cell mediated by the Python GIL.
use crate::{types::PyString, Py, Python};
use std::{
    cell::{Cell, UnsafeCell},
    ops::{Deref, DerefMut},
    panic::RefUnwindSafe,
};

/// A write-once cell similar to [`once_cell::OnceCell`](https://docs.rs/once_cell/1.4.0/once_cell/).
///
/// Unlike `once_cell::sync` which blocks threads to achieve thread safety, this implementation
/// uses the Python GIL to mediate concurrent access. This helps in cases where `once_sync` or
/// `lazy_static`'s synchronization strategy can lead to deadlocks when interacting with the Python
/// GIL. For an example, see [the FAQ section](https://pyo3.rs/latest/faq.html) of the guide.
///
/// # Examples
///
/// The following example shows how to use `GILOnceCell` to share a reference to a Python list
/// between threads:
///
/// ```
/// use pyo3::once_cell::GILOnceCell;
/// use pyo3::prelude::*;
/// use pyo3::types::PyList;
///
/// static LIST_CELL: GILOnceCell<Py<PyList>> = GILOnceCell::new();
///
/// pub fn get_shared_list(py: Python<'_>) -> &PyList {
///     LIST_CELL
///         .get_or_init(py, || PyList::empty(py).into())
///         .as_ref(py)
/// }
/// # Python::with_gil(|py| assert_eq!(get_shared_list(py).len(), 0));
/// ```
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
    /// Note that:
    ///  1) reentrant initialization can cause a stack overflow.
    ///  2) if f() temporarily releases the GIL (e.g. by calling `Python::import`) then it is
    ///     possible (and well-defined) that a second thread may also call get_or_init and begin
    ///     calling `f()`. Even when this happens `GILOnceCell` guarantees that only **one** write
    ///     to the cell ever occurs - other threads will simply discard the value they compute and
    ///     return the result of the first complete computation.
    ///  3) if f() does not release the GIL and does not panic, it is guaranteed to be called
    ///     exactly once, even if multiple threads attempt to call `get_or_init`
    ///  4) if f() can panic but still does not release the GIL, it may be called multiple times,
    ///     but it is guaranteed that f() will never be called concurrently
    #[inline]
    pub fn get_or_init<F>(&self, py: Python<'_>, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if let Some(value) = self.get(py) {
            return value;
        }

        self.init(py, f)
    }

    #[cold]
    fn init<F>(&self, py: Python<'_>, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        // Note that f() could temporarily release the GIL, so it's possible that another thread
        // writes to this GILOnceCell before f() finishes. That's fine; we'll just have to discard
        // the value computed here and accept a bit of wasted computation.
        let value = f();
        let _ = self.set(py, value);

        self.get(py).unwrap()
    }

    /// Get the contents of the cell mutably. This is only possible if the reference to the cell is
    /// unique.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        // Safe because we have &mut self
        unsafe { &mut *self.0.get() }.as_mut()
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
}

/// Interns `text` as a Python string and stores a reference to it in static storage.
///
/// A reference to the same Python string is returned on each invocation.
///
/// # Example: Using `intern!` to avoid needlessly recreating the same Python string
///
/// ```
/// use pyo3::intern;
/// # use pyo3::{pyfunction, types::PyDict, wrap_pyfunction, PyResult, Python};
///
/// #[pyfunction]
/// fn create_dict(py: Python<'_>) -> PyResult<&PyDict> {
///    let dict = PyDict::new(py);
///    //             👇 A new `PyString` is created
///    //                for every call of this function.
///    dict.set_item("foo", 42)?;
///    Ok(dict)
/// }
///
/// #[pyfunction]
/// fn create_dict_faster(py: Python<'_>) -> PyResult<&PyDict> {
///    let dict = PyDict::new(py);
///    //               👇 A `PyString` is created once and reused
///    //                  for the lifetime of the program.
///    dict.set_item(intern!(py, "foo"), 42)?;
///    Ok(dict)
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
        static INTERNED: $crate::once_cell::Interned = $crate::once_cell::Interned::new($text);
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
    pub fn get<'py>(&'py self, py: Python<'py>) -> &'py PyString {
        self.1
            .get_or_init(py, || PyString::intern(py, self.0).into())
            .as_ref(py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::PyDict;

    #[test]
    fn test_intern() {
        Python::with_gil(|py| {
            let foo1 = "foo";
            let foo2 = intern!(py, "foo");
            let foo3 = intern!(py, stringify!(foo));

            let dict = PyDict::new(py);
            dict.set_item(foo1, 42_usize).unwrap();
            assert!(dict.contains(foo2).unwrap());
            assert_eq!(dict.get_item(foo3).unwrap().extract::<usize>().unwrap(), 42);
        });
    }
}

/// A value which is initialized on the first access based on
/// [`once_cell::sync::Lazy`](https://docs.rs/once_cell/1.14.0/once_cell/sync/struct.Lazy.html).
///
/// This type is thread-safe and can be used in statics.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
///
/// use rust_circuit::lazy::GILLazy;
///
/// pyo3::prepare_freethreaded_python();
///
/// static HASHMAP: GILLazy<HashMap<i32, String>> = GILLazy::new(|| {
///     println!("initializing");
///     let mut m = HashMap::default();
///     m.insert(13, "Spica".to_string());
///     m.insert(74, "Hoyten".to_string());
///     m
/// });
///
/// fn main() {
///     println!("ready");
///     std::thread::spawn(|| {
///         println!("{:?}", HASHMAP.get(&13));
///     })
///     .join()
///     .unwrap();
///     println!("{:?}", HASHMAP.get(&74));
///
///     // Prints:
///     //   ready
///     //   initializing
///     //   Some("Spica")
///     //   Some("Hoyten")
/// }
/// ```
///
/// TODO: contribute to pyo3!
pub struct GILLazyPy<T, F = for<'py> fn(Python<'py>) -> T> {
    cell: GILOnceCell<T>,
    init: Cell<Option<F>>,
}

/// Wrapper type, see docs for GILLazyPy
pub struct PyCallWrap<F>(F);

/// Wrapper for GILLazyPy, see docs for that.
pub type GILLazy<T, S = PyCallWrap<fn() -> T>> = GILLazyPy<T, S>;

// We never create a `&F` from a `&GILLazyPy<T, F>` so it is fine to not impl
// `Sync` for `F`. We do create a `&mut Option<F>` in `force`, but this is
// properly synchronized, so it only happens once so it also does not
// contribute to this impl.
// TODO: is this ok with GILOnceCell????
unsafe impl<T, F: Send> Sync for GILLazyPy<T, F> where GILOnceCell<T>: Sync {}
// auto-derived `Send` impl is OK.

impl<T, F: RefUnwindSafe> RefUnwindSafe for GILLazyPy<T, F> where GILOnceCell<T>: RefUnwindSafe {}

impl<T, F> GILLazyPy<T, PyCallWrap<F>> {
    /// Creates a new lazy value with the given initializing
    /// function.
    pub const fn new(f: F) -> GILLazyPy<T, PyCallWrap<F>> {
        GILLazyPy {
            cell: GILOnceCell::new(),
            init: Cell::new(Some(PyCallWrap(f))),
        }
    }
}

impl<T, F> GILLazyPy<T, F> {
    /// TODO: doc
    pub const fn new_py(f: F) -> GILLazyPy<T, F> {
        GILLazyPy {
            cell: GILOnceCell::new(),
            init: Cell::new(Some(f)),
        }
    }
}

/// trait for once cell stuff
pub trait MaybePyCallable<T> {
    /// call trait
    fn call<'py>(self, py: Python<'py>) -> T;
}

impl<T, F: for<'py> FnOnce(Python<'py>) -> T> MaybePyCallable<T> for F {
    fn call<'py>(self, py: Python<'py>) -> T {
        self(py)
    }
}

impl<T, F: FnOnce() -> T> MaybePyCallable<T> for PyCallWrap<F> {
    fn call<'py>(self, _: Python<'py>) -> T {
        self.0()
    }
}

impl<T, F: MaybePyCallable<T>> GILLazyPy<T, F> {
    /// Forces the evaluation of this lazy value and
    /// returns a reference to the result. This is equivalent
    /// to the `Deref` impl, but is explicit.
    ///
    /// # Example
    /// ```
    /// use rust_circuit::lazy::GILLazyPy;
    ///
    /// pyo3::prepare_freethreaded_python();
    ///
    /// let lazy = GILLazyPy::new(|| 92);
    ///
    /// assert_eq!(GILLazyPy::force(&lazy), &92);
    /// assert_eq!(&*lazy, &92);
    /// ```
    pub fn force(this: &GILLazyPy<T, F>) -> &T {
        Python::with_gil(|py| {
            this.cell.get_or_init(py, || match this.init.take() {
                Some(f) => f.call(py),
                None => panic!("GILLazyPy instance has previously been poisoned"),
            })
        })
    }

    /// Forces the evaluation of this lazy value and
    /// returns a mutable reference to the result. This is equivalent
    /// to the `Deref` impl, but is explicit.
    ///
    /// # Example
    /// ```
    /// use rust_circuit::lazy::GILLazyPy;
    ///
    /// pyo3::prepare_freethreaded_python();
    ///
    /// let mut lazy = GILLazyPy::new(|| 92);
    ///
    /// assert_eq!(GILLazyPy::force_mut(&mut lazy), &mut 92);
    /// ```
    pub fn force_mut(this: &mut GILLazyPy<T, F>) -> &mut T {
        Self::force(this);
        Self::get_mut(this).unwrap_or_else(|| unreachable!())
    }

    /// Gets the reference to the result of this lazy value if
    /// it was initialized, otherwise returns `None`.
    ///
    /// # Example
    /// ```
    /// use rust_circuit::lazy::GILLazyPy;
    ///
    /// pyo3::prepare_freethreaded_python();
    ///
    /// let lazy = GILLazyPy::new(|| 92);
    ///
    /// assert_eq!(GILLazyPy::get(&lazy), None);
    /// assert_eq!(&*lazy, &92);
    /// assert_eq!(GILLazyPy::get(&lazy), Some(&92));
    /// ```
    pub fn get(this: &GILLazyPy<T, F>) -> Option<&T> {
        Python::with_gil(|py| this.cell.get(py))
    }

    /// Gets the reference to the result of this lazy value if
    /// it was initialized, otherwise returns `None`.
    ///
    /// # Example
    /// ```
    /// use rust_circuit::lazy::GILLazyPy;
    ///
    /// pyo3::prepare_freethreaded_python();
    ///
    /// let mut lazy = GILLazyPy::new(|| 92);
    ///
    /// assert_eq!(GILLazyPy::get_mut(&mut lazy), None);
    /// assert_eq!(&*lazy, &92);
    /// assert_eq!(GILLazyPy::get_mut(&mut lazy), Some(&mut 92));
    /// ```
    pub fn get_mut(this: &mut GILLazyPy<T, F>) -> Option<&mut T> {
        this.cell.get_mut()
    }
}

impl<T, F: MaybePyCallable<T>> Deref for GILLazyPy<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        GILLazyPy::force(self)
    }
}

impl<T, F: MaybePyCallable<T>> DerefMut for GILLazyPy<T, F> {
    fn deref_mut(&mut self) -> &mut T {
        GILLazyPy::force(self);
        self.cell.get_mut().unwrap_or_else(|| unreachable!())
    }
}

impl<T: Default> Default for GILLazy<T> {
    /// Creates a new lazy value using `Default` as the initializing function.
    fn default() -> GILLazy<T> {
        GILLazyPy::new(T::default)
    }
}
