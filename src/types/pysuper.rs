use crate::ffi;
use crate::types::PyType;
use crate::{PyAny, PyResult};

/// Represents a Python `super` object.
///
/// This type is immutable.
#[repr(transparent)]
pub struct PySuper(PyAny);

pyobject_native_type_core!(PySuper, ffi::PySuper_Type);

impl PySuper {
    /// Constructs a new super object. More read about super object: [docs](https://docs.python.org/3/library/functions.html#super)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    ///#[pyclass(subclass)]
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
    ///     fn method(self_: &PyCell<Self>) -> PyResult<&PyAny> {
    ///         let super_ = self_.py_super()?;
    ///         super_.call_method("method", (), None)
    ///     }
    /// }
    /// ```
    pub fn new<'py>(ty: &'py PyType, obj: &'py PyAny) -> PyResult<&'py PySuper> {
        let py = ty.py();
        let super_ = py.get_type::<PySuper>().call1((ty, obj))?;
        let super_ = super_.downcast::<PySuper>()?;
        Ok(super_)
    }
}
