use pyo3::prelude::*;

#[pyclass(extend=pyo3::types::PyDict)]
struct TypoIntheKey {}

#[pyclass(extends = "PyDict")]
struct InvalidExtends {}

#[pyclass(name = m::MyClass)]
struct InvalidName {}

#[pyclass(module = my_module)]
struct InvalidModule {}

#[pyclass(weakrev)]
struct InvalidArg {}

fn main() {}
