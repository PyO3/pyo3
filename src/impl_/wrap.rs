use std::convert::Infallible;

use crate::{
    conversion::IntoPyObject, ffi, types::PyNone, Bound, IntoPy, PyErr, PyObject, PyResult, Python,
};

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
#[cfg_attr(
    diagnostic_namespace,
    diagnostic::on_unimplemented(
        message = "`{Self}` cannot be converted to a Python object",
        note = "`IntoPy` is automatically implemented by the `#[pyclass]` macro",
        note = "if you do not wish to have a corresponding Python type, implement `IntoPy` manually",
        note = "if you do not own `{Self}` you can perform a manual conversion to one of the types in `pyo3::types::*`"
    )
)]
pub trait OkWrapIntoPy<T> {
    type Error;
    fn wrap(self) -> Result<T, Self::Error>;
}

// The T: IntoPy<PyObject> bound here is necessary to prevent the
// implementation for Result<T, E> from conflicting
impl<T> OkWrapIntoPy<T> for T
where
    T: IntoPy<PyObject>,
{
    type Error = Infallible;
    #[inline]
    fn wrap(self) -> Result<T, Infallible> {
        Ok(self)
    }
}

impl<T, E> OkWrapIntoPy<T> for Result<T, E>
where
    T: IntoPy<PyObject>,
{
    type Error = E;
    #[inline]
    fn wrap(self) -> Result<T, Self::Error> {
        self
    }
}

/// Used to wrap the result of `#[pyfunction]` and `#[pymethods]`.
pub trait OkWrapIntoPyObject<T> {
    type Error;
    fn wrap(self) -> Result<T, Self::Error>;
}

// The T: IntoPy<PyObject> bound here is necessary to prevent the
// implementation for Result<T, E> from conflicting
impl<'py, T> OkWrapIntoPyObject<T> for T
where
    T: IntoPyObject<'py>,
{
    type Error = Infallible;
    #[inline]
    fn wrap(self) -> Result<T, Infallible> {
        Ok(self)
    }
}

impl<'py, T, E> OkWrapIntoPyObject<T> for Result<T, E>
where
    T: IntoPyObject<'py>,
{
    type Error = E;
    #[inline]
    fn wrap(self) -> Result<T, Self::Error> {
        self
    }
}

pub struct IntoPyTag;
impl IntoPyTag {
    #[inline]
    pub fn map_into_ptr<T: IntoPy<PyObject>>(
        self,
        py: Python<'_>,
        obj: PyResult<T>,
    ) -> PyResult<*mut ffi::PyObject> {
        obj.map(|obj| obj.into_py(py).into_ptr())
    }

    #[inline]
    pub fn wrap<S, T: OkWrapIntoPy<S>>(self, obj: T) -> Result<S, T::Error> {
        obj.wrap()
    }
}
pub trait IntoPyKind {
    #[inline]
    fn conversion_kind(&self) -> IntoPyTag {
        IntoPyTag
    }
}
impl<T: IntoPy<PyObject>> IntoPyKind for T {}
impl<T: IntoPy<PyObject>, E> IntoPyKind for Result<T, E> {}

pub struct IntoPyObjectTag;
impl IntoPyObjectTag {
    #[inline]
    pub fn map_into_ptr<'py, T>(
        self,
        py: Python<'py>,
        obj: PyResult<T>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        T: IntoPyObject<'py>,
        PyErr: From<T::Error>,
    {
        obj.and_then(|obj| obj.into_pyobject(py).map_err(Into::into))
            .map(Bound::into_ptr)
    }

    #[inline]
    pub fn wrap<S, T: OkWrapIntoPyObject<S>>(self, obj: T) -> Result<S, T::Error> {
        obj.wrap()
    }
}
pub trait IntoPyObjectKind {
    #[inline]
    fn conversion_kind(&self) -> IntoPyObjectTag {
        IntoPyObjectTag
    }
}
impl<'py, T: IntoPyObject<'py>> IntoPyObjectKind for &T {}
impl<'py, T: IntoPyObject<'py>, E> IntoPyObjectKind for &Result<T, E> {}

pub struct IntoPyNoneTag;
impl IntoPyNoneTag {
    #[inline]
    pub fn map_into_ptr(self, py: Python<'_>, obj: PyResult<()>) -> PyResult<*mut ffi::PyObject> {
        obj.map(|_| PyNone::get_bound(py).to_owned().into_ptr())
    }
}
pub trait IntoPyNoneKind {
    #[inline]
    fn conversion_kind(&self) -> IntoPyNoneTag {
        IntoPyNoneTag
    }
}
impl<E> IntoPyNoneKind for &&Result<(), E> {}

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
        let a: Result<u8, _> = OkWrapIntoPy::wrap(42u8);
        assert!(matches!(a, Ok(42)));

        let b: PyResult<u8> = OkWrapIntoPy::wrap(Ok(42u8));
        assert!(matches!(b, Ok(42)));

        let c: Result<u8, &str> = OkWrapIntoPy::wrap(Err("error"));
        assert_eq!(c, Err("error"));
    }
}
