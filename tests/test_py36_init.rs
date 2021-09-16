// This test checks Python initialization on python 3.6, so needs to be standalone in its own process.

#[cfg(not(PyPy))]
#[test]
fn test_py36_init_threads() {
    unsafe { pyo3::ffi::Py_InitializeEx(0) };
    pyo3::prepare_freethreaded_python();
    assert_eq!(unsafe { pyo3::ffi::PyEval_ThreadsInitialized() }, 1);
}
