#![cfg(feature = "macros")]

#[rustversion::stable]
#[test]
fn test_compile_errors() {
    // stable - require all tests to pass
    _test_compile_errors()
}

#[cfg(not(feature = "nightly"))]
#[rustversion::nightly]
#[test]
fn test_compile_errors() {
    // nightly - don't care if test output is potentially wrong, to avoid churn in PyO3's CI thanks
    // to diagnostics changing on nightly.
    let _ = std::panic::catch_unwind(_test_compile_errors);
}

#[cfg(feature = "nightly")]
#[rustversion::nightly]
#[test]
fn test_compile_errors() {
    // nightly - don't care if test output is potentially wrong, to avoid churn in PyO3's CI thanks
    // to diagnostics changing on nightly.
    _test_compile_errors()
}

#[cfg(not(feature = "nightly"))]
fn _test_compile_errors() {
    let t = trybuild::TestCases::new();

    t.compile_fail("tests/ui/invalid_macro_args.rs");
    t.compile_fail("tests/ui/invalid_need_module_arg_position.rs");
    t.compile_fail("tests/ui/invalid_property_args.rs");
    t.compile_fail("tests/ui/invalid_pyclass_args.rs");
    t.compile_fail("tests/ui/invalid_pyclass_enum.rs");
    t.compile_fail("tests/ui/invalid_pyclass_item.rs");
    #[cfg(not(Py_LIMITED_API))]
    t.compile_fail("tests/ui/invalid_pymethods_buffer.rs");
    t.compile_fail("tests/ui/invalid_pymethod_names.rs");
    t.compile_fail("tests/ui/invalid_pymodule_args.rs");
    t.compile_fail("tests/ui/reject_generics.rs");
    t.compile_fail("tests/ui/invalid_pymethod_proto_args.rs");
    t.compile_fail("tests/ui/invalid_pymethod_proto_args_py.rs");

    tests_rust_1_49(&t);
    tests_rust_1_56(&t);
    tests_rust_1_57(&t);
    tests_rust_1_58(&t);
    tests_rust_1_60(&t);

    #[rustversion::since(1.49)]
    fn tests_rust_1_49(t: &trybuild::TestCases) {
        t.compile_fail("tests/ui/deprecations.rs");
    }
    #[rustversion::before(1.49)]
    fn tests_rust_1_49(_t: &trybuild::TestCases) {}

    #[rustversion::since(1.56)]
    fn tests_rust_1_56(t: &trybuild::TestCases) {
        t.compile_fail("tests/ui/invalid_closure.rs");
        t.compile_fail("tests/ui/invalid_result_conversion.rs");
        t.compile_fail("tests/ui/pyclass_send.rs");
    }

    #[rustversion::before(1.56)]
    fn tests_rust_1_56(_t: &trybuild::TestCases) {}

    #[rustversion::since(1.57)]
    fn tests_rust_1_57(t: &trybuild::TestCases) {
        t.compile_fail("tests/ui/invalid_argument_attributes.rs");
        t.compile_fail("tests/ui/invalid_frompy_derive.rs");
        t.compile_fail("tests/ui/static_ref.rs");
        t.compile_fail("tests/ui/wrong_aspyref_lifetimes.rs");
    }

    #[rustversion::before(1.57)]
    fn tests_rust_1_57(_t: &trybuild::TestCases) {}

    #[rustversion::since(1.58)]
    fn tests_rust_1_58(t: &trybuild::TestCases) {
        t.compile_fail("tests/ui/invalid_pyfunctions.rs");
        t.compile_fail("tests/ui/invalid_pymethods.rs");
        t.compile_fail("tests/ui/missing_clone.rs");
        t.compile_fail("tests/ui/not_send.rs");
        t.compile_fail("tests/ui/not_send2.rs");
        t.compile_fail("tests/ui/not_send3.rs");
        #[cfg(Py_LIMITED_API)]
        t.compile_fail("tests/ui/abi3_nativetype_inheritance.rs");
    }

    #[rustversion::before(1.58)]
    fn tests_rust_1_58(_t: &trybuild::TestCases) {}

    #[rustversion::since(1.60)]
    fn tests_rust_1_60(t: &trybuild::TestCases) {
        t.compile_fail("tests/ui/invalid_immutable_pyclass_borrow.rs");
        t.compile_fail("tests/ui/invalid_pymethod_receiver.rs");
    }

    #[rustversion::before(1.60)]
    fn tests_rust_1_60(_t: &trybuild::TestCases) {}
}

#[cfg(feature = "nightly")]
fn _test_compile_errors() {
    let t = trybuild::TestCases::new();

    t.compile_fail("tests/ui/not_send_auto_trait.rs");
    t.compile_fail("tests/ui/not_send_auto_trait2.rs");
    t.compile_fail("tests/ui/send_wrapper.rs");
}
