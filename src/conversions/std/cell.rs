use std::cell::Cell;

use crate::{
    conversion::IntoPyObject, types::any::PyAnyMethods, Bound, FromPyObject, PyAny, PyResult,
    Python,
};

impl<'py, T: Copy + IntoPyObject<'py>> IntoPyObject<'py> for Cell<T> {
    type Target = T::Target;
    type Output = T::Output;
    type Error = T::Error;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = T::OUTPUT_TYPE;

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
    const OUTPUT_TYPE: &'static str = T::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.get().into_pyobject(py)
    }
}

impl<'py, T: FromPyObject<'py>> FromPyObject<'py> for Cell<T> {
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = T::INPUT_TYPE;

    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        ob.extract().map(Cell::new)
    }
}
