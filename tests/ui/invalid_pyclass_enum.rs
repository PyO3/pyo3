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

#[pyclass(eq)]
enum SimpleEqOptRequiresPartialEq {
    A,
    B,
}

#[pyclass(eq)]
enum ComplexEqOptRequiresPartialEq {
    A(i32),
    B { msg: String },
}

fn main() {}
