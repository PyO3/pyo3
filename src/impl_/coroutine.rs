use std::future::Future;

use crate::coroutine::cancel::ThrowCallback;
use crate::{coroutine::Coroutine, types::PyString, IntoPy, PyErr, PyObject};

pub fn new_coroutine<F, T, E>(
    name: &PyString,
    qualname_prefix: Option<&'static str>,
    throw_callback: Option<ThrowCallback>,
    future: F,
) -> Coroutine
where
    F: Future<Output = Result<T, E>> + Send + 'static,
    T: IntoPy<PyObject>,
    E: Into<PyErr>,
{
    Coroutine::new(Some(name.into()), qualname_prefix, throw_callback, future)
}
