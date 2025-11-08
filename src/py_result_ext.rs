use crate::{Bound, PyAny, PyResult, PyTypeCheck};

pub(crate) trait PyResultExt<'py>: crate::sealed::Sealed {
    fn cast_into<T: PyTypeCheck>(self) -> PyResult<Bound<'py, T>>;
    unsafe fn cast_into_unchecked<T>(self) -> PyResult<Bound<'py, T>>;
}

impl<'py> PyResultExt<'py> for PyResult<Bound<'py, PyAny>> {
    #[inline]
    fn cast_into<T: PyTypeCheck>(self) -> PyResult<Bound<'py, T>> where {
        self.and_then(|instance| instance.cast_into().map_err(Into::into))
    }

    #[inline]
    unsafe fn cast_into_unchecked<T>(self) -> PyResult<Bound<'py, T>> {
        self.map(|instance| unsafe { instance.cast_into_unchecked() })
    }
}
