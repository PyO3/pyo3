#![cfg(PyRustPython)]

use pyo3::prelude::*;

#[test]
fn worker_thread_can_import_array() {
    let handle = std::thread::spawn(|| {
        Python::attach(|py| {
            let module = py.import("array");
            assert!(module.is_ok(), "array import failed: {module:?}");
        });
    });

    handle.join().expect("worker thread panicked");
}
