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

#[pyclass(eq, eq_int)]
enum SimpleEqOptRequiresPartialEq {
    A,
    B,
}

#[pyclass(eq)]
enum ComplexEqOptRequiresPartialEq {
    A(i32),
    B { msg: String },
}

#[pyclass(eq_int)]
enum NoEqInt {
    A(i32),
    B { msg: String },
}

#[pyclass(frozen, eq, eq_int, hash)]
#[derive(PartialEq)]
enum SimpleHashOptRequiresHash {
    A,
    B,
}

#[pyclass(frozen, eq, hash)]
#[derive(PartialEq)]
enum ComplexHashOptRequiresHash {
    A(i32),
    B { msg: String },
}

#[pyclass(hash)]
#[derive(Hash)]
enum SimpleHashOptRequiresFrozenAndEq {
    A,
    B,
}

#[pyclass(hash)]
#[derive(Hash)]
enum ComplexHashOptRequiresEq {
    A(i32),
    B { msg: String },
}

fn main() {}
