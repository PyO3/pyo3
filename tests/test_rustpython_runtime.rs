#![cfg(PyRustPython)]

use pyo3::prelude::*;

#[test]
fn backend_runtime_attach_roundtrip() {
    unsafe { pyo3::ffi::Py_FinalizeEx() };
    assert_eq!(unsafe { pyo3::ffi::Py_IsInitialized() }, 0);

    Python::initialize();
    assert_eq!(unsafe { pyo3::ffi::Py_IsInitialized() }, 1);

    Python::attach(|py| {
        let err = pyo3::exceptions::PyValueError::new_err("boom");
        err.restore(py);
        let fetched = pyo3::PyErr::fetch(py);
        assert_eq!(fetched.to_string(), "ValueError: boom");
    });

    unsafe { pyo3::ffi::Py_FinalizeEx() };
    assert_eq!(unsafe { pyo3::ffi::Py_IsInitialized() }, 0);

    Python::attach(|py| {
        let err = pyo3::exceptions::PyRuntimeError::new_err("reinitialized");
        err.restore(py);
        assert_eq!(pyo3::PyErr::fetch(py).to_string(), "RuntimeError: reinitialized");
    });
}

#[test]
#[ignore = "upstream RustPython bug: spawned-thread imports recurse in importlib (_blocking_on); see RustPython/RustPython#7586"]
fn worker_thread_can_import_array() {
    let handle = std::thread::spawn(|| {
        Python::attach(|py| {
            let module = py.import("array");
            assert!(module.is_ok(), "array import failed: {module:?}");
        });
    });

    handle.join().expect("worker thread panicked");
}

#[test]
#[ignore = "upstream RustPython bug: embedded main-thread import of re recurses in importlib; see RustPython/RustPython#7587"]
fn main_thread_can_import_re() {
    Python::attach(|py| {
        let module = py.import("re");
        assert!(module.is_ok(), "re import failed: {module:?}");
    });
}

#[test]
#[ignore = "upstream RustPython bug: warnings.filterwarnings lazily imports re and hits the same embedded import recursion; see RustPython/RustPython#7587"]
fn main_thread_warnings_filterwarnings_works() {
    Python::attach(|py| {
        let warnings = py.import("warnings").expect("import warnings");
        warnings.call_method0("resetwarnings").expect("resetwarnings");
        let cls = py.get_type::<pyo3::exceptions::PyUserWarning>();
        warnings
            .call_method1("filterwarnings", ("error", "", &cls, "pyo3test"))
            .expect("filterwarnings");
    });
}
