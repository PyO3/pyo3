use std::fmt::{Display, Formatter};
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

#[pyclass(eq)]
struct EqOptRequiresEq {}

#[pyclass(eq)]
#[derive(PartialEq)]
struct EqOptAndManualRichCmp {}

#[pymethods]
impl EqOptAndManualRichCmp {
    fn __richcmp__(
        &self,
        _py: Python,
        _other: Bound<'_, PyAny>,
        _op: pyo3::pyclass::CompareOp,
    ) -> PyResult<PyObject> {
        todo!()
    }
}

#[pyclass(eq_int)]
struct NoEqInt {}

#[pyclass(frozen, eq, hash)]
#[derive(PartialEq)]
struct HashOptRequiresHash;

#[pyclass(hash)]
#[derive(Hash)]
struct HashWithoutFrozenAndEq;

#[pyclass(frozen, eq, hash)]
#[derive(PartialEq, Hash)]
struct HashOptAndManualHash {}

#[pymethods]
impl HashOptAndManualHash {
    fn __hash__(&self) -> u64 {
        todo!()
    }
}

#[pyclass(str)]
struct StrOptAndManualStr {}

impl Display for StrOptAndManualStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[pymethods]
impl StrOptAndManualStr {
    fn __str__(
        &self,
    ) -> String {
        todo!()
    }
}

#[pyclass(str = "{")]
#[derive(PartialEq)]
struct Coord(u32, u32, u32);

#[pyclass(str = "{$}")]
#[derive(PartialEq)]
struct Coord2(u32, u32, u32);

fn main() {}
