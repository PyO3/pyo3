//! Implementation details of `#[pymodule]` which need to be accessible from proc-macro generated code.

use std::cell::UnsafeCell;
use std::os::raw::c_void;

use crate::{
    callback::panic_result_into_callback_output, ffi, types::PyModule, GILPool, IntoPyPointer, Py,
    PyObject, PyResult, Python,
};

/// `Sync` wrapper of `ffi::PyModuleDef_Slot`
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ModuleDefSlot(ffi::PyModuleDef_Slot);

unsafe impl Sync for ModuleDefSlot {}

/// `Sync` wrapper of `ffi::PyModuleDef`.
pub struct ModuleDef {
    // wrapped in UnsafeCell so that Rust compiler treats this as interior mutability
    ffi_def: UnsafeCell<ffi::PyModuleDef>,
}

unsafe impl Sync for ModuleDef {}

impl ModuleDefSlot {
    /// Make new module definition slot
    pub const fn new(slot: i32, value: *mut c_void) -> Self {
        Self(ffi::PyModuleDef_Slot { slot, value })
    }
}

impl ModuleDef {
    /// Make new module definition with given module name.
    ///
    /// # Safety
    /// `name` and `doc` must be null-terminated strings.
    pub const unsafe fn new(
        name: &'static str,
        doc: &'static str,
        slots: *mut ffi::PyModuleDef_Slot,
    ) -> Self {
        const INIT: ffi::PyModuleDef = ffi::PyModuleDef {
            m_base: ffi::PyModuleDef_HEAD_INIT,
            m_name: std::ptr::null(),
            m_doc: std::ptr::null(),
            m_size: 0,
            m_methods: std::ptr::null_mut(),
            m_slots: std::ptr::null_mut(),
            m_traverse: None,
            m_clear: None,
            m_free: None,
        };

        let ffi_def = UnsafeCell::new(ffi::PyModuleDef {
            m_name: name.as_ptr() as *const _,
            m_doc: doc.as_ptr() as *const _,
            m_slots: slots,
            ..INIT
        });

        ModuleDef { ffi_def }
    }
    /// Return module def
    pub fn module_def(&'static self) -> *mut ffi::PyModuleDef {
        self.ffi_def.get()
    }
    /// Builds a module using user given initializer. Used for [`#[pymodule]`][crate::pymodule].
    pub fn make_module(&'static self, py: Python<'_>) -> PyResult<PyObject> {
        let module = unsafe {
            Py::<PyModule>::from_owned_ptr_or_err(py, ffi::PyModule_Create(self.ffi_def.get()))?
        };
        // (self.initializer.0)(py, module.as_ref(py))?;
        Ok(module.into())
    }
    /// Implementation of `PyInit_foo` functions generated in [`#[pymodule]`][crate::pymodule]..
    ///
    /// # Safety
    /// The Python GIL must be held.
    pub unsafe fn module_init(&'static self) -> *mut ffi::PyObject {
        let pool = GILPool::new();
        let py = pool.python();
        let unwind_safe_self = std::panic::AssertUnwindSafe(self);
        panic_result_into_callback_output(
            py,
            std::panic::catch_unwind(move || -> PyResult<_> {
                #[cfg(all(PyPy, not(Py_3_8)))]
                {
                    const PYPY_GOOD_VERSION: [u8; 3] = [7, 3, 8];
                    let version = py
                        .import("sys")?
                        .getattr("implementation")?
                        .getattr("version")?;
                    if version.lt(crate::types::PyTuple::new(py, &PYPY_GOOD_VERSION))? {
                        let warn = py.import("warnings")?.getattr("warn")?;
                        warn.call1((
                            "PyPy 3.7 versions older than 7.3.8 are known to have binary \
                             compatibility issues which may cause segfaults. Please upgrade.",
                        ))?;
                    }
                }
                Ok(unwind_safe_self.make_module(py)?.into_ptr())
            }),
        )
    }
}
