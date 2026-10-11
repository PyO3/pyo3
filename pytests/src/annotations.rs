//! Example of custom annotations.

use pyo3::prelude::*;

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
}
