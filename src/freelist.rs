// Copyright (c) 2017-present PyO3 Project and Contributors

//! Free allocation list

use crate::ffi;
use crate::pyclass::{tp_free_fallback, PyClassAlloc};
use crate::type_object::{PyLayout, PyTypeInfo};
use crate::Python;
use std::mem;
use std::os::raw::c_void;

/// Implementing this trait for custom class adds free allocation list to class.
/// The performance improvement applies to types that are often created and deleted in a row,
/// so that they can benefit from a freelist.
pub trait PyClassWithFreeList {
    fn get_free_list() -> &'static mut FreeList<*mut ffi::PyObject>;
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
    unsafe fn alloc(_py: Python) -> *mut Self::Layout {
        if let Some(obj) = <Self as PyClassWithFreeList>::get_free_list().pop() {
            ffi::PyObject_Init(obj, <Self as PyTypeInfo>::type_object() as *const _ as _);
            obj as _
        } else {
            crate::pyclass::default_alloc::<Self>() as _
        }
    }

    unsafe fn dealloc(py: Python, self_: *mut Self::Layout) {
        (*self_).py_drop(py);

        let obj = self_ as _;
        if ffi::PyObject_CallFinalizerFromDealloc(obj) < 0 {
            return;
        }

        if let Some(obj) = <Self as PyClassWithFreeList>::get_free_list().insert(obj) {
            match Self::type_object().tp_free {
                Some(free) => free(obj as *mut c_void),
                None => tp_free_fallback(obj),
            }
        }
    }
}
