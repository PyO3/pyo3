/// Helper function that can be used at compile time to emit a diagnostic if
/// the type does not implement `Sync` when it should.
///
/// The mere act of invoking this function will cause the diagnostic to be
/// emitted if `T` does not implement `Sync` when it should.
///
/// The additional `const IS_SYNC: bool` parameter is used to allow the custom
/// diagnostic to be emitted; if `PyClassSync`
pub const fn assert_pyclass_sync<T, const IS_SYNC: bool>()
where
    T: PyClassSync<IS_SYNC> + Sync,
{
}

#[cfg_attr(
    diagnostic_namespace,
    diagnostic::on_unimplemented(
        message = "the trait `Sync` is not implemented for `{Self}`",
        label = "needs to implement `Sync` to be `#[pyclass]`",
        note = "to opt-out of threading support, use `#[pyclass(unsendable)]`",
        note = "see <TODO INSERT PYO3 GUIDE> for more information",
    )
)]
pub trait PyClassSync<const IS_SYNC: bool>: private::Sealed<IS_SYNC> {}

mod private {
    pub trait Sealed<const IS_SYNC: bool> {}
    impl<T> Sealed<true> for T {}
    #[cfg(not(diagnostic_namespace))]
    impl<T> Sealed<false> for T {}
}

// If `true` is passed for the const parameter, then the diagnostic will
// not be emitted.
impl<T> PyClassSync<true> for T {}

// Without `diagnostic_namespace`, the trait bound is not useful, so we add
// an implementation for `false`` to avoid a useless diagnostic.
#[cfg(not(diagnostic_namespace))]
impl<T> PyClassSync<false> for T {}

mod tests {
    use super::assert_pyclass_sync;

    #[test]
    fn test_assert_pyclass_sync() {
        #[crate::pyclass(crate = "crate")]
        struct IntWrapper {}
        assert_pyclass_sync::<IntWrapper, true>();
    }
}
