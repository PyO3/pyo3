#[test]
fn test_compile_errors() {
    let t = trybuild::TestCases::new();
    testcase_common(&t);
    testcase_latest_stable(&t);

    fn testcase_common(t: &trybuild::TestCases) {
        t.compile_fail("tests/ui/invalid_macro_args.rs");
        t.compile_fail("tests/ui/invalid_property_args.rs");
        t.compile_fail("tests/ui/invalid_pyclass_args.rs");
        t.compile_fail("tests/ui/missing_clone.rs");
        t.compile_fail("tests/ui/reject_generics.rs");
        t.compile_fail("tests/ui/wrong_aspyref_lifetimes.rs");
        t.compile_fail("tests/ui/invalid_pymethod_names.rs");
    }
    #[rustversion::since(1.43)]
    fn testcase_latest_stable(t: &trybuild::TestCases) {
        t.compile_fail("tests/ui/static_ref.rs");
    }
    #[rustversion::before(1.43)]
    fn testcase_latest_stable(_t: &trybuild::TestCases) {}
}
