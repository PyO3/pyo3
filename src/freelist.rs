// Copyright (c) 2017-present PyO3 Project and Contributors

//! Support for [free allocation lists][1].
//!
//! This can improve performance for types that are often created and deleted in quick succession.
//!
//! Rather than implementing this manually,
//! implement it by annotating a struct with `#[pyclass(freelist = N)]`,
//! where `N` is the size of the freelist.
//!
//! [1]: https://en.wikipedia.org/wiki/Free_list

use crate::class::impl_::PyClassImpl;
use crate::pyclass::{get_type_free, tp_free_fallback, PyClassAlloc};
use crate::type_object::{PyLayout, PyTypeInfo};
use crate::{ffi, AsPyPointer, FromPyPointer, PyAny, Python};
use std::mem;
use std::os::raw::c_void;

/// Implements a freelist.
///
/// Do not implement this trait directly; instead use `#[pyclass(freelist = N)]`
/// on a Rust struct to implement it.
pub trait PyClassWithFreeList {
    fn get_free_list(py: Python) -> &mut FreeList<*mut ffi::PyObject>;
}

/// Represents a slot of a [`FreeList`].
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
    /// Creates a new `FreeList` instance with specified capacity.
    pub fn with_capacity(capacity: usize) -> FreeList<T> {
        let entries = (0..capacity).map(|_| Slot::Empty).collect::<Vec<_>>();

        FreeList {
            entries,
            split: 0,
            capacity,
        }
    }

    /// Pops the first non empty item.
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

    /// Inserts a value into the list. Returns `None` if the `FreeList` is full.
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
    T: PyTypeInfo + PyClassImpl + PyClassWithFreeList,
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
