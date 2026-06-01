use pyo3::prelude::*;

#[pyclass]
enum ComplexEnum {
    Int { int: i32 },
    Str { string: String },
}

#[pymethods]
//~^ ERROR: type mismatch resolving `<ComplexEnum as PyClass>::Frozen == False`
impl ComplexEnum {
    fn mutate_in_place(&mut self) {
//~^ ERROR: type mismatch resolving `<ComplexEnum as PyClass>::Frozen == False`
        *self = match self {
            ComplexEnum::Int { int } => ComplexEnum::Str { string: int.to_string() },
            ComplexEnum::Str { string } => ComplexEnum::Int { int: string.len() as i32 },
        }
    }
}

#[pyclass]
enum TupleEnum {
    Int(i32),
    Str(String),
}

#[pymethods]
//~^ ERROR: type mismatch resolving `<TupleEnum as PyClass>::Frozen == False`
impl TupleEnum {
    fn mutate_in_place(&mut self) {
//~^ ERROR: type mismatch resolving `<TupleEnum as PyClass>::Frozen == False`
        *self = match self {
            TupleEnum::Int(int) => TupleEnum::Str(int.to_string()),
            TupleEnum::Str(string) => TupleEnum::Int(string.len() as i32),
        }
    }
}

fn main() {}
