use std::ptr::NonNull;

use crate::internal::typemap::{CloneAny, TypeMap};
use crate::types::PyModule;
use crate::{ffi, Bound};

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

    pub fn state_map_ref(&self) -> &StateMap {
        &self.inner_ref().sm
    }

    pub fn state_map_mut(&mut self) -> &mut StateMap {
        &mut self.inner_mut().sm
    }

    fn inner_ref(&self) -> &StateCapsule {
        self.inner
            .as_ref()
            .map(|ptr| unsafe { ptr.as_ref() })
            .expect("BUG: ModuleState.inner should always be Some, except when dropping")
    }

    fn inner_mut(&mut self) -> &mut StateCapsule {
        self.inner
            .as_mut()
            .map(|ptr| unsafe { ptr.as_mut() })
            .expect("BUG: ModuleState.inner should always be Some, except when dropping")
    }

    /// This is the actual [`Drop::drop`] implementation, split out
    /// so we can run it on the state ptr returned from [`Self::pymodule_get_state`]
    ///
    /// While this function does not take a owned `self`, the calling ModuleState
    /// should not be accessed again
    ///
    /// Calling this function multiple times on a single ModuleState is a noop,
    /// beyond the first
    unsafe fn drop_impl(&mut self) {
        if let Some(ptr) = self.inner.take().map(|state| state.as_ptr()) {
            // SAFETY: This ptr is allocated via Box::new in Self::new, and is
            // non null
            unsafe { drop(Box::from_raw(ptr)) }
        }
    }
}

impl ModuleState {
    /// Fetch the [`ModuleState`] from a bound PyModule, inheriting it's lifetime
    ///
    /// ## Panics
    ///
    /// This function can panic if called on a PyModule that has not yet been
    /// initialized
    pub(crate) fn from_bound<'a>(this: &'a Bound<'_, PyModule>) -> &'a Self {
        unsafe {
            Self::pymodule_get_state(this.as_ptr())
                .map(|ptr| ptr.as_ref())
                .expect("pyo3 PyModules should always have per-module state")
        }
    }

    /// Fetch the [`ModuleState`] mutably from a bound PyModule, inheriting it's
    /// lifetime
    ///
    /// ## Panics
    ///
    /// This function can panic if called on a PyModule that has not yet been
    /// initialized
    pub(crate) fn from_bound_mut<'a>(this: &'a mut Bound<'_, PyModule>) -> &'a mut Self {
        unsafe {
            Self::pymodule_get_state(this.as_ptr())
                .map(|mut ptr| ptr.as_mut())
                .expect("pyo3 PyModules should always have per-module state")
        }
    }

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
                // SAFETY: this callback is called when python is freeing the
                // associated PyModule, so we should never be accessed again
                (*state.as_ptr()).drop_impl()
            }
        }
    }
}

impl Drop for ModuleState {
    fn drop(&mut self) {
        // SAFETY: we're being dropped, so we'll never be accessed again
        unsafe { self.drop_impl() };
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
