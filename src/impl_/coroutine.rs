use std::future::Future;

use crate::{coroutine::Coroutine, types::PyString, IntoPy, PyErr, PyObject};

pub fn new_coroutine<F, T, E>(
    name: &PyString,
    qualname_prefix: Option<&'static str>,
    future: F,
) -> Coroutine
where
    F: Future<Output = Result<T, E>> + Send + 'static,
    T: IntoPy<PyObject>,
    E: Into<PyErr>,
{
    Coroutine::new(Some(name.into()), qualname_prefix, future)
}
