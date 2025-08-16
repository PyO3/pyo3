use std::{convert::Infallible, marker::PhantomData, ops::Deref};

use crate::{
    ffi, types::PyNone, Bound, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyResult, Python,
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

// Hierarchy of conversions used in the `IntoPy` implementation
pub struct Converter<T>(EmptyTupleConverter<T>);
pub struct EmptyTupleConverter<T>(IntoPyObjectConverter<T>);
pub struct IntoPyObjectConverter<T>(IntoPyConverter<T>);
pub struct IntoPyConverter<T>(UnknownReturnResultType<T>);
pub struct UnknownReturnResultType<T>(UnknownReturnType<T>);
pub struct UnknownReturnType<T>(PhantomData<T>);

pub fn converter<T>(_: &T) -> Converter<T> {
    Converter(EmptyTupleConverter(IntoPyObjectConverter(IntoPyConverter(
        UnknownReturnResultType(UnknownReturnType(PhantomData)),
    ))))
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
    type Target = IntoPyConverter<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Deref for IntoPyConverter<T> {
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

impl EmptyTupleConverter<PyResult<()>> {
    #[inline]
    pub fn map_into_ptr(&self, py: Python<'_>, obj: PyResult<()>) -> PyResult<*mut ffi::PyObject> {
        obj.map(|_| PyNone::get(py).to_owned().into_ptr())
    }
}

impl<'py, T: IntoPyObject<'py>> IntoPyObjectConverter<T> {
    #[inline]
    pub fn wrap(&self, obj: T) -> Result<T, Infallible> {
        Ok(obj)
    }
}

impl<'py, T: IntoPyObject<'py>, E> IntoPyObjectConverter<Result<T, E>> {
    #[inline]
    pub fn wrap(&self, obj: Result<T, E>) -> Result<T, E> {
        obj
    }

    #[inline]
    pub fn map_into_pyobject(&self, py: Python<'py>, obj: PyResult<T>) -> PyResult<Py<PyAny>>
    where
        T: IntoPyObject<'py>,
    {
        obj.and_then(|obj| obj.into_py_any(py))
    }

    #[inline]
    pub fn map_into_ptr(&self, py: Python<'py>, obj: PyResult<T>) -> PyResult<*mut ffi::PyObject>
    where
        T: IntoPyObject<'py>,
    {
        obj.and_then(|obj| obj.into_bound_py_any(py))
            .map(Bound::into_ptr)
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
    pub fn map_into_pyobject<'py>(&self, _: Python<'py>, _: PyResult<T>) -> PyResult<Py<PyAny>>
    where
        T: IntoPyObject<'py>,
    {
        unreachable!("should be handled by IntoPyObjectConverter")
    }

    #[inline]
    pub fn map_into_ptr<'py>(&self, _: Python<'py>, _: PyResult<T>) -> PyResult<*mut ffi::PyObject>
    where
        T: IntoPyObject<'py>,
    {
        unreachable!("should be handled by IntoPyObjectConverter")
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
