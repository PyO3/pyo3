//! Implementation details of `#[pymodule]` which need to be accessible from proc-macro generated code.

use std::cell::UnsafeCell;
use std::ffi::CStr;
use std::os::raw::c_void;

use crate::{ffi, types::PyModule, AsPyPointer, Py, PyErr, PyObject, PyResult, Python};

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
        let module_spec_type = py.import("importlib.machinery")?.getattr("ModuleSpec")?;
        let module = unsafe {
            let mod_def = self.ffi_def.get();
            // Mock a ModuleSpec object just good enough for PyModule_FromDefAndSpec()
            // an object with just a name attribute.
            let module_name = CStr::from_ptr((*mod_def).m_name);
            let args = (module_name.to_str()?, py.None());
            let spec = module_spec_type.call1(args)?;
            let module = Py::<PyModule>::from_owned_ptr_or_err(
                py,
                ffi::PyModule_FromDefAndSpec(mod_def, spec.as_ptr()),
            )?;
            if ffi::PyModule_ExecDef(module.as_ptr(), mod_def) != 0 {
                return Err(PyErr::fetch(py));
            }
            module
        };
        Ok(module.into())
    }
}
