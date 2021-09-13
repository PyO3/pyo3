//! Contains additional fields for `#[pyclass]`.
//!
//! Mainly used by PyO3's proc-macro code.
use crate::{ffi, Python};

/// Represents the `__dict__` field for `#[pyclass]`.
pub trait PyClassDict {
    /// Whether this `__dict__` field is capable of holding a dictionary.
    const IS_DUMMY: bool = true;
    /// Initializes a [PyObject](crate::ffi::PyObject) `__dict__` reference.
    fn new() -> Self;
    /// Empties the dictionary of its key-value pairs.
    #[inline]
    fn clear_dict(&mut self, _py: Python) {}
    private_decl! {}
}

/// Represents the `__weakref__` field for `#[pyclass]`.
pub trait PyClassWeakRef {
    /// Whether this `weakref` type is capable of holding weak references.
    const IS_DUMMY: bool = true;
    /// Initializes a `weakref` instance.
    fn new() -> Self;
    /// Clears the weak references to the given object.
    ///
    /// # Safety
    /// - `_obj` must be a pointer to the pyclass instance which contains `self`.
    /// - The GIL must be held.
    #[inline]
    unsafe fn clear_weakrefs(&mut self, _obj: *mut ffi::PyObject, _py: Python) {}
    private_decl! {}
}

/// Zero-sized dummy field.
pub struct PyClassDummySlot;

impl PyClassDict for PyClassDummySlot {
    private_impl! {}
    #[inline]
    fn new() -> Self {
        PyClassDummySlot
    }
}

impl PyClassWeakRef for PyClassDummySlot {
    private_impl! {}
    #[inline]
    fn new() -> Self {
        PyClassDummySlot
    }
}

/// Actual dict field, which holds the pointer to `__dict__`.
///
/// `#[pyclass(dict)]` automatically adds this.
#[repr(transparent)]
pub struct PyClassDictSlot(*mut ffi::PyObject);

impl PyClassDict for PyClassDictSlot {
    private_impl! {}
    const IS_DUMMY: bool = false;
    #[inline]
    fn new() -> Self {
        Self(std::ptr::null_mut())
    }
    #[inline]
    fn clear_dict(&mut self, _py: Python) {
        if !self.0.is_null() {
            unsafe { ffi::PyDict_Clear(self.0) }
        }
    }
}

/// Actual weakref field, which holds the pointer to `__weakref__`.
///
/// `#[pyclass(weakref)]` automatically adds this.
#[repr(transparent)]
pub struct PyClassWeakRefSlot(*mut ffi::PyObject);

impl PyClassWeakRef for PyClassWeakRefSlot {
    private_impl! {}
    const IS_DUMMY: bool = false;
    #[inline]
    fn new() -> Self {
        Self(std::ptr::null_mut())
    }
    #[inline]
    unsafe fn clear_weakrefs(&mut self, obj: *mut ffi::PyObject, _py: Python) {
        if !self.0.is_null() {
            ffi::PyObject_ClearWeakRefs(obj)
        }
    }
}
