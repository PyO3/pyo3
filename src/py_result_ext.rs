use crate::{internal::err::ErrorAlreadySet, Bound, PyAny, PyErr, PyResult, PyTypeCheck};

pub(crate) trait PyResultExt<'py>: crate::sealed::Sealed {
    type Error;
    fn cast_into<T: PyTypeCheck>(self) -> Result<Bound<'py, T>, PyErr>;
    unsafe fn cast_into_unchecked<T>(self) -> Result<Bound<'py, T>, Self::Error>;
}

impl<'py> PyResultExt<'py> for PyResult<Bound<'py, PyAny>> {
    type Error = PyErr;

    #[inline]
    fn cast_into<T: PyTypeCheck>(self) -> PyResult<Bound<'py, T>> {
        self.and_then(|instance| instance.cast_into().map_err(Into::into))
    }

    #[inline]
    unsafe fn cast_into_unchecked<T>(self) -> PyResult<Bound<'py, T>> {
        self.map(|instance| unsafe { instance.cast_into_unchecked() })
    }
}

impl<'py> PyResultExt<'py> for Result<Bound<'py, PyAny>, ErrorAlreadySet<'py>> {
    type Error = ErrorAlreadySet<'py>;

    #[inline]
    fn cast_into<T: PyTypeCheck>(self) -> Result<Bound<'py, T>, PyErr> {
        self?.cast_into().map_err(Into::into)
    }

    #[inline]
    unsafe fn cast_into_unchecked<T>(self) -> Result<Bound<'py, T>, ErrorAlreadySet<'py>> {
        self.map(|instance| unsafe { instance.cast_into_unchecked() })
    }
}
