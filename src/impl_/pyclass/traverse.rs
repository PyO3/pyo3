use crate::{
    gc::{PyTraverseError, PyVisit},
    impl_::pyclass::PyClassImplCollector,
};

/// Autoref specialization point for `__traverse__` implementations.
///
/// Type parameter `T` is necessary to make the trait local to the implementing crate.
pub trait PyClassTraverse<T> {
    /// `self` is used for autoref specialization of `PyClassImplCollector` and `&PyClassImplCollector`.
    /// `this` is the instance of the class being traversed.
    fn __traverse__(self, this: &T, visit: PyVisit<'_>) -> Result<(), PyTraverseError>;
}

/// Fallback implementation for types without a `__traverse__` pymethod.
impl<T> PyClassTraverse<T> for &'_ PyClassImplCollector<T> {
    fn __traverse__(self, _this: &T, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        Ok(())
    }
}
