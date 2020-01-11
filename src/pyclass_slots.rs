//! This module contains additional fields for `#[pyclass]`..
//! Mainly used by our proc-macro codes.
use crate::{ffi, Python};

const POINTER_SIZE: isize = std::mem::size_of::<*mut ffi::PyObject>() as _;

/// Represents `__dict__` field for `#[pyclass]`.
pub trait PyClassDict {
    const OFFSET: Option<isize> = None;
    fn new() -> Self;
    unsafe fn clear_dict(&mut self, _py: Python) {}
    private_decl! {}
}

/// Represents `__weakref__` field for `#[pyclass]`.
pub trait PyClassWeakRef {
    const OFFSET: Option<isize> = None;
    fn new() -> Self;
    unsafe fn clear_weakrefs(&mut self, _obj: *mut ffi::PyObject, _py: Python) {}
    private_decl! {}
}

/// Zero-sized dummy field.
pub struct PyClassDummySlot;

impl PyClassDict for PyClassDummySlot {
    private_impl! {}
    fn new() -> Self {
        PyClassDummySlot
    }
}

impl PyClassWeakRef for PyClassDummySlot {
    private_impl! {}
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
    const OFFSET: Option<isize> = Some(-POINTER_SIZE);
    fn new() -> Self {
        Self(std::ptr::null_mut())
    }
    unsafe fn clear_dict(&mut self, _py: Python) {
        if !self.0.is_null() {
            ffi::PyDict_Clear(self.0)
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
    const OFFSET: Option<isize> = Some(-POINTER_SIZE);
    fn new() -> Self {
        Self(std::ptr::null_mut())
    }
    unsafe fn clear_weakrefs(&mut self, obj: *mut ffi::PyObject, _py: Python) {
        if !self.0.is_null() {
            ffi::PyObject_ClearWeakRefs(obj)
        }
    }
}
