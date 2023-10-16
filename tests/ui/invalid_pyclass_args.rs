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

#[pyclass(rename_all = camelCase)]
struct InvalidRenamingRule {}

#[pyclass(rename_all = "Camel-Case")]
struct InvalidRenamingRule2 {}

#[pyclass(module = my_module)]
struct InvalidModule {}

#[pyclass(weakrev)]
struct InvalidArg {}

#[pyclass(mapping, sequence)]
struct CannotBeMappingAndSequence {}

fn main() {}
