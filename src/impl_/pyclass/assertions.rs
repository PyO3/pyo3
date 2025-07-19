/// Helper function that can be used at compile time to emit a diagnostic if
/// the type does not implement `Sync` when it should.
///
/// The mere act of invoking this function will cause the diagnostic to be
/// emitted if `T` does not implement `Sync` when it should.
///
/// The additional `const IS_SYNC: bool` parameter is used to allow the custom
/// diagnostic to be emitted; if `PyClassSync`
#[allow(unused)]
pub const fn assert_pyclass_sync<T>()
where
    T: PyClassSync + Sync,
{
}

#[cfg_attr(
    diagnostic_namespace,
    diagnostic::on_unimplemented(
        message = "the trait `Sync` is not implemented for `{Self}`",
        label = "required by `#[pyclass]`",
        note = "replace thread-unsafe fields with thread-safe alternatives",
        note = "see <TODO INSERT PYO3 GUIDE> for more information",
    )
)]
pub trait PyClassSync<T: Sync = Self> {}

impl<T> PyClassSync for T where T: Sync {}

mod tests {
    #[cfg(feature = "macros")]
    #[test]
    fn test_assert_pyclass_sync() {
        use super::assert_pyclass_sync;

        #[crate::pyclass(crate = "crate")]
        struct MyClass {}

        assert_pyclass_sync::<MyClass>();
    }
}
