#[test]
fn cpython_backend_smoke() {
    let _ = crate::backend::cpython::CpythonBackend;
}

#[cfg(feature = "runtime-cpython")]
#[test]
fn active_backend_defaults_to_cpython_family() {
    assert_eq!(crate::active_backend_kind(), crate::backend::BackendKind::Cpython);
}
