#![deny(deprecated)]
use pyo3::prelude::*;

#[pyfunction]
fn from_py_with_in_function(
    #[pyo3(from_py_with = "Bound::<'_, PyAny>::len")] argument: usize,
) -> usize {
    argument
}

#[pyclass]
struct Number(usize);

#[pymethods]
impl Number {
    #[new]
    fn from_py_with_in_method(
        #[pyo3(from_py_with = "Bound::<'_, PyAny>::len")] value: usize,
    ) -> Self {
        Self(value)
    }
}

#[derive(FromPyObject)]
struct FromPyWithStruct {
    #[pyo3(from_py_with = "Bound::<'_, PyAny>::len")]
    len: usize,
    other: usize,
}

#[derive(FromPyObject)]
struct FromPyWithTuple(
    #[pyo3(from_py_with = "Bound::<'_, PyAny>::len")] usize,
    usize,
);

fn main() {}
