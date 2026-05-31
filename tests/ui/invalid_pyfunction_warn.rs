use pyo3::prelude::*;

#[pyfunction]
#[pyo3(warn)]
//~^ ERROR: unexpected end of input, expected parentheses
fn no_parenthesis_deprecated() {}

#[pyfunction]
#[pyo3(warn())]
//~^ ERROR: missing `message` in `warn` attribute
fn no_message_deprecated() {}

#[pyfunction]
#[pyo3(warn(category = pyo3::exceptions::PyDeprecationWarning))]
//~^ ERROR: missing `message` in `warn` attribute
fn no_message_deprecated_with_category() {}

#[pyfunction]
#[pyo3(warn(category = pyo3::exceptions::PyDeprecationWarning, message = ,))]
//~^ ERROR: expected string literal
fn empty_message_deprecated_with_category() {}

#[pyfunction]
#[pyo3(warn(message = "deprecated function", category = ,))]
//~^ ERROR: expected identifier
fn empty_category_deprecated_with_message() {}

#[pyfunction]
#[pyo3(warn(message = "deprecated function", random_key))]
//~^ ERROR: expected `message` or `category`
fn random_key_deprecated() {}

#[pyclass]
struct DeprecatedMethodContainer {}

#[pymethods]
impl DeprecatedMethodContainer {
    #[classattr]
    #[pyo3(warn(message = "deprecated class attr"))]
//~^ ERROR: #[classattr] cannot be used with #[pyo3(warn)]
    fn deprecated_class_attr() -> i32 {
        5
    }
}

#[pymethods]
impl DeprecatedMethodContainer {
    #[pyo3(warn(message = "deprecated __traverse__"))]
//~^ ERROR: __traverse__ cannot be used with #[pyo3(warn)]
    fn __traverse__(&self, _visit: pyo3::gc::PyVisit<'_>) -> Result<(), pyo3::PyTraverseError> {
        Ok(())
    }
}

fn main() {}
