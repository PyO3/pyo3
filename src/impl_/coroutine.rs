use std::future::Future;

use crate::{
    coroutine::{cancel::ThrowCallback, Coroutine},
    instance::Bound,
    types::PyString,
    IntoPyObject, Py, PyClass, PyResult,
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

fn get_ptr<T: PyClass>(obj: &Py<T>) -> *mut T {
    obj.get_class_object().get_ptr()
}
