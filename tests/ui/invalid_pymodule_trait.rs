use pyo3::prelude::*;

#[pymodule]
mod module {
    #[pymodule_export]
//~^ ERROR: cannot find attribute `pymodule_export` in this scope
//~| ERROR: `#[pymodule_export]` may only be used on `use` or `const` statements
    trait Foo {}
}

fn main() {}
