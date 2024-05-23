use pyo3::prelude::*;

#[pyclass(subclass)]
enum NotBaseClass {
    X,
    Y,
}

#[pyclass(extends = PyList)]
enum NotDrivedClass {
    X,
    Y,
}

#[pyclass]
enum NoEmptyEnum {}

#[pyclass]
enum NoUnitVariants {
    StructVariant { field: i32 },
    UnitVariant,
}

#[pyclass]
enum SimpleNoSignature {
    #[pyo3(constructor = (a, b))]
    A,
    B,
}

#[pyclass(ord)]
enum InvalidOrderedComplexEnum {
    VariantA (i32),
    VariantB { msg: String }
}

fn main() {}
