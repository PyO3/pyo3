use crate::{types::any::PyAnyMethods, Bound, PyAny, PyResult, PyTypeCheck};

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl Sealed for PyResult<Bound<'_, PyAny>> {}
}

use sealed::Sealed;

pub(crate) trait PyResultExt<'py>: Sealed {
    fn downcast_into<T: PyTypeCheck>(self) -> PyResult<Bound<'py, T>>;
    unsafe fn downcast_into_unchecked<T>(self) -> PyResult<Bound<'py, T>>;
}

impl<'py> PyResultExt<'py> for PyResult<Bound<'py, PyAny>> {
    #[inline]
    fn downcast_into<T: PyTypeCheck>(self) -> PyResult<Bound<'py, T>> where {
        self.and_then(|instance| instance.downcast_into().map_err(Into::into))
    }

    #[inline]
    unsafe fn downcast_into_unchecked<T>(self) -> PyResult<Bound<'py, T>> {
        self.map(|instance| instance.downcast_into_unchecked())
    }
}
