pub mod objects;

pub use objects::{FromPyObject, PyNativeObject, PyTryFrom};

pub mod types {
    pub use crate::types::experimental::*;
}

/// Alternative prelude to use the new experimental types / traits.
pub mod prelude {
    pub use super::{FromPyObject, PyTryFrom};

    pub use crate::err::{PyErr, PyResult};
    pub use crate::gil::GILGuard;
    pub use crate::instance::{Py, PyObject};
    pub use crate::pycell::{PyCell, PyRef, PyRefMut};
    pub use crate::pyclass_init::PyClassInitializer;
    pub use crate::python::Python;
    pub use crate::{IntoPy, IntoPyPointer, PyTryInto, ToPyObject};
    // PyModule is only part of the prelude because we need it for the pymodule function
    pub use crate::objects::{PyAny, PyModule};
    #[cfg(feature = "macros")]
    pub use pyo3cls::{pyclass, pyfunction, pymethods, pymodule, pyproto, FromPyObject};
}

use crate::Python;

/// Conversion trait that allows various objects to be converted into `PyObject`.
pub trait ToPyObject {
    /// Converts self into a Python object.
    fn to_object<'py>(&self, py: Python<'py>) -> PyObject<'py>;
}

impl<T> ToPyObject for T
where
    T: crate::ToPyObject,
{
    /// Converts self into a Python object.
    fn to_object<'py>(&self, py: Python<'py>) -> PyObject<'py> {
        use crate::IntoPyPointer;
        unsafe {
            PyObject::from_raw_or_panic(
                py,
                <Self as crate::ToPyObject>::to_object(self, py).into_ptr(),
            )
        }
    }
}

type PyObject<'py> = objects::PyAny<'py>;
