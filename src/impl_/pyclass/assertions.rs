/// Helper function that can be used at compile time to emit a diagnostic if
/// the type does not implement `Send` or `Sync` when it should; the mere act
/// of invoking this function will cause the diagnostic to be emitted if needed.
pub const fn assert_pyclass_send_sync<T>()
where
    T: Send + Sync,
{
}

mod tests {
    #[cfg(feature = "macros")]
    #[test]
    fn test_assert_pyclass_send_sync() {
        #[crate::pyclass(crate = "crate")]
        struct MyClass {}

        super::assert_pyclass_send_sync::<MyClass>();
    }
}
