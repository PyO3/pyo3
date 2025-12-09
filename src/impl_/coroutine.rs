use std::future::Future;

use crate::{
    coroutine::{cancel::ThrowCallback, Coroutine},
    instance::Bound,
    types::PyString,
    IntoPyObject, PyResult, Python,
};

pub fn new_coroutine<'py, F, T>(
    name: &Bound<'py, PyString>,
    qualname_prefix: Option<&'static str>,
    throw_callback: Option<ThrowCallback>,
    future: F,
) -> Coroutine
where
    F: Future<Output = PyResult<T>> + Send + 'static,
    T: IntoPyObject<'py>,
{
    Coroutine::new(Some(name.clone()), qualname_prefix, throw_callback, future)
}

/// Handle which assumes that the coroutine is attached to the thread. Unlike `Python<'_>`, this is `Send`.
pub struct AssumeAttachedInCoroutine(());

impl AssumeAttachedInCoroutine {
    /// Safety: this should only be used inside a future passed to `new_coroutine`, where the coroutine is
    /// guaranteed to be attached to the thread when polled.
    pub unsafe fn new() -> Self {
        Self(())
    }

    pub fn py(&self) -> Python<'_> {
        // Safety: this type holds the invariant that the thread is attached
        unsafe { Python::assume_attached() }
    }
}
