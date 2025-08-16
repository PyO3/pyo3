use pyo3::prelude::*;

#[pyclass(eq)]
#[derive(PartialEq)]
pub enum SimpleItems {
    Error,
    Output,
    Target,
}

#[pyclass]
pub enum ComplexItems {
    Error(Py<PyAny>),
    Output(Py<PyAny>),
    Target(Py<PyAny>),
}

#[derive(IntoPyObject)]
enum DeriveItems {
    Error(Py<PyAny>),
    Output(Py<PyAny>),
    Target(Py<PyAny>),
}

fn main() {}
