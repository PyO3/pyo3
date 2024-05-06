use std::ffi::{c_int, c_void};
use std::ptr::NonNull;

use crate::ffi;

/// Represents a Python module's state.
///
/// More precisely, this `struct` resides on the per-module memory area
/// allocated during the module's creation.
#[repr(C)]
#[derive(Debug)]
pub struct ModuleState {
    inner: Option<NonNull<ModuleStateImpl>>,
}

impl ModuleState {
    pub fn new() -> Self {
        let boxed = Box::new(ModuleStateImpl::new());

        Self {
            inner: NonNull::new(Box::into_raw(boxed)),
        }
    }
}

impl Default for ModuleState {
    fn default() -> Self {
        Self::new()
    }
}

/// Inner layout of [`ModuleState`].
///
/// In order to guarantee that all resources acquired during the initialization
/// of per-module state are correctly released, this `struct` exists as the sole
/// field of [`ModuleState`] in the form of a pointer. This allows
/// [`module_state_free`] to safely [`drop`] this `struct` when [`ModuleState`]
/// is being deallocated by the Python interpreter.
#[repr(C)]
#[derive(Debug)]
struct ModuleStateImpl {}

impl ModuleStateImpl {
    fn new() -> Self {
        Self {}
    }
}

/// Called during multi-phase initialization in order to create an instance of
/// [`ModuleState`] on the memory area specific to modules.
///
/// Slot: [`Py_mod_exec`]
///
/// [`Py_mod_exec`]: https://docs.python.org/3/c-api/module.html#c.Py_mod_exec
pub unsafe extern "C" fn module_state_init(module: *mut ffi::PyObject) -> c_int {
    let state: *mut ModuleState = ffi::PyModule_GetState(module.cast()).cast();

    if state.is_null() {
        *state = ModuleState::new();
        return 0;
    }

    0
}

/// Called during GC traversal of the module object.
///
/// Used for the [`m_traverse`] field of [`PyModuleDef`].
///
/// [`m_traverse`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef.m_traverse
/// [`PyModuleDef`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef
pub unsafe extern "C" fn module_state_traverse(
    _module: *mut ffi::PyObject,
    _visit: ffi::visitproc,
    _arg: *mut c_void,
) -> c_int {
    0
}

/// Called during GC clearing of the module object.
///
/// Used for the [`m_clear`] field of [`PyModuleDef`].
///
/// [`m_clear`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef.m_clear
/// [`PyModuleDef`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef
pub unsafe extern "C" fn module_state_clear(_module: *mut ffi::PyObject) -> c_int {
    // Should any PyObjects be made part of ModuleState or ModuleStateInner,
    // these have to be Py_CLEARed here.
    // See: examples/sequential/src/module.rs
    0
}

/// Called during deallocation of the module object.
///
/// Used for the [`m_free`] field of [`PyModuleDef`].
///
/// [`m_free`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef.m_free
/// [`PyModuleDef`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef
pub unsafe extern "C" fn module_state_free(module: *mut c_void) {
    let state: *mut ModuleState = ffi::PyModule_GetState(module.cast()).cast();
    if let Some(inner) = (*state).inner {
        let ptr = inner.as_ptr();
        // SAFETY: We obtained this pointer via Box::into_raw beforehand.
        drop(unsafe { Box::from_raw(ptr) });
    }
}
