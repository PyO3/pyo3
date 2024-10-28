//! A trait for extending a collection with elements from an iterator, returning an error if the operation fails.
use crate::PyResult;

/// A trait for extending a collection with elements from an iterator, returning an error if the operation fails.
/// This trait is similar to the standard library's `Extend` trait, but it returns a `PyResult` instead of panicking.
pub trait TryExtend<I, T>
where
    I: IntoIterator<Item = T>,
{
    /// Extends a collection with elements from an iterator, returning an error if the operation fails.
    fn try_extend(&mut self, iter: I) -> PyResult<()>;
}
