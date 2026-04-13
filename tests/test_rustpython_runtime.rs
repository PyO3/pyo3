#![cfg(PyRustPython)]

use pyo3::prelude::*;

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
