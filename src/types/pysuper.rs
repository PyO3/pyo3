use crate::instance::Bound;
use crate::types::any::PyAnyMethods;
use crate::types::PyType;
use crate::{ffi, PyNativeType, PyTypeInfo};
use crate::{PyAny, PyResult};

/// Represents a Python `super` object.
///
/// This type is immutable.
#[repr(transparent)]
pub struct PySuper(PyAny);

pyobject_native_type_core!(
    PySuper,
    pyobject_native_static_type_object!(ffi::PySuper_Type)
);

impl PySuper {
    /// Deprecated form of `PySuper::new_bound`.
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PySuper::new` will be replaced by `PySuper::new_bound` in a future PyO3 version"
        )
    )]
    pub fn new<'py>(ty: &'py PyType, obj: &'py PyAny) -> PyResult<&'py PySuper> {
        Self::new_bound(&ty.as_borrowed(), &obj.as_borrowed()).map(Bound::into_gil_ref)
    }

    /// Constructs a new super object. More read about super object: [docs](https://docs.python.org/3/library/functions.html#super)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass(subclass)]
    /// struct BaseClass {
    ///     val1: usize,
    /// }
    ///
    /// #[pymethods]
    /// impl BaseClass {
    ///     #[new]
    ///     fn new() -> Self {
    ///         BaseClass { val1: 10 }
    ///     }
    ///
    ///     pub fn method(&self) -> usize {
    ///         self.val1
    ///     }
    /// }
    ///
    /// #[pyclass(extends=BaseClass)]
    /// struct SubClass {}
    ///
    /// #[pymethods]
    /// impl SubClass {
    ///     #[new]
    ///     fn new() -> (Self, BaseClass) {
    ///         (SubClass {}, BaseClass::new())
    ///     }
    ///
    ///     fn method<'py>(self_: &Bound<'py, Self>) -> PyResult<Bound<'py, PyAny>> {
    ///         let super_ = self_.py_super()?;
    ///         super_.call_method("method", (), None)
    ///     }
    /// }
    /// ```
    pub fn new_bound<'py>(
        ty: &Bound<'py, PyType>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PySuper>> {
        PySuper::type_object_bound(ty.py())
            .call1((ty, obj))
            .map(|any| {
                // Safety: super() always returns instance of super
                unsafe { any.downcast_into_unchecked() }
            })
    }
}
