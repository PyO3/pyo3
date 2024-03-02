use pyo3::prelude::*;

#[pyclass]
enum ComplexEnum {
    Int { int: i32 },
    Str { string: String },
}

#[pymethods]
impl ComplexEnum {
    fn mutate_in_place(&mut self) {
        *self = match self {
            ComplexEnum::Int { int } => ComplexEnum::Str { string: int.to_string() },
            ComplexEnum::Str { string } => ComplexEnum::Int { int: string.len() as i32 },
        }
    }
}

fn main() {}
