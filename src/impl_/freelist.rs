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
use core::ptr::NonNull;

/// A free allocation list for PyObject ffi pointers.
///
/// See [the parent module](crate::impl_::freelist) for more details.
pub struct PyObjectFreeList {
    entries: Box<[Option<NonNull<ffi::PyObject>>]>,
    split: usize,
    capacity: usize,
}

// safety: the pointers are never used internally and they are cleared when they are given out
unsafe impl Send for PyObjectFreeList {}

impl PyObjectFreeList {
    /// Creates a new `PyObjectFreeList` instance with specified capacity.
    pub fn with_capacity(capacity: usize) -> PyObjectFreeList {
        let entries = vec![None; capacity].into_boxed_slice();

        PyObjectFreeList {
            entries,
            split: 0,
            capacity,
        }
    }

    /// Pops the first non empty item.
    pub fn pop(&mut self) -> Option<NonNull<ffi::PyObject>> {
        let idx = self.split;
        if idx == 0 {
            None
        } else {
            let val = self.entries[idx - 1]
                .take()
                .expect("PyObjectFreeList is corrupt");
            self.split = idx - 1;
            Some(val)
        }
    }

    /// Inserts a value into the list. Returns `Some(val)` if the `PyObjectFreeList` is full.
    pub fn insert(&mut self, val: NonNull<ffi::PyObject>) -> Option<NonNull<ffi::PyObject>> {
        let next = self.split + 1;
        if next < self.capacity {
            self.entries[self.split] = Some(val);
            self.split = next;
            None
        } else {
            Some(val)
        }
    }
}
