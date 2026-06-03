#![deny(deprecated)]

use pyo3::prelude::*;

#[pyclass(subclass)]
struct Base;

#[pyclass(extends=Base)]
struct Sub1;

#[pymethods]
impl Sub1 {
    #[new]
    fn new() -> (Sub1, Base) {
        //~^ERROR: use of deprecated method `pyo3::internal::pyclass_init::TpNewTupleResolver::<S, (S, B)>::resolve`: Tuple syntax for super class initialization is phased out. Use `PyClassInitializer` instead.
        (Sub1, Base)
    }
}

#[pyclass(extends=Base)]
struct Sub2;

#[pymethods]
impl Sub2 {
    #[new]
    fn new() -> PyResult<(Sub2, Base)> {
        //~^ERROR: use of deprecated method `pyo3::internal::pyclass_init::TpNewTupleResolver::<S, (S, B)>::resolve`: Tuple syntax for super class initialization is phased out. Use `PyClassInitializer` instead.
        Ok((Sub2, Base))
    }
}

fn main() {}
