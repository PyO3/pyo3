use core::ffi::c_ulong;
use core::num::NonZero;

use pyo3_ffi::PyThread_get_thread_ident;

#[must_use]
pub fn current() -> Thread {
    Thread {
        // SAFETY: PyThread_get_thread_ident never returns zero
        // https://docs.python.org/3/c-api/threads.html#c.PyThread_get_thread_ident
        id: ThreadId(unsafe { NonZero::new_unchecked(PyThread_get_thread_ident()) }),
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
pub struct ThreadId(NonZero<c_ulong>);
