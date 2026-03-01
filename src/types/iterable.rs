#[cfg(feature = "experimental-inspect")]
use crate::inspect::{type_hint_identifier, PyStaticExpr};
use crate::instance::Bound;
use crate::sync::PyOnceLock;
use crate::type_object::PyTypeInfo;
use crate::types::any::PyAnyMethods;
use crate::types::typeobject::PyTypeMethods;
use crate::types::{PyAny, PyIterator, PyType};
use crate::{ffi, Py, PyResult, Python};

/// A Python object that implements the `collections.abc.Iterable` protocol.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyIterable>`][crate::Py] or [`Bound<'py, PyIterable>`][Bound].
///
/// # Examples
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3::types::PyIterable;
///
/// # fn main() -> PyResult<()> {
/// Python::attach(|py| -> PyResult<()> {
///     let list = py.eval(c"[1, 2, 3, 4]", None, None)?;
///     let iterable = list.cast::<PyIterable>()?;
///     let numbers: PyResult<Vec<usize>> = iterable
///         .try_iter()?
///         .map(|i| i.and_then(|i| i.extract::<usize>()))
///         .collect();
///     let sum: usize = numbers?.iter().sum();
///     assert_eq!(sum, 10);
///     Ok(())
/// })
/// # }
/// ```
#[repr(transparent)]
pub struct PyIterable(PyAny);

pyobject_native_type_named!(PyIterable);

unsafe impl PyTypeInfo for PyIterable {
    const NAME: &'static str = "Iterable";
    const MODULE: Option<&'static str> = Some("collections.abc");

    #[cfg(feature = "experimental-inspect")]
    const TYPE_HINT: PyStaticExpr = type_hint_identifier!("collections.abc", "Iterable");

    #[inline]
    fn type_object_raw(py: Python<'_>) -> *mut ffi::PyTypeObject {
        static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        TYPE.import(py, "collections.abc", "Iterable")
            .unwrap()
            .as_type_ptr()
    }

    #[inline]
    fn is_type_of(object: &Bound<'_, PyAny>) -> bool {
        // Every iterator is also an iterable, fast-path it
        PyIterator::is_type_of(object)
            || object
                .is_instance(&Self::type_object(object.py()).into_any())
                .unwrap_or_else(|err| {
                    err.write_unraisable(object.py(), Some(object));
                    false
                })
    }
}

impl PyIterable {
    /// Register a pyclass as a subclass of `collections.abc.Iterable` (from the Python standard
    /// library). This is equivalent to `collections.abc.Iterable.register(T)` in Python.
    /// This registration is required for a pyclass to be castable from `PyAny` to `PyIterable`.
    pub fn register<T: PyTypeInfo>(py: Python<'_>) -> PyResult<()> {
        let ty = T::type_object(py);
        Self::type_object(py).call_method1("register", (ty,))?;
        Ok(())
    }
}

/// Implementation of functionality for [`PyIterable`].
///
/// These methods are defined for the `Bound<'py, PyIterable>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyIterable")]
pub trait PyIterableMethods<'py>: crate::sealed::Sealed {
    /// Returns an iterator over the contents of this iterable.
    ///
    /// This is equivalent to calling `iter(obj)` in Python, or
    /// [`PyIterator::from_object`].
    fn try_iter(&self) -> PyResult<Bound<'py, PyIterator>>;
}

impl<'py> PyIterableMethods<'py> for Bound<'py, PyIterable> {
    fn try_iter(&self) -> PyResult<Bound<'py, PyIterator>> {
        self.as_any().try_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::PyIterable;
    use crate::types::{PyAnyMethods, PyList};
    use crate::{IntoPyObject, PyTypeInfo, Python};

    #[test]
    fn list_is_iterable() {
        Python::attach(|py| {
            let list = vec![1_i32, 2, 3].into_pyobject(py).unwrap();
            assert!(list.cast::<PyIterable>().is_ok());
        });
    }

    #[test]
    fn int_not_iterable() {
        Python::attach(|py| {
            let x = 5i32.into_pyobject(py).unwrap();
            assert!(x.cast::<PyIterable>().is_err());
        });
    }

    #[test]
    fn try_iter_from_iterable() {
        Python::attach(|py| {
            let list = vec![10_i32, 20, 30].into_pyobject(py).unwrap();
            let iterable = list.cast::<PyIterable>().unwrap();
            use super::PyIterableMethods;
            let sum: i32 = iterable
                .try_iter()
                .unwrap()
                .map(|x| x.unwrap().extract::<i32>().unwrap())
                .sum();
            assert_eq!(sum, 60);
        });
    }

    #[test]
    fn iterator_is_iterable() {
        Python::attach(|py| {
            let list = PyList::new(py, [1_i32, 2, 3]).unwrap();
            let iter = list.try_iter().unwrap();
            assert!(iter.cast::<PyIterable>().is_ok());
        });
    }

    #[test]
    fn test_type_object() {
        Python::attach(|py| {
            let abc = PyIterable::type_object(py);
            let list = py.eval(c"[1, 2, 3]", None, None).unwrap();
            assert!(list.is_instance(&abc).unwrap());
        });
    }
}
