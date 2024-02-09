use pyo3::prelude::*;

#[pymodule]
mod module {
    #[pyo3]
    trait Foo {}
}

fn main() {}
