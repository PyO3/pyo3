/// Helper function that can be used at compile time to emit a diagnostic if
/// the type does not implement `Send` or `Sync` when it should; the mere act
/// of invoking this function will cause the diagnostic to be emitted if needed.
pub const fn assert_pyclass_send_sync<T>()
where
    T: Send + Sync,
{
}

#[track_caller]
#[allow(clippy::assertions_on_constants, reason = "invoked by a proc macro")]
pub const fn assert_dict_supported() {
    assert!(
        cfg!(any(not(Py_LIMITED_API), Py_3_9)),
        "`dict` requires Python >= 3.9 when using the `abi3` feature"
    );
}

#[track_caller]
#[allow(clippy::assertions_on_constants, reason = "invoked by a proc macro")]
pub const fn assert_weakref_supported() {
    assert!(
        cfg!(any(not(Py_LIMITED_API), Py_3_9)),
        "`weakref` requires Python >= 3.9 when using the `abi3` feature"
    )
}

#[track_caller]
#[allow(clippy::assertions_on_constants, reason = "invoked by a proc macro")]
pub const fn assert_immutable_type_supported() {
    assert!(
        cfg!(any(all(Py_3_10, not(Py_LIMITED_API)), Py_3_14)),
        "`immutable_type` requires Python >= 3.10 (or >= 3.14 when using the `abi3` feature)"
    );
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
