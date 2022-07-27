use std::{
    ffi::CString,
    sync::atomic::{AtomicBool, Ordering},
};

use pyo3::{prelude::*, types::PyCapsule};

#[pyfunction]
fn issue_219() {
    // issue 219: acquiring GIL inside #[pyfunction] deadlocks.
    let gil = Python::acquire_gil();
    let _py = gil.python();
}

#[pyfunction]
fn capsule_send_destructor(py: Python<'_>) {
    // safety defence - due to lack of send bound in PyO3 0.16, the PyCapsule type
    // must not execute destructors in different thread
    // (and will emit a Python warning)
    let destructor_called = AtomicBool::new(false);

    let cap: PyObject = {
        // so that intermediate capsule references are cleared, use a pool
        let _pool = unsafe { pyo3::GILPool::new() };
        PyCapsule::new_with_destructor(py, 0i32, &CString::new("some name").unwrap(), |_, _| {
            destructor_called.store(true, Ordering::SeqCst)
        })
        .unwrap()
        .into()
    };

    py.allow_threads(|| {
        std::thread::spawn(move || Python::with_gil(|_| drop(cap)))
            .join()
            .unwrap();
    });

    assert!(!destructor_called.load(Ordering::SeqCst));
}

#[pymodule]
pub fn misc(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(issue_219, m)?)?;
    m.add_function(wrap_pyfunction!(capsule_send_destructor, m)?)?;
    Ok(())
}
