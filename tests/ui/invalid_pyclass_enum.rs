use pyo3::prelude::*;

#[pyclass(subclass)]
enum NotBaseClass {
    x,
    y,
}

#[pyclass(extends = PyList)]
enum NotDrivedClass {
    x,
    y,
}

#[pyclass]
enum NoEmptyEnum {}

fn main() {}
