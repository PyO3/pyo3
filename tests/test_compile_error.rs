#[test]
fn test_compile_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/invalid_macro_args.rs");
    t.compile_fail("tests/ui/invalid_property_args.rs");
    t.compile_fail("tests/ui/invalid_pyclass_args.rs");
    t.compile_fail("tests/ui/missing_clone.rs");
    t.compile_fail("tests/ui/reject_generics.rs");
    t.compile_fail("tests/ui/wrong_aspyref_lifetimes.rs");
    t.compile_fail("tests/ui/static_ref.rs");
    #[cfg(not(feature = "nightly"))]
    t.compile_fail("tests/ui/invalid_pymethod_names.rs");
}
