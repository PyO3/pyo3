// Copyright (c) 2017-present PyO3 Project and Contributors

//! Free allocation list
use std;

use ffi;
use err::PyResult;
use python::Python;
use typeob::{PyTypeInfo, PyObjectAlloc};

/// Implementing this trait for custom class adds free allocation list to class.
/// The performance improvement applies to types that are often created and deleted in a row,
/// so that they can benefit from a freelist.
pub trait PyObjectWithFreeList: PyTypeInfo {

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
            entries: entries,
            split: 0,
            capacity: capacity,
        }
    }

    /// Pop first non empty item
    pub fn pop(&mut self) -> Option<T> {
        let idx = self.split;
        if idx == 0 {
            None
        } else {
            match std::mem::replace(&mut self.entries[idx-1], Slot::Empty) {
                Slot::Filled(v) => {
                    self.split = idx - 1;
                    Some(v)
                }
                _ => panic!("FreeList is corrupt")
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


impl<T> PyObjectAlloc<T> for T where T: PyObjectWithFreeList {

    unsafe fn alloc(_py: Python, value: T) -> PyResult<*mut ffi::PyObject> {
        let obj = if let Some(obj) = <T as PyObjectWithFreeList>::get_free_list().pop() {
            ffi::PyObject_Init(obj, <T as PyTypeInfo>::type_object());
            obj
        } else {
            ffi::PyType_GenericAlloc(<T as PyTypeInfo>::type_object(), 0)
        };

        let offset = <T as PyTypeInfo>::offset();
        let ptr = (obj as *mut u8).offset(offset) as *mut T;
        std::ptr::write(ptr, value);

        Ok(obj)
    }

    unsafe fn dealloc(_py: Python, obj: *mut ffi::PyObject) {
        let ptr = (obj as *mut u8).offset(<T as PyTypeInfo>::offset()) as *mut T;
        std::ptr::drop_in_place(ptr);

        if let Some(obj) = <T as PyObjectWithFreeList>::get_free_list().insert(obj) {
            ffi::PyObject_Free(obj as *mut ::c_void);
        }
    }
}
