//! Example of custom annotations.

use pyo3::prelude::*;

#[pymodule(stubs = {
    from datetime import datetime as dt, time
    from uuid import UUID
})]
pub mod annotations {
    use pyo3::prelude::*;
    use pyo3::types::{PyDate, PyDateTime, PyDict, PyTime, PyTuple};

    #[pyfunction(signature = (a: "dt | time | UUID", *_args: "str", _b: "int | None" = None, **_kwargs: "bool") -> "int")]
    fn with_custom_type_annotations<'py>(
        a: Bound<'py, PyAny>,
        _args: Bound<'py, PyTuple>,
        _b: Option<Bound<'py, PyAny>>,
        _kwargs: Option<Bound<'py, PyDict>>,
    ) -> Bound<'py, PyAny> {
        a
    }

    #[pyfunction]
    fn with_built_in_type_annotations(
        _date_time: Bound<'_, PyDateTime>,
        _time: Bound<'_, PyTime>,
        _date: Bound<'_, PyDate>,
    ) {
    }
}
