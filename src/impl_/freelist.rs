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
use crate::marker::Python;
use crate::platform::prelude::*;
use crate::platform::sync::non_poison::Mutex;
use crate::sync::{MutexExt, PyOnceLock};

use core::ops::DerefMut;
use core::ptr::NonNull;

pub struct FreeList(PyOnceLock<Mutex<PyObjectFreeList>>);

impl FreeList {
    #[expect(clippy::new_without_default, reason = "always called in const context")]
    pub const fn new() -> Self {
        Self(PyOnceLock::new())
    }

    pub fn get(
        &self,
        py: Python<'_>,
        capacity: usize,
    ) -> impl DerefMut<Target = PyObjectFreeList> + '_ {
        self.0
            .get_or_init(py, || Mutex::new(PyObjectFreeList::with_capacity(capacity)))
            .lock_py_attached(py)
    }
}

/// A free allocation list for PyObject ffi pointers.
///
/// See [the parent module](crate::impl_::freelist) for more details.
pub struct PyObjectFreeList {
    entries: Box<[Option<NonNull<ffi::PyObject>>]>,
    split: usize,
}

// safety: the pointers are never used internally and they are cleared when they are given out
unsafe impl Send for PyObjectFreeList {}

impl PyObjectFreeList {
    /// Creates a new `PyObjectFreeList` instance with specified capacity.
    pub fn with_capacity(capacity: usize) -> PyObjectFreeList {
        let entries = vec![None; capacity].into_boxed_slice();

        PyObjectFreeList { entries, split: 0 }
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
        if self.split < self.entries.len() {
            self.entries[self.split] = Some(val);
            self.split += 1;
            None
        } else {
            Some(val)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_capacity_and_lifo() {
        let mut allocations = [0u8; 18];
        let pointers: Vec<_> = allocations
            .iter_mut()
            .map(|value| NonNull::from(value).cast())
            .collect();
        for capacity in [0, 1, 2, 17] {
            let mut list = PyObjectFreeList::with_capacity(capacity);
            for _ in 0..2 {
                assert_eq!(list.pop(), None);
                for &ptr in &pointers[..capacity] {
                    assert_eq!(list.insert(ptr), None, "capacity {capacity}");
                }
                assert_eq!(list.insert(pointers[capacity]), Some(pointers[capacity]));
                for &ptr in pointers[..capacity].iter().rev() {
                    assert_eq!(list.pop(), Some(ptr));
                }
                assert_eq!(list.pop(), None);
            }
        }
    }
}
