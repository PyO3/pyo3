/// Helper function that can be used at compile time to emit a diagnostic if
/// the type does not implement `Send` or `Sync` when it should; the mere act
/// of invoking this function will cause the diagnostic to be emitted if needed.
pub const fn assert_pyclass_send_sync<T>()
where
    T: Send + Sync,
{
}

pub const DICT_SUPPORTED: bool = cfg!(any(not(Py_LIMITED_API), Py_3_9));
pub const DICT_UNSUPPORTED_ERROR: &str =
    "`dict` requires Python >= 3.9 when using the `abi3` feature";

pub const WEAKREF_SUPPORTED: bool = cfg!(any(not(Py_LIMITED_API), Py_3_9));
pub const WEAKREF_UNSUPPORTED_ERROR: &str =
    "`weakref` requires Python >= 3.9 when using the `abi3` feature";

pub const IMMUTABLE_TYPE_SUPPORTED: bool = cfg!(any(all(Py_3_10, not(Py_LIMITED_API)), Py_3_14));
pub const IMMUTABLE_TYPE_UNSUPPORTED_ERROR: &str =
    "`immutable_type` requires Python >= 3.10 or >= 3.14 (ABI3)";

mod tests {
    #[cfg(feature = "macros")]
    #[test]
    fn test_assert_pyclass_send_sync() {
        #[crate::pyclass(crate = "crate")]
        struct MyClass {}

        super::assert_pyclass_send_sync::<MyClass>();
    }
}
