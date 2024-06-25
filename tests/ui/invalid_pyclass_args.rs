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

#[pyclass(ord)]
struct InvalidOrderedStruct {
    inner: i32
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

#[pyclass(str = "X: {aaaa}, Y: {y}, Z: {z}")]
#[derive(PartialEq, Eq, Clone, PartialOrd)]
pub struct Point {
    x: i32,
    y: i32,
    z: i32,
}

#[pyclass(str = "X: {x}, Y: {y}}}, Z: {zzz}")]
#[derive(PartialEq, Eq, Clone, PartialOrd)]
pub struct Point2 {
    x: i32,
    y: i32,
    z: i32,
}

#[pyclass(str = "{0}, {162543}, {2}")]
#[derive(PartialEq)]
struct Coord3(u32, u32, u32);

#[pyclass(name = "aaa", str="unsafe: {unsafe_variable}")]
struct StructRenamingWithStrFormatter {
    #[pyo3(name = "unsafe", get, set)]
    unsafe_variable: usize,
}

#[pyclass(name = "aaa", str="unsafe: {unsafe_variable}")]
struct StructRenamingWithStrFormatter2 {
    unsafe_variable: usize,
}

#[pyclass(str="unsafe: {unsafe_variable}")]
struct StructRenamingWithStrFormatter3 {
    #[pyo3(name = "unsafe", get, set)]
    unsafe_variable: usize,
}

#[pyclass(eq, eq_int, rename_all = "SCREAMING_SNAKE_CASE", str="{:?}")]
#[derive(Debug, PartialEq, Eq, Clone)]
enum RenameAllVariantsEnumWithStrFormatter {
    VariantOne,
    VariantTwo,
    VariantFour,
}

#[pyclass(eq, eq_int, str="MyEnum.{:?}")]
#[derive(Debug, PartialEq)]
pub enum RenamedEnumWithStrFormatter {
    #[pyo3(name = "AwesomeVariant")]
    Variant,
    OtherVariant,
}

#[pyclass(name="zzz", str = "{:?}")]
#[derive(PartialEq, Debug)]
enum RenamedComplexEnumWithStr {
    A(u32),
    B { msg: String },
}

fn main() {}
