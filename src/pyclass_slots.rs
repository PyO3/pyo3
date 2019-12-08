//! This module contains additional fields pf pyclass
// TODO(kngwyu): Add vectorcall support
use crate::{ffi, Python};

const POINTER_SIZE: isize = std::mem::size_of::<*mut ffi::PyObject>() as _;

/// Represents `__dict__`.
pub trait PyClassDict {
    const OFFSET: Option<isize> = None;
    fn new() -> Self;
    fn clear_dict(&mut self, _py: Python) {}
    private_decl! {}
}

/// Represents `__weakref__`.
pub trait PyClassWeakRef {
    const OFFSET: Option<isize> = None;
    fn new() -> Self;
    fn clear_weakrefs(&mut self, _obj: *mut ffi::PyObject, _py: Python) {}
    private_decl! {}
}

/// Dummy slot means the function doesn't has such a feature.
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

/// actural dict field
#[repr(transparent)]
pub struct PyClassDictSlot(*mut ffi::PyObject);

impl PyClassDict for PyClassDictSlot {
    private_impl! {}
    const OFFSET: Option<isize> = Some(-POINTER_SIZE);
    fn new() -> Self {
        Self(std::ptr::null_mut())
    }
    fn clear_dict(&mut self, _py: Python) {
        if self.0 != std::ptr::null_mut() {
            unsafe { ffi::PyDict_Clear(self.0) }
        }
    }
}

/// actural weakref field
#[repr(transparent)]
pub struct PyClassWeakRefSlot(*mut ffi::PyObject);

impl PyClassWeakRef for PyClassWeakRefSlot {
    private_impl! {}
    const OFFSET: Option<isize> = Some(-POINTER_SIZE);
    fn new() -> Self {
        Self(std::ptr::null_mut())
    }
    fn clear_weakrefs(&mut self, obj: *mut ffi::PyObject, _py: Python) {
        if self.0 != std::ptr::null_mut() {
            unsafe { ffi::PyObject_ClearWeakRefs(obj) }
        }
    }
}
