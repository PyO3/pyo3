#[rustversion::stable]
#[test]
fn test_compile_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/invalid_frompy_derive.rs");
    t.compile_fail("tests/ui/invalid_macro_args.rs");
    t.compile_fail("tests/ui/invalid_need_module_arg_position.rs");
    t.compile_fail("tests/ui/invalid_property_args.rs");
    t.compile_fail("tests/ui/invalid_pyclass_args.rs");
    t.compile_fail("tests/ui/invalid_pymethod_names.rs");
    t.compile_fail("tests/ui/reject_generics.rs");
    t.compile_fail("tests/ui/wrong_aspyref_lifetimes.rs");

    tests_rust_1_43(&t);
    tests_rust_1_46(&t);

    #[rustversion::since(1.43)]
    fn tests_rust_1_43(t: &trybuild::TestCases) {
        t.compile_fail("tests/ui/static_ref.rs");
    }
    #[rustversion::before(1.43)]
    fn tests_rust_1_43(_t: &trybuild::TestCases) {}

    #[rustversion::since(1.46)]
    fn tests_rust_1_46(t: &trybuild::TestCases) {
        t.compile_fail("tests/ui/invalid_pymethod_receiver.rs");
        t.compile_fail("tests/ui/invalid_result_conversion.rs");
        t.compile_fail("tests/ui/missing_clone.rs");
        #[cfg(Py_LIMITED_API)]
        t.compile_fail("tests/ui/abi3_nativetype_inheritance.rs");
    }
    #[rustversion::before(1.46)]
    fn tests_rust_1_46(_t: &trybuild::TestCases) {}
}
