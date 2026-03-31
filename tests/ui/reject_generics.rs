use pyo3::prelude::*;

#[pyclass]
struct ClassWithGenerics<A> {
//~^ ERROR: #[pyclass] cannot have generic parameters. For an explanation, see https://pyo3.rs/v0.28.2/class.html#no-generic-parameters
    a: A,
}

#[pyclass]
struct ClassWithLifetimes<'a> {
//~^ ERROR: #[pyclass] cannot have lifetime parameters. For an explanation, see https://pyo3.rs/v0.28.2/class.html#no-lifetime-parameters
    a: &'a str,
}

fn main() {}
