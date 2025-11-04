use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::ffi;

/// Represents a Python module's state.
///
/// More precisely, this `struct` resides on the per-module memory area
/// allocated during the module's creation.
#[repr(C)]
#[derive(Debug)]
pub struct ModuleState {
    inner: Option<NonNull<StateCapsule>>,
}

impl ModuleState {
    /// Create a new, empty [`ModuleState`]
    pub fn new() -> Self {
        let boxed = Box::new(StateCapsule::new());

        Self {
            inner: NonNull::new(Box::into_raw(boxed)),
        }
    }

    /// Returns the byte size of this type, for use in `ffi::PyModuleDef.m_size`
    pub const fn size_of() -> ffi::Py_ssize_t {
        std::mem::size_of::<Self>() as ffi::Py_ssize_t
    }

    /// This is the actual [`Drop::drop`] implementation, split out
    /// so we can run it on the state ptr returned from [`Self::pymodule_get_state`]
    ///
    /// While this function does not take a owned `self`, the calling ModuleState
    /// should not be accessed again
    ///
    /// Calling this function multiple times on a single ModuleState is a noop,
    /// beyond the first
    fn drop_impl(&mut self) {
        if let Some(ptr) = self.inner.take().map(|state| state.as_ptr()) {
            // SAFETY: This ptr is allocated via Box::new in Self::new, and is
            // non null
            unsafe { drop(Box::from_raw(ptr)) }
        }
    }
}

impl ModuleState {
    /// Associated low level function for retrieving a pyo3 `pymodule`'s state
    ///
    /// If this function returns None, it means the underlying C PyModule does
    /// not have module state.
    ///
    /// This function should only be called on a PyModule that is already
    /// initialized via PyModule_New (or Py_mod_create)
    pub(crate) unsafe fn pymodule_get_state(module: *mut ffi::PyObject) -> Option<NonNull<Self>> {
        unsafe {
            let state: *mut ModuleState = ffi::PyModule_GetState(module).cast();

            match state.is_null() {
                true => None,
                false => Some(NonNull::new_unchecked(state)),
            }
        }
    }

    /// Associated low level function for freeing our `pymodule`'s state
    /// via a ModuleDef's m_free C callback
    pub(crate) unsafe fn pymodule_free_state(module: *mut ffi::PyObject) {
        unsafe {
            if let Some(state) = Self::pymodule_get_state(module) {
                (*state.as_ptr()).drop_impl()
            }
        }
    }
}

impl Drop for ModuleState {
    fn drop(&mut self) {
        self.drop_impl();
    }
}

impl Default for ModuleState {
    fn default() -> Self {
        Self::new()
    }
}

/// Inner layout of [`ModuleState`].
#[derive(Debug)]
struct StateCapsule {
    #[allow(dead_code)]
    sm: StateMap,
}

impl StateCapsule {
    fn new() -> Self {
        Self {
            sm: StateMap::new(),
        }
    }
}

impl Default for StateCapsule {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder for the actual TypeMap implementation
#[derive(Debug, Clone)]
struct StateMap {
    /// The actual typemap is !Sync + Send, so emulate this
    _mkr: PhantomData<*const ()>,
}

impl StateMap {
    pub fn new() -> Self {
        Self {
            _mkr: Default::default(),
        }
    }
}

unsafe impl Send for StateMap {}
