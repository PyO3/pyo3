use crate::{types::any::PyAnyMethods, Bound, PyAny, PyResult};

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl Sealed for PyResult<Bound<'_, PyAny>> {}
}

use sealed::Sealed;

pub(crate) trait PyResultExt<'py>: Sealed {
    unsafe fn downcast_into_unchecked<T>(self) -> PyResult<Bound<'py, T>>;
}

impl<'py> PyResultExt<'py> for PyResult<Bound<'py, PyAny>> {
    #[inline]
    unsafe fn downcast_into_unchecked<T>(self) -> PyResult<Bound<'py, T>> {
        self.map(|instance| instance.downcast_into_unchecked())
    }
}
