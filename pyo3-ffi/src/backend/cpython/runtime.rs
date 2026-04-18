pub fn initialize() {
    unsafe { crate::pylifecycle::Py_InitializeEx(0) };
}

pub fn finalize() {
    unsafe {
        let _ = crate::pylifecycle::Py_FinalizeEx();
    }
}

pub fn is_initialized() -> bool {
    unsafe { crate::pylifecycle::Py_IsInitialized() != 0 }
}
