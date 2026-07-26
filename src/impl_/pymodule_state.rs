use crate::types::PyModule;
use crate::{ffi, Bound};
use alloc::boxed::Box;
use core::ptr::NonNull;

/// Represents a Python module's state.
///
/// More precisely, this `struct` resides on the per-module memory area
/// allocated during the module's creation.
#[repr(C)]
#[derive(Debug)]
pub struct ModuleState {
    inner: Option<Box<dyn std::any::Any>>,
}

impl ModuleState {
    /// Create a new, empty [`ModuleState`]
    pub fn new<T: 'static>(value: T) -> Self {
        Self {
            inner: Some(Box::new(value)),
        }
    }

    /// Retrieve immutable reference to state
    pub fn inner_ref<T: 'static>(&self) -> Option<&T> {
        self.inner.as_ref().and_then(|s| s.downcast_ref::<T>())
    }

    /// Retrieve mutable reference to state
    pub fn inner_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.as_mut().and_then(|s| s.downcast_mut::<T>())
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
        self.inner.take();
    }
}

impl ModuleState {
    /// Fetch the [`ModuleState`] from a bound PyModule, inheriting it's lifetime
    ///
    /// ## Panics
    ///
    /// This function can panic if called on a PyModule that has not yet been
    /// initialized
    pub(crate) fn from_bound<'a>(this: &'a Bound<'_, PyModule>) -> Option<&'a Self> {
        unsafe { Self::pymodule_get_state(this.as_ptr()).map(|ptr| ptr.as_ref()) }
    }

    /// Fetch the [`ModuleState`] mutably from a bound PyModule, inheriting it's
    /// lifetime
    ///
    /// ## Panics
    ///
    /// This function can panic if called on a PyModule that has not yet been
    /// initialized
    pub(crate) fn from_bound_mut<'a>(this: &'a mut Bound<'_, PyModule>) -> Option<&'a mut Self> {
        unsafe { Self::pymodule_get_state(this.as_ptr()).map(|mut ptr| ptr.as_mut()) }
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
