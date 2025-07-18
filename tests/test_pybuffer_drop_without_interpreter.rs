#![cfg(any(not(Py_LIMITED_API), Py_3_11))] // buffer availability
#![cfg(not(any(PyPy, GraalPy)))] // cannot control interpreter lifecycle in PyPy or GraalPy

//! Dropping `Py<T>` after the interpreter has been finalized should be sound.
//!
//! See e.g. https://github.com/PyO3/pyo3/issues/4632 for an extension of this problem
//! where the interpreter was finalized before `PyBuffer<T>` was dropped.
//!
//! This test runs in its own process to control the interpreter lifecycle.

use pyo3::{buffer::PyBuffer, types::PyBytes};

#[test]
fn test_pybuffer_drop_without_interpreter() {
    // SAFETY: this is knowingly unsafe as we're preserving the `Py<T>` object
    // after the Python interpreter has been finalized.
    //
    // However we should still be able to drop it without causing undefined behavior,
    // so that process shutdown is sound.
    let obj: PyBuffer<u8> = unsafe {
        pyo3::with_embedded_python_interpreter(|py| {
            PyBuffer::get(&PyBytes::new(py, b"abcdef")).unwrap()
        })
    };

    // there should be no interpreter outside of the `with_embedded_python_interpreter` block
    assert_eq!(unsafe { pyo3_ffi::Py_IsInitialized() }, 0);

    // dropping object should be sound
    drop(obj);

    // dropping object should not re-initialize the interpreter
    assert_eq!(unsafe { pyo3_ffi::Py_IsInitialized() }, 0);
}
