use pyo3::prelude::*;

#[pyclass(generic)]
//~^ ERROR: enums do not support #[pyclass(generic)]
enum NotGenericForEnum {
    A,
    B,
}

#[pyclass(generic)]
//~^ ERROR: enums do not support #[pyclass(generic)]
enum NoGenericForComplexEnum {
    A { x: f64 },
    B { y: f64, z: f64 },
}

fn main() {}
