use pyo3::prelude::*;

#[pyclass]
struct ClassWithGenerics<A> {
    //~^ ERROR: #[pyclass] cannot have generic parameters. For an explanation, see
    a: A,
}

#[pyclass]
struct ClassWithLifetimes<'a> {
    //~^ ERROR: #[pyclass] cannot have lifetime parameters. For an explanation, see
    a: &'a str,
}

fn main() {}
