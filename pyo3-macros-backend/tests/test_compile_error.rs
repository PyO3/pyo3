#[cfg(not(target_arch = "wasm32"))] // Not possible to invoke compiler from wasm
#[test]
fn test_compile_errors() {
    let t = trybuild::TestCases::new();

    t.compile_fail("tests/test_multiple_errors.rs");
}
