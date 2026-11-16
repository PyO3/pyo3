//! Example of custom annotations.

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
    /// Public in the stubs of `pyo3_pytests.annotations`, which exports it
    pub SupportsClose {
        fn close(&self);
    }
}

pyo3::pyprotocol! {
    /// Not referenced by any annotation, so left out of the stubs
    pub SupportsUnused {
        fn unused(&self);
    }
}

#[pymodule]
pub mod annotations {
    use super::SupportsReadWrite;
    use crate::pyclasses::EmptyClass;
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyTuple};

    #[pymodule_export]
    use super::SupportsClose;

    #[pyfunction(signature = (
        a: collections.abc.Iterable[int | None],
        b: typing.Literal["foo", 1, -2, True] | None = None,
        c: collections.abc.Callable[[str, tuple[int, ...]], tuple[()]] | None = None,
        *_args: { <pyo3::types::PyInt as pyo3::type_object::PyTypeInfo>::TYPE_HINT },
        **_kwargs: (bool | str)
    ) -> collections.abc.Iterable[int | None])]
    fn with_custom_type_annotations<'py>(
        a: Bound<'py, PyAny>,
        b: Option<Bound<'py, PyAny>>,
        c: Option<Bound<'py, PyAny>>,
        _args: Bound<'py, PyTuple>,
        _kwargs: Option<Bound<'py, PyDict>>,
    ) -> Bound<'py, PyAny> {
        let _ = (b, c);
        a
    }

    #[pyfunction]
    fn cross_module_imports(_a: &EmptyClass) {}

    #[pyfunction(signature = (f: { SupportsReadWrite::TYPE_HINT }, data))]
    fn write_to(f: Bound<'_, PyAny>, data: &[u8]) -> PyResult<usize> {
        f.call_method1("write", (data,))?.extract()
    }
}
