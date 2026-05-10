use pyo3::prelude::*;
use std::fmt::{Display, Formatter};

#[pyclass(extend=pyo3::types::PyDict)]
//~^ ERROR: expected one of: `crate`, `dict`, `eq`, `eq_int`, `extends`, `freelist`, `frozen`, `get_all`, `hash`, `immutable_type`, `mapping`, `module`, `name`, `ord`, `rename_all`, `sequence`, `set_all`, `new`, `str`, `subclass`, `unsendable`, `weakref`, `generic`, `from_py_object`, `skip_from_py_object`
struct TypoIntheKey {}

#[pyclass(extends = "PyDict")]
//~^ ERROR: expected identifier
struct InvalidExtends {}

#[pyclass(name = m::MyClass)]
//~^ ERROR: expected string literal
struct InvalidName {}

#[pyclass(name = "Custom Name")]
//~^ ERROR: expected a single identifier in double quotes
struct InvalidName2 {}

#[pyclass(name = CustomName)]
//~^ ERROR: expected string literal
struct DeprecatedName {}

#[pyclass(rename_all = camelCase)]
//~^ ERROR: expected string literal
struct InvalidRenamingRule {}

#[pyclass(rename_all = "Camel-Case")]
//~^ ERROR: expected a valid renaming rule, possible values are: "camelCase", "kebab-case", "lowercase", "PascalCase", "SCREAMING-KEBAB-CASE", "SCREAMING_SNAKE_CASE", "snake_case", "UPPERCASE"
struct InvalidRenamingRule2 {}

#[pyclass(module = my_module)]
//~^ ERROR: expected string literal
struct InvalidModule {}

#[pyclass(weakrev)]
//~^ ERROR: expected one of: `crate`, `dict`, `eq`, `eq_int`, `extends`, `freelist`, `frozen`, `get_all`, `hash`, `immutable_type`, `mapping`, `module`, `name`, `ord`, `rename_all`, `sequence`, `set_all`, `new`, `str`, `subclass`, `unsendable`, `weakref`, `generic`, `from_py_object`, `skip_from_py_object`
struct InvalidArg {}

#[pyclass(mapping, sequence)]
struct CannotBeMappingAndSequence {}
//~^ ERROR: a `#[pyclass]` cannot be both a `mapping` and a `sequence`

#[pyclass(eq)]
//~^ ERROR: binary operation `==` cannot be applied to type `&EqOptRequiresEq`
//~| ERROR: binary operation `!=` cannot be applied to type `&EqOptRequiresEq`
struct EqOptRequiresEq {}

#[pyclass(eq)]
//~^ ERROR: duplicate definitions with name `__pymethod___richcmp____`
//~| ERROR: multiple applicable items in scope
#[derive(PartialEq)]
struct EqOptAndManualRichCmp {}

#[pymethods]
//~^ ERROR: multiple applicable items in scope
impl EqOptAndManualRichCmp {
    fn __richcmp__(
        &self,
        _py: Python,
        _other: Bound<'_, PyAny>,
        _op: pyo3::pyclass::CompareOp,
    ) -> PyResult<Py<PyAny>> {
        todo!()
    }
}

#[pyclass(eq_int)]
//~^ ERROR: `eq_int` can only be used on simple enums.
struct NoEqInt {}

#[pyclass(frozen, eq, hash)]
//~^ ERROR: the trait bound `HashOptRequiresHash: Hash` is not satisfied
#[derive(PartialEq)]
struct HashOptRequiresHash;

#[pyclass(hash)]
//~^ ERROR: The `hash` option requires the `frozen` option.
//~| ERROR: The `hash` option requires the `eq` option.
#[derive(Hash)]
struct HashWithoutFrozenAndEq;

#[pyclass(frozen, eq, hash)]
//~^ ERROR: duplicate definitions with name `__pymethod___hash____`
//~| ERROR: multiple applicable items in scope
#[derive(PartialEq, Hash)]
struct HashOptAndManualHash {}

#[pymethods]
//~^ ERROR: multiple applicable items in scope
impl HashOptAndManualHash {
    fn __hash__(&self) -> u64 {
        todo!()
    }
}

#[pyclass(ord)]
//~^ ERROR: The `ord` option requires the `eq` option.
struct InvalidOrderedStruct {
    inner: i32,
}

#[pyclass]
struct MultipleErrors {
    #[pyo3(foo)]
    //~^ ERROR: expected one of: `get`, `set`, `name`
    #[pyo3(blah)]
    //~^ ERROR: expected one of: `get`, `set`, `name`
    x: i32,
    #[pyo3(pop)]
    //~^ ERROR: expected one of: `get`, `set`, `name`
    y: i32,
}

#[pyclass(str)]
//~^ ERROR: duplicate definitions with name `__pymethod___str____`
//~| ERROR: multiple applicable items in scope
struct StrOptAndManualStr {}

impl Display for StrOptAndManualStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[pymethods]
//~^ ERROR: multiple applicable items in scope
impl StrOptAndManualStr {
    fn __str__(&self) -> String {
        todo!()
    }
}

#[pyclass(str = "{")]
//~^ ERROR: invalid format string: expected `}` but string was terminated
#[derive(PartialEq)]
struct Coord(u32, u32, u32);

#[pyclass(str = "{$}")]
//~^ ERROR: invalid format string: expected `}`, found `$`
#[derive(PartialEq)]
struct Coord2(u32, u32, u32);

#[pyclass(str = "X: {aaaa}, Y: {y}, Z: {z}", skip_from_py_object)]
//~^ ERROR: no field `aaaa` on type `&Point`
#[derive(PartialEq, Eq, Clone, PartialOrd)]
pub struct Point {
    x: i32,
    y: i32,
    z: i32,
}

#[pyclass(str = "X: {x}, Y: {y}}}, Z: {zzz}", skip_from_py_object)]
//~^ ERROR: no field `zzz` on type `&Point2`
#[derive(PartialEq, Eq, Clone, PartialOrd)]
pub struct Point2 {
    x: i32,
    y: i32,
    z: i32,
}

#[pyclass(str = "{0}, {162543}, {2}")]
//~^ ERROR: no field `162543` on type `&Coord3`
#[derive(PartialEq)]
struct Coord3(u32, u32, u32);

#[pyclass(name = "aaa", str = "unsafe: {unsafe_variable}")]
//~^ ERROR: The format string syntax is incompatible with any renaming via `name` or `rename_all`
struct StructRenamingWithStrFormatter {
    #[pyo3(name = "unsafe", get, set)]
    unsafe_variable: usize,
}

#[pyclass(name = "aaa", str = "unsafe: {unsafe_variable}")]
//~^ ERROR: The format string syntax is incompatible with any renaming via `name` or `rename_all`
struct StructRenamingWithStrFormatter2 {
    unsafe_variable: usize,
}

#[pyclass(str = "unsafe: {unsafe_variable}")]
//~^ ERROR: The format string syntax is incompatible with any renaming via `name` or `rename_all`
struct StructRenamingWithStrFormatter3 {
    #[pyo3(name = "unsafe", get, set)]
    unsafe_variable: usize,
}

#[pyclass(rename_all = "SCREAMING_SNAKE_CASE", str = "{a_a}, {b_b}, {c_d_e}")]
//~^ ERROR: The format string syntax is incompatible with any renaming via `name` or `rename_all`
struct RenameAllVariantsStruct {
    a_a: u32,
    b_b: u32,
    c_d_e: String,
}

#[pyclass(str = "{:?}")]
//~^ ERROR: No member found, you must provide a named or positionally specified member.
#[derive(Debug)]
struct StructWithNoMember {
    a: String,
    b: String,
}

#[pyclass(str = "{}")]
//~^ ERROR: No member found, you must provide a named or positionally specified member.
#[derive(Debug)]
struct StructWithNoMember2 {
    a: String,
    b: String,
}

#[pyclass(eq, str = "Stuff...")]
//~^ ERROR: The format string syntax cannot be used with enums
#[derive(Debug, PartialEq)]
pub enum MyEnumInvalidStrFmt {
    Variant,
    OtherVariant,
}

impl Display for MyEnumInvalidStrFmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass(from_py_object, skip_from_py_object)]
//~^ ERROR: `skip_from_py_object` and `from_py_object` are mutually exclusive
struct StructTooManyFromPyObject {
    a: String,
    b: String,
}

#[pyclass(from_py_object)]
//~^ ERROR: the trait bound `StructFromPyObjectNoClone: Clone` is not satisfied
struct StructFromPyObjectNoClone {
    a: String,
    b: String,
}

#[pyclass]
#[derive(Clone)]
struct StructImplicitFromPyObjectDeprecated {
    a: String,
    b: String,
}

#[pyclass(new = "from_fields")]
struct NonPythonField {
    field: Box<dyn std::error::Error + Send + Sync>,
    //~^ ERROR: `Box<dyn std::error::Error + Send + Sync>` cannot be used as a Python function argument
    //~| ERROR: `Box<dyn std::error::Error + Send + Sync>` cannot be used as a Python function argument
    //~| ERROR: the trait bound `dyn std::error::Error + Send + Sync: Clone` is not satisfied
}

#[pyclass(new = "from_fields")]
//~^ ERROR: conflicting implementations of trait `pyo3::impl_::pyclass::doc::PyClassNewTextSignature` for type `NewFromFieldsWithManualNew`
//~| ERROR: duplicate definitions with name `__pymethod___new____`
//~| ERROR: multiple applicable items in scope
struct NewFromFieldsWithManualNew {
    field: i32,
}

#[pymethods]
//~^ ERROR: multiple applicable items in scope
impl NewFromFieldsWithManualNew {
    #[new]
    fn new(field: i32) -> Self {
        Self { field }
    }
}

fn main() {}
