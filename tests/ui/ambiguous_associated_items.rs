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

#[pyclass(from_py_object)]
#[derive(Clone)]
pub enum SimpleItemsFromPyObject {
    Error,
    Output,
    Target,
}

#[pyclass(from_py_object)]
#[derive(Clone)]
pub enum ComplexItemsFromPyObject {
    Error(i32),
    Output(i32),
    Target(i32),
}

#[derive(FromPyObject, Clone)]
enum DeriveItemsFromPyObject {
    Error(i32),
    Output(i32),
    Target(i32),
}

fn main() {}
