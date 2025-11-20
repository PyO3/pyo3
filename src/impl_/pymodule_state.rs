use std::ptr::NonNull;

use crate::internal::typemap::{CloneAny, TypeMap};
use crate::ffi;

/// The internal typemap for [`ModuleState`]
pub type StateMap = TypeMap<dyn CloneAny + Send>;

/// A marker trait for indicating what type level guarantees (and requirements)
/// are made for PyO3 `PyModule` state types.
///
/// In general, a type *must be*
///
/// 1. Fully owned (`'static`)
/// 2. Cloneable (`Clone`)
/// 3. Sendable (`Send`)
///
/// To qualify as `PyModule` state.
///
/// This type is automatically implemented for all types that qualify, so no
/// further action is required.
pub trait ModuleStateType: Clone + Send {}
impl<T: Clone + Send> ModuleStateType for T {}

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
#[derive(Debug, Clone)]
struct StateCapsule {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_assertions() {
        fn is_send<T: Send>(_t: &T) {}
        fn is_clone<T: Clone>(_t: &T) {}

        let this = StateCapsule::new();
        is_send(&this);
        is_clone(&this);
    }
}
