use pyo3::prelude::*;

#[pyclass(set_all)]
struct Foo;

#[pyclass(set_all)]
struct Foo2{
    #[pyo3(set)]
    field: u8,
}

#[pyclass(get_all)]
struct Foo3;

#[pyclass(get_all)]
struct Foo4{
    #[pyo3(get)]
    field: u8,
}

fn main() {}
