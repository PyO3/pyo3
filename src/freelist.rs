// Copyright (c) 2017-present PyO3 Project and Contributors

//! Free allocation list

use crate::pyclass::{get_type_free, tp_free_fallback, PyClassAlloc};
use crate::type_object::{PyLayout, PyTypeInfo};
use crate::{ffi, AsPyPointer, FromPyPointer, PyAny, Python};
use std::mem;
use std::os::raw::c_void;

/// Implementing this trait for custom class adds free allocation list to class.
/// The performance improvement applies to types that are often created and deleted in a row,
/// so that they can benefit from a freelist.
pub trait PyClassWithFreeList {
    fn get_free_list(py: Python) -> &mut FreeList<*mut ffi::PyObject>;
}

pub enum Slot<T> {
    Empty,
    Filled(T),
}

pub struct FreeList<T> {
    entries: Vec<Slot<T>>,
    split: usize,
    capacity: usize,
}

impl<T> FreeList<T> {
    /// Create new `FreeList` instance with specified capacity
    pub fn with_capacity(capacity: usize) -> FreeList<T> {
        let entries = (0..capacity).map(|_| Slot::Empty).collect::<Vec<_>>();

        FreeList {
            entries,
            split: 0,
            capacity,
        }
    }

    /// Pop first non empty item
    pub fn pop(&mut self) -> Option<T> {
        let idx = self.split;
        if idx == 0 {
            None
        } else {
            match mem::replace(&mut self.entries[idx - 1], Slot::Empty) {
                Slot::Filled(v) => {
                    self.split = idx - 1;
                    Some(v)
                }
                _ => panic!("FreeList is corrupt"),
            }
        }
    }

    /// Insert a value into the list
    pub fn insert(&mut self, val: T) -> Option<T> {
        let next = self.split + 1;
        if next < self.capacity {
            self.entries[self.split] = Slot::Filled(val);
            self.split = next;
            None
        } else {
            Some(val)
        }
    }
}

impl<T> PyClassAlloc for T
where
    T: PyTypeInfo + PyClassWithFreeList,
{
    unsafe fn new(py: Python, subtype: *mut ffi::PyTypeObject) -> *mut Self::Layout {
        // if subtype is not equal to this type, we cannot use the freelist
        if subtype == Self::type_object_raw(py) {
            if let Some(obj) = <Self as PyClassWithFreeList>::get_free_list(py).pop() {
                ffi::PyObject_Init(obj, subtype);
                #[cfg(not(Py_3_8))]
                crate::pyclass::bpo_35810_workaround(py, subtype);
                return obj as _;
            }
        }
        crate::pyclass::default_new::<Self>(py, subtype) as _
    }

    #[allow(clippy::clippy::collapsible_if)] // for if cfg!
    unsafe fn dealloc(py: Python, self_: *mut Self::Layout) {
        (*self_).py_drop(py);
        let obj = PyAny::from_borrowed_ptr_or_panic(py, self_ as _);

        if let Some(obj) = <Self as PyClassWithFreeList>::get_free_list(py).insert(obj.as_ptr()) {
            let ty = ffi::Py_TYPE(obj);
            let free = get_type_free(ty).unwrap_or_else(|| tp_free_fallback(ty));
            free(obj as *mut c_void);

            if cfg!(Py_3_8) {
                if ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
                    ffi::Py_DECREF(ty as *mut ffi::PyObject);
                }
            }
        }
    }
}
