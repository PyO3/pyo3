//@revisions: default nostd
//@[default] with-std
//@[nostd] no-std

use pyo3::prelude::*;

#[pyclass(subclass)]
//~^ ERROR: enums can't be inherited by other classes
enum NotBaseClass {
    X,
    Y,
}

#[pyclass(extends = PyList)]
//~^ ERROR: enums can't extend from other classes
enum NotDerivedClass {
    X,
    Y,
}

#[pyclass]
enum NoEmptyEnum {}
//~^ ERROR: #[pyclass] can't be used on enums without any variants

#[pyclass]
enum NoUnitVariants {
    StructVariant { field: i32 },
    UnitVariant,
    //~^ ERROR: Unit variant `UnitVariant` is not yet supported in a complex enum
}

#[pyclass]
enum SimpleNoSignature {
    #[pyo3(constructor = (a, b))]
    //~^ ERROR: `constructor` can't be used on a simple enum variant
    A,
    B,
}

#[pyclass(eq, eq_int)]
//~^ ERROR: can't compare `SimpleEqOptRequiresPartialEq` with `SimpleEqOptRequiresPartialEq`
enum SimpleEqOptRequiresPartialEq {
    A,
    B,
}

#[pyclass(eq)]
//~^ ERROR: can't compare `ComplexEqOptRequiresPartialEq` with `ComplexEqOptRequiresPartialEq`
enum ComplexEqOptRequiresPartialEq {
    A(i32),
    B { msg: String },
}

#[pyclass(eq_int)]
//~^ ERROR: The `eq_int` option requires the `eq` option.
enum SimpleEqIntWithoutEq {
    A,
    B,
}

#[pyclass(eq_int)]
//~^ ERROR: `eq_int` can only be used on simple enums.
enum NoEqInt {
    A(i32),
    B { msg: String },
}

#[pyclass(frozen, eq, eq_int, hash)]
//~[default]^ ERROR: the trait bound `SimpleHashOptRequiresHash: Hash` is not satisfied
//~[nostd]| ERROR: `#[pyclass(hash)]` requires PyO3's `std` feature
#[derive(PartialEq)]
enum SimpleHashOptRequiresHash {
    A,
    B,
}

#[pyclass(frozen, eq, hash)]
//~[default]^ ERROR: the trait bound `ComplexHashOptRequiresHash: Hash` is not satisfied
//~[nostd]| ERROR: `#[pyclass(hash)]` requires PyO3's `std` feature
#[derive(PartialEq)]
enum ComplexHashOptRequiresHash {
    A(i32),
    B { msg: String },
}

#[pyclass(hash)]
//~^ ERROR: The `hash` option requires the `frozen` option.
//~| ERROR: The `hash` option requires the `eq` option.
#[derive(Hash)]
enum SimpleHashOptRequiresFrozenAndEq {
    A,
    B,
}

#[pyclass(hash)]
//~^ ERROR: The `hash` option requires the `eq` option.
#[derive(Hash)]
enum ComplexHashOptRequiresEq {
    A(i32),
    B { msg: String },
}

#[pyclass(ord)]
//~^ ERROR: The `ord` option requires the `eq` option.
enum InvalidOrderedComplexEnum {
    VariantA(i32),
    VariantB { msg: String },
}

#[pyclass(eq, ord)]
//~^ ERROR: can't compare `InvalidOrderedComplexEnum2` with `InvalidOrderedComplexEnum2`
#[derive(PartialEq)]
enum InvalidOrderedComplexEnum2 {
    VariantA(i32),
    VariantB { msg: String },
}

#[pyclass(eq)]
#[derive(PartialEq)]
enum AllEnumVariantsDisabled {
    //~^ ERROR: #[pyclass] can't be used on enums without any variants - all variants of enum `AllEnumVariantsDisabled` have been configured out by cfg attributes
    #[cfg(any())]
    DisabledA,
    #[cfg(not(all()))]
    DisabledB,
}

fn main() {}
