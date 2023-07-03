//! Support for [free allocation lists][1].
//!
//! This can improve performance for types that are often created and deleted in quick succession.
//!
//! Rather than implementing this manually,
//! implement it by annotating a struct with `#[pyclass(freelist = N)]`,
//! where `N` is the size of the freelist.
//!
//! [1]: https://en.wikipedia.org/wiki/Free_list

use std::mem;

/// Represents a slot of a [`FreeList`].
pub enum Slot<T> {
    /// A free slot.
    Empty,
    /// An allocated slot.
    Filled(T),
}

/// A free allocation list.
///
/// See [the parent module](crate::impl_::freelist) for more details.
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
