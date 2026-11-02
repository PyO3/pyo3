#![cfg(all(Py_3_13, not(any(PyPy, GraalPy)), not(target_arch = "wasm32")))]

use pyo3::Python;

#[test]
fn test_attach_panics_during_finalization() {
    let worker = std::thread::spawn(|| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            // SAFETY: both state queries are safe while detached.
            if unsafe { pyo3::ffi::Py_IsFinalizing() } != 0
                && unsafe { pyo3::ffi::Py_IsInitialized() } == 0
            {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "CPython did not reach the expected finalization state"
            );
            std::thread::yield_now();
        }

        assert!(Python::try_attach(|_| ()).is_none());
        assert!(std::panic::catch_unwind(|| Python::attach(|_| ())).is_err());
    });

    // SAFETY: this test runs in its own process and initializes Python only once.
    unsafe { pyo3::with_embedded_python_interpreter(|_| ()) };
    worker.join().unwrap();

    // SAFETY: Py_IsInitialized is always safe to call.
    assert_eq!(unsafe { pyo3::ffi::Py_IsInitialized() }, 0);
}
