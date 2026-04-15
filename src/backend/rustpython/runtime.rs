pub(crate) use pyo3_ffi::backend::current::runtime::{finalize, initialize, is_initialized};

pub(crate) fn initialize_embedded() {
    initialize();
}

pub(crate) fn finalize_embedded() {
    finalize();
}

pub(crate) fn wait_for_initialization() {
    // RustPython initialization is modeled as a single process-global runtime setup.
    // Once `initialize()` returns, there is no separate partially-initialized phase to wait for.
}

pub(crate) fn prepare_embedded_python_main_thread(_: crate::Python<'_>) {}

pub(crate) fn ensure_initialized_or_panic() {
    assert!(
        is_initialized(),
        "The Python interpreter is not initialized and the `auto-initialize` \
                feature is not enabled.\n\n\
                Consider calling `Python::initialize()` before attempting \
                to use Python APIs."
    );
}
