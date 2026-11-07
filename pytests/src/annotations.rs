//! Example of custom annotations.

use pyo3::inspect::PyStaticExpr;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

pyo3::pyprotocol! {
    /// A binary stream that can be read from and written to
    pub SupportsReadWrite {
        #[pyo3(signature = (n = -1, /))]
        fn read(&self, n: isize) -> Py<PyBytes>;
        fn write(&self, data: &[u8]) -> usize;
    }
}

pyo3::pyprotocol! {
    /// Not referenced by any annotation, so left out of the stubs
    pub SupportsUnused {
        fn unused(&self);
    }
}

/// Any object with `read()` and `write()` methods, typed as the `SupportsReadWrite` protocol
pub struct ReadWriteFile<'py>(Bound<'py, PyAny>);

impl<'py> FromPyObject<'_, 'py> for ReadWriteFile<'py> {
    type Error = PyErr;

    const INPUT_TYPE: PyStaticExpr = SupportsReadWrite::TYPE_HINT;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_owned()))
    }
}

#[pymodule]
pub mod annotations {
    use crate::pyclasses::EmptyClass;
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyTuple};

    #[pyfunction(signature = (a: "list[int]", *_args: "str", _b: "int | None" = None, **_kwargs: "bool") -> "int")]
    fn with_custom_type_annotations<'py>(
        a: Bound<'py, PyAny>,
        _args: Bound<'py, PyTuple>,
        _b: Option<Bound<'py, PyAny>>,
        _kwargs: Option<Bound<'py, PyDict>>,
    ) -> Bound<'py, PyAny> {
        a
    }

    #[pyfunction]
    fn cross_module_imports(_a: &EmptyClass) {}

    #[pyfunction]
    fn write_to(f: super::ReadWriteFile<'_>, data: &[u8]) -> PyResult<usize> {
        f.0.call_method1("write", (data,))?.extract()
    }
}
