use pyo3::prelude::*;

#[pyclass]
struct ClassWithGenerics<A> {
    a: A,
}

fn main() {}
