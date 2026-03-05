use std::cell::Cell;

#[cfg(feature = "experimental-inspect")]
use crate::inspect::PyStaticExpr;
use crate::{conversion::IntoPyObject, Borrowed, FromPyObject, PyAny, Python};

impl<'py, T: Copy + IntoPyObject<'py>> IntoPyObject<'py> for Cell<T> {
    type Target = T::Target;
    type Output = T::Output;
    type Error = T::Error;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = T::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.get().into_pyobject(py)
    }
}

impl<'py, T: Copy + IntoPyObject<'py>> IntoPyObject<'py> for &Cell<T> {
    type Target = T::Target;
    type Output = T::Output;
    type Error = T::Error;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = T::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.get().into_pyobject(py)
    }
}

impl<'a, 'py, T: FromPyObject<'a, 'py>> FromPyObject<'a, 'py> for Cell<T> {
    type Error = T::Error;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = T::INPUT_TYPE;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        ob.extract().map(Cell::new)
    }
}
