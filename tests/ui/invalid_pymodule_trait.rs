use pyo3::prelude::*;

#[pymodule]
mod module {
    #[pymodule_export]
    trait Foo {}
}

fn main() {}
