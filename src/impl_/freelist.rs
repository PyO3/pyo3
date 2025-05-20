//! Support for [free allocation lists][1].
//!
//! This can improve performance for types that are often created and deleted in quick succession.
//!
//! Rather than implementing this manually,
//! implement it by annotating a struct with `#[pyclass(freelist = N)]`,
//! where `N` is the size of the freelist.
//!
//! [1]: https://en.wikipedia.org/wiki/Free_list

use crate::ffi;
use std::mem;

/// Represents a slot of a [`PyObjectFreeList`].
enum PyObjectSlot {
    /// A free slot.
    Empty,
    /// An allocated slot.
    Filled(*mut ffi::PyObject),
}

// safety: access is guarded by a per-pyclass mutex
unsafe impl Send for PyObjectSlot {}

/// A free allocation list for PyObject ffi pointers.
///
/// See [the parent module](crate::impl_::freelist) for more details.
pub struct PyObjectFreeList {
    entries: Box<[PyObjectSlot]>,
    split: usize,
    capacity: usize,
}

impl PyObjectFreeList {
    /// Creates a new `PyObjectFreeList` instance with specified capacity.
    pub fn with_capacity(capacity: usize) -> PyObjectFreeList {
        let entries = (0..capacity)
            .map(|_| PyObjectSlot::Empty)
            .collect::<Box<[_]>>();

        PyObjectFreeList {
            entries,
            split: 0,
            capacity,
        }
    }

    /// Pops the first non empty item.
    pub fn pop(&mut self) -> Option<*mut ffi::PyObject> {
        let idx = self.split;
        if idx == 0 {
            None
        } else {
            match mem::replace(&mut self.entries[idx - 1], PyObjectSlot::Empty) {
                PyObjectSlot::Filled(v) => {
                    self.split = idx - 1;
                    Some(v)
                }
                _ => panic!("PyObjectFreeList is corrupt"),
            }
        }
    }

    /// Inserts a value into the list. Returns `Some(val)` if the `PyObjectFreeList` is full.
    pub fn insert(&mut self, val: *mut ffi::PyObject) -> Option<*mut ffi::PyObject> {
        let next = self.split + 1;
        if next < self.capacity {
            self.entries[self.split] = PyObjectSlot::Filled(val);
            self.split = next;
            None
        } else {
            Some(val)
        }
    }
}
