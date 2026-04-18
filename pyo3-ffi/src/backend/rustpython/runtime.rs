pub fn initialize() {
    crate::rustpython_runtime::initialize();
}

pub fn finalize() {
    crate::rustpython_runtime::finalize();
}

pub fn is_initialized() -> bool {
    crate::rustpython_runtime::is_initialized()
}
