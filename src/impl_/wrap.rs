#![warn(clippy::undocumented_unsafe_blocks)]

use core::{convert::Infallible, marker::PhantomData, ops::Deref};

use crate::{
    ffi, types::PyNone, Bound, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyErr, PyResult, Python,
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

pub struct OkWrapper<T>(OkWrapperInner<T>);
pub struct OkWrapperInner<T>(PhantomData<T>);

impl<T> OkWrapper<T> {
    pub fn new(_: &T) -> Self {
        Self(OkWrapperInner(PhantomData))
    }
}

impl<T> Deref for OkWrapper<T> {
    type Target = OkWrapperInner<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, E> OkWrapper<Result<T, E>> {
    pub fn ok_wrap(&self, value: Result<T, E>) -> Result<T, E> {
        value
    }
}

impl<T> OkWrapperInner<T> {
    pub fn ok_wrap(&self, value: T) -> Result<T, Infallible> {
        Ok(value)
    }
}

// Hierarchy of conversions used in the function return type machinery
pub struct Converter<T>(EmptyTupleConverter<T>);
pub struct EmptyTupleConverter<T>(IntoPyObjectConverter<T>);
pub struct IntoPyObjectConverter<T>(UnknownReturnResultType<T>);
pub struct UnknownReturnResultType<T>(UnknownReturnType<T>);
pub struct UnknownReturnType<T>(PhantomData<T>);

pub fn converter<T>(_: &T) -> Converter<T> {
    Converter(EmptyTupleConverter(IntoPyObjectConverter(
        UnknownReturnResultType(UnknownReturnType(PhantomData)),
    )))
}

impl<T> Deref for Converter<T> {
    type Target = EmptyTupleConverter<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Deref for EmptyTupleConverter<T> {
    type Target = IntoPyObjectConverter<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Deref for IntoPyObjectConverter<T> {
    type Target = UnknownReturnResultType<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Deref for UnknownReturnResultType<T> {
    type Target = UnknownReturnType<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EmptyTupleConverter<()> {
    #[inline]
    pub fn wrap_into_ptr(&self, py: Python<'_>, _obj: ()) -> PyResult<*mut ffi::PyObject> {
        Ok(PyNone::get(py).to_owned().into_ptr())
    }

    #[inline]
    pub fn wrap_into_pyobject(&self, py: Python<'_>, _obj: ()) -> PyResult<Py<PyAny>> {
        Ok(PyNone::get(py).to_owned().into_any().unbind())
    }
}

impl<E> EmptyTupleConverter<Result<(), E>>
where
    PyErr: From<E>,
{
    #[inline]
    pub fn wrap_into_ptr(
        &self,
        py: Python<'_>,
        obj: Result<(), E>,
    ) -> PyResult<*mut ffi::PyObject> {
        obj.map(|_| PyNone::get(py).to_owned().into_ptr())
            .map_err(PyErr::from)
    }

    #[inline]
    pub fn wrap_into_pyobject(&self, py: Python<'_>, obj: Result<(), E>) -> PyResult<Py<PyAny>> {
        obj.map(|_| PyNone::get(py).to_owned().into_any().unbind())
            .map_err(PyErr::from)
    }
}

impl<'py, T: IntoPyObject<'py>> IntoPyObjectConverter<T> {
    #[inline]
    pub fn wrap(&self, obj: T) -> Result<T, Infallible> {
        Ok(obj)
    }

    #[inline]
    pub fn wrap_into_ptr(&self, py: Python<'py>, obj: T) -> PyResult<*mut ffi::PyObject> {
        obj.into_bound_py_any(py).map(Bound::into_ptr)
    }

    #[inline]
    pub fn wrap_into_pyobject(&self, py: Python<'py>, obj: T) -> PyResult<Py<PyAny>> {
        obj.into_py_any(py)
    }
}

impl<'py, T: IntoPyObject<'py>, E> IntoPyObjectConverter<Result<T, E>> {
    #[inline]
    pub fn wrap(&self, obj: Result<T, E>) -> Result<T, E> {
        obj
    }

    #[inline]
    pub fn wrap_into_ptr(&self, py: Python<'py>, obj: Result<T, E>) -> PyResult<*mut ffi::PyObject>
    where
        PyErr: From<E>,
    {
        obj.map_err(PyErr::from)
            .and_then(|obj| obj.into_bound_py_any(py))
            .map(Bound::into_ptr)
    }

    #[inline]
    pub fn wrap_into_pyobject(&self, py: Python<'py>, obj: Result<T, E>) -> PyResult<Py<PyAny>>
    where
        PyErr: From<E>,
    {
        obj.map_err(PyErr::from).and_then(|obj| obj.into_py_any(py))
    }
}

impl<T, E> UnknownReturnResultType<Result<T, E>> {
    #[inline]
    pub fn wrap<'py>(&self, _: Result<T, E>) -> Result<T, E>
    where
        T: IntoPyObject<'py>,
    {
        unreachable!("should be handled by IntoPyObjectConverter")
    }

    #[inline]
    pub fn wrap_into_ptr<'py>(
        &self,
        _: Python<'py>,
        _: Result<T, E>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        T: IntoPyObject<'py>,
    {
        unreachable!("should be handled by IntoPyObjectConverter")
    }

    #[inline]
    pub fn wrap_into_pyobject<'py>(&self, _: Python<'py>, _: Result<T, E>) -> PyResult<Py<PyAny>>
    where
        T: IntoPyObject<'py>,
    {
        unreachable!("should be handled by IntoPyObjectConverter")
    }
}

impl<T> UnknownReturnType<T> {
    #[inline]
    pub fn wrap<'py>(&self, _: T) -> T
    where
        T: IntoPyObject<'py>,
    {
        unreachable!("should be handled by IntoPyObjectConverter")
    }

    #[inline]
    pub fn wrap_into_ptr<'py>(&self, _: Python<'py>, _: T) -> PyResult<*mut ffi::PyObject>
    where
        T: IntoPyObject<'py>,
    {
        unreachable!("should be handled by IntoPyObjectConverter")
    }

    #[inline]
    pub fn wrap_into_pyobject<'py>(&self, _: Python<'py>, _: T) -> PyResult<Py<PyAny>>
    where
        T: IntoPyObject<'py>,
    {
        unreachable!("should be handled by IntoPyObjectConverter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::prelude::*;

    #[test]
    fn wrap_option() {
        let a: Option<u8> = SomeWrap::wrap(42);
        assert_eq!(a, Some(42));

        let b: Option<u8> = SomeWrap::wrap(None);
        assert_eq!(b, None);
    }

    #[test]
    fn wrap_result() {
        let a = 42;
        let Ok(a) = OkWrapper::new(&a).ok_wrap(a);
        assert_eq!(a, 42);

        let b = Result::<_, String>::Ok(42);
        let b = OkWrapper::new(&b).ok_wrap(b);
        assert_eq!(b, Ok(42));
    }
}
