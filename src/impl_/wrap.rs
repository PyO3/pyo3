use crate::{IntoPy, Py, PyAny, PyErr, PyObject, PyResult, Python};

/// Used to wrap values in `Option<T>` for default arguments.
pub trait SomeWrap<T> {
    fn wrap(self) -> T;
}

impl<T> SomeWrap<Option<T>> for T {
    fn wrap(self) -> Option<T> {
        Some(self)
    }
}

impl<T> SomeWrap<Option<T>> for Option<T> {
    fn wrap(self) -> Self {
        self
    }
}

/// Used to wrap the result of `#[pyfunction]` and `#[pymethods]`.
pub trait OkWrap<T> {
    type Error;
    fn wrap(self, py: Python<'_>) -> Result<Py<PyAny>, Self::Error>;
}

// The T: IntoPy<PyObject> bound here is necessary to prevent the
// implementation for Result<T, E> from conflicting
impl<T> OkWrap<T> for T
where
    T: IntoPy<PyObject>,
{
    type Error = PyErr;
    fn wrap(self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(self.into_py(py))
    }
}

impl<T, E> OkWrap<T> for Result<T, E>
where
    T: IntoPy<PyObject>,
{
    type Error = E;
    fn wrap(self, py: Python<'_>) -> Result<Py<PyAny>, Self::Error> {
        self.map(|o| o.into_py(py))
    }
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
}
