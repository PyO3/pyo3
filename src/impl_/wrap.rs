use std::convert::Infallible;

use crate::{ffi, IntoPy, PyObject, PyResult, Python};

/// Used to wrap values in `Option<T>` for default arguments.
pub trait SomeWrap<T> {
    fn wrap(self) -> Option<T>;
}

impl<T> SomeWrap<T> for T {
    fn wrap(self) -> Option<T> {
        Some(self)
    }
}

impl<T> SomeWrap<T> for Option<T> {
    fn wrap(self) -> Self {
        self
    }
}

/// Used to wrap the result of `#[pyfunction]` and `#[pymethods]`.
pub trait OkWrap<T> {
    type Error;
    fn wrap(self) -> Result<T, Self::Error>;
}

// The T: IntoPy<PyObject> bound here is necessary to prevent the
// implementation for Result<T, E> from conflicting
impl<T> OkWrap<T> for T
where
    T: IntoPy<PyObject>,
{
    type Error = Infallible;
    #[inline]
    fn wrap(self) -> Result<T, Infallible> {
        Ok(self)
    }
}

impl<T, E> OkWrap<T> for Result<T, E>
where
    T: IntoPy<PyObject>,
{
    type Error = E;
    #[inline]
    fn wrap(self) -> Result<T, Self::Error> {
        self
    }
}

/// This is a follow-up function to `OkWrap::wrap` that converts the result into
/// a `*mut ffi::PyObject` pointer.
pub fn map_result_into_ptr<T: IntoPy<PyObject>>(
    py: Python<'_>,
    result: PyResult<T>,
) -> PyResult<*mut ffi::PyObject> {
    result.map(|obj| obj.into_py(py).into_ptr())
}

/// This is a follow-up function to `OkWrap::wrap` that converts the result into
/// a safe wrapper.
pub fn map_result_into_py<T: IntoPy<PyObject>>(
    py: Python<'_>,
    result: PyResult<T>,
) -> PyResult<PyObject> {
    result.map(|err| err.into_py(py))
}

/// Used to wrap the result of async `#[pyfunction]` and `#[pymethods]`.
#[cfg(feature = "macros")]
pub fn wrap_future<F, R, T>(future: F) -> crate::coroutine::Coroutine
where
    F: std::future::Future<Output = R> + Send + 'static,
    R: OkWrap<T>,
    T: IntoPy<PyObject>,
    crate::PyErr: From<R::Error>,
{
    crate::coroutine::Coroutine::from_future::<_, T, crate::PyErr>(async move {
        OkWrap::wrap(future.await).map_err(Into::into)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_option() {
        let a: Option<u8> = SomeWrap::wrap(42);
        assert_eq!(a, Some(42));

        let b: Option<u8> = SomeWrap::wrap(None);
        assert_eq!(b, None);
    }

    #[test]
    fn wrap_result() {
        let a: Result<u8, _> = OkWrap::wrap(42u8);
        assert!(matches!(a, Ok(42)));

        let b: PyResult<u8> = OkWrap::wrap(Ok(42u8));
        assert!(matches!(b, Ok(42)));

        let c: Result<u8, &str> = OkWrap::wrap(Err("error"));
        assert_eq!(c, Err("error"));
    }
}
