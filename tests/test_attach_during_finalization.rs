#![cfg(all(Py_3_13, not(any(PyPy, GraalPy)), not(target_arch = "wasm32")))]

use pyo3::Python;

#[test]
fn test_attach_panics_during_finalization() {
    let worker = std::thread::spawn(|| {
        // SAFETY: both state queries are safe while detached.
        while unsafe { pyo3::ffi::Py_IsFinalizing() } == 0
            || unsafe { pyo3::ffi::Py_IsInitialized() } != 0
        {
            std::thread::yield_now();
        }

        assert!(std::panic::catch_unwind(|| Python::attach(|_| ())).is_err());
    });

    // SAFETY: this test runs in its own process and initializes Python only once.
    unsafe { pyo3::with_embedded_python_interpreter(|_| ()) };
    worker.join().unwrap();

    // SAFETY: Py_IsInitialized is always safe to call.
    assert_eq!(unsafe { pyo3::ffi::Py_IsInitialized() }, 0);
}
