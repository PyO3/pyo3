#[test]
fn cpython_backend_smoke() {
    let _ = crate::backend::cpython::CpythonBackend;
}

#[cfg(feature = "runtime-cpython")]
#[test]
fn active_backend_defaults_to_cpython_family() {
    assert_eq!(crate::active_backend_kind(), crate::backend::BackendKind::Cpython);
}

#[cfg(all(feature = "runtime-rustpython", PyRustPython))]
#[test]
fn active_backend_switches_to_rustpython() {
    assert_eq!(crate::active_backend_kind(), crate::backend::BackendKind::Rustpython);
}
