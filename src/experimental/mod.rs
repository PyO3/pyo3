pub mod owned;
pub mod objects;

pub use objects::{FromPyObject, PyTryFrom, PyNativeObject};

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
    pub use crate::types::{PyAny, PyModule};
    #[cfg(feature = "macros")]
    pub use pyo3cls::{pyclass, pyfunction, pymethods, pymodule, pyproto, FromPyObject};
}
