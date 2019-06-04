#[test]
#[cfg(testkcovstopmarker)]
fn test_compile_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/reject_generics.rs");
}
