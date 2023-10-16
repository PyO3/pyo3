use pyo3::prelude::*;

#[pyclass]
struct ClassWithGenerics<A> {
    a: A,
}

#[pyclass]
struct ClassWithLifetimes<'a> {
    a: &'a str,
}

fn main() {}
