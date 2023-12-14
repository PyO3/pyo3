use crate::{types::any::PyAnyMethods, Py2, PyAny, PyResult};

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl Sealed for PyResult<Py2<'_, PyAny>> {}
}

use sealed::Sealed;

pub(crate) trait PyResultExt<'py>: Sealed {
    unsafe fn downcast_into_unchecked<T>(self) -> PyResult<Py2<'py, T>>;
}

impl<'py> PyResultExt<'py> for PyResult<Py2<'py, PyAny>> {
    #[inline]
    unsafe fn downcast_into_unchecked<T>(self) -> PyResult<Py2<'py, T>> {
        self.map(|instance| instance.downcast_into_unchecked())
    }
}
