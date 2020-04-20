#[test]
fn test_compile_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/invalid_macro_args.rs");
    t.compile_fail("tests/ui/invalid_property_args.rs");
    t.compile_fail("tests/ui/invalid_pyclass_args.rs");
    t.compile_fail("tests/ui/invalid_pymethod_names.rs");
    t.compile_fail("tests/ui/missing_clone.rs");
    t.compile_fail("tests/ui/reject_generics.rs");
    t.compile_fail("tests/ui/wrong_aspyref_lifetimes.rs");
    // Since the current minimum nightly(2020-01-20) has a different error message,
    // we skip this test.
    // TODO(kngwyu): Remove this `if` when we update minimum nightly.
    if option_env!("TRAVIS_JOB_NAME") != Some("Minimum nightly") {
        t.compile_fail("tests/ui/static_ref.rs");
    }
}
