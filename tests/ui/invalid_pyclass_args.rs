use pyo3::prelude::*;

#[pyclass(extend=pyo3::types::PyDict)]
struct TypoIntheKey {}

#[pyclass(extends = "PyDict")]
struct InvalidExtends {}

#[pyclass(name = m::MyClass)]
struct InvalidName {}

#[pyclass(name = "Custom Name")]
struct InvalidName2 {}

#[pyclass(name = CustomName)]
struct DeprecatedName {}

#[pyclass(module = my_module)]
struct InvalidModule {}

#[pyclass(weakrev)]
struct InvalidArg {}

#[pyclass(mapping, sequence)]
struct CannotBeMappingAndSequence {}

fn main() {}
