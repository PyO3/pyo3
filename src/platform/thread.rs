use core::num::NonZero;

use pyo3_ffi::PyThread_get_thread_ident;

#[must_use]
pub fn current() -> Thread {
    Thread {
        id: ThreadId(unsafe { PyThread_get_thread_ident() }),
    }
}

pub struct Thread {
    id: ThreadId,
}

impl Thread {
    pub fn id(&self) -> ThreadId {
        self.id
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ThreadId(NonZero<u64>);
