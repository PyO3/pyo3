use crate::coroutine::Coroutine;
use crate::impl_::wrap::OkWrap;
use crate::{IntoPy, PyErr, PyObject, Python};
use std::future::Future;

/// Used to wrap the result of async `#[pyfunction]` and `#[pymethods]`.
pub fn wrap_future<F, R, T>(future: F) -> Coroutine
where
    F: Future<Output = R> + Send + 'static,
    R: OkWrap<T>,
    T: IntoPy<PyObject>,
    PyErr: From<R::Error>,
{
    let future = async move {
        // SAFETY: GIL is acquired when future is polled (see `Coroutine::poll`)
        future.await.wrap(unsafe { Python::assume_gil_acquired() })
    };
    Coroutine::from_future(future)
}
