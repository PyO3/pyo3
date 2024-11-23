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
    Error(PyObject),
    Output(PyObject),
    Target(PyObject),
}

#[derive(IntoPyObject)]
enum DeriveItems {
    Error(PyObject),
    Output(PyObject),
    Target(PyObject),
}

fn main() {}
