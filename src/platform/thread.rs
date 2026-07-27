use core::ffi::c_ulong;
use core::num::NonZero;

#[cfg(all(PyPy, not(wip_feature_std)))]
compile_error!("Thread ID is not available for PyPy with no_std");

#[cfg(not(wip_feature_std))]
use pyo3_ffi::PyThread_get_thread_ident;

#[must_use]
pub fn current() -> Thread {
    cfg_select! {
        wip_feature_std => Thread { id: ThreadId(std::thread::current().id()) },
        _ => Thread {
            // SAFETY: PyThread_get_thread_ident never returns zero
            // https://docs.python.org/3/c-api/threads.html#c.PyThread_get_thread_ident
            id: ThreadId(unsafe { NonZero::new_unchecked(PyThread_get_thread_ident()) }),
        }
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

// This needs to be in a separate mod so that the `expect` attribute covers the derived code.
#[cfg(wip_feature_std)]
#[expect(clippy::disallowed_types)]
mod std_feature {
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
    pub struct ThreadId(pub(super) std::thread::ThreadId);
}

#[cfg(wip_feature_std)]
pub use std_feature::ThreadId;

#[cfg(not(wip_feature_std))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ThreadId(NonZero<c_ulong>);
