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
//~^ ERROR: binary operation `==` cannot be applied to type `&SimpleEqOptRequiresPartialEq`
//~| ERROR: binary operation `!=` cannot be applied to type `&SimpleEqOptRequiresPartialEq`
enum SimpleEqOptRequiresPartialEq {
    A,
    B,
}

#[pyclass(eq)]
//~^ ERROR: binary operation `==` cannot be applied to type `&ComplexEqOptRequiresPartialEq`
//~| ERROR: binary operation `!=` cannot be applied to type `&ComplexEqOptRequiresPartialEq`
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
//~^ ERROR: the trait bound `SimpleHashOptRequiresHash: Hash` is not satisfied
#[derive(PartialEq)]
enum SimpleHashOptRequiresHash {
    A,
    B,
}

#[pyclass(frozen, eq, hash)]
//~^ ERROR: the trait bound `ComplexHashOptRequiresHash: Hash` is not satisfied
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
    VariantA (i32),
    VariantB { msg: String }
}

#[pyclass(eq,ord)]
//~^ ERROR: binary operation `>` cannot be applied to type `&InvalidOrderedComplexEnum2`
//~| ERROR: binary operation `<` cannot be applied to type `&InvalidOrderedComplexEnum2`
//~| ERROR: binary operation `<=` cannot be applied to type `&InvalidOrderedComplexEnum2`
//~| ERROR: binary operation `>=` cannot be applied to type `&InvalidOrderedComplexEnum2`
#[derive(PartialEq)]
enum InvalidOrderedComplexEnum2 {
    VariantA (i32),
    VariantB { msg: String }
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
