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

fn main() {}
