use crate::instance::Bound;
use crate::types::any::PyAnyMethods;
use crate::types::PyType;
use crate::{ffi, PyTypeInfo};
use crate::{PyAny, PyResult};

/// Represents a Python `super` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PySuper>`][crate::Py] or [`Bound<'py, PySuper>`][Bound].
#[repr(transparent)]
pub struct PySuper(PyAny);

pyobject_native_type_core!(
    PySuper,
    pyobject_native_static_type_object!(ffi::PySuper_Type)
);

impl PySuper {
    /// Constructs a new super object. More read about super object: [docs](https://docs.python.org/3/library/functions.html#super)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
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
    pub fn new<'py>(
        ty: &Bound<'py, PyType>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PySuper>> {
        PySuper::type_object(ty.py()).call1((ty, obj)).map(|any| {
            // Safety: super() always returns instance of super
            unsafe { any.cast_into_unchecked() }
        })
    }
}
