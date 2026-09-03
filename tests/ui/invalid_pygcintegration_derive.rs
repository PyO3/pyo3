use pyo3::prelude::*;
use pyo3::PyGcTraversable;

struct NotTraversable;

struct Traversable;

unsafe impl PyGcTraversable for Traversable {
    const MAY_CONTAIN_CYCLES: bool = false;

    fn traverse(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        Ok(())
    }

    fn clear(&mut self) {}
}

#[derive(PyGcTraversable)]
//~^ ERROR: the trait bound `NotTraversable: pyo3::PyGcTraversable` is not satisfied
struct MissingFieldImpl {
    field: NotTraversable,
}

#[derive(PyGcTraversable)]
//~^ ERROR: the trait bound `NotTraversable: pyo3::PyGcTraversable` is not satisfied
struct InvalidGcTrue {
    #[pyo3(gc = true)]
    field: NotTraversable,
}

#[derive(PyGcTraversable)]
struct InvalidGcValue {
    #[pyo3(gc = "false")]
    //~^ ERROR: expected boolean literal
    field: NotTraversable,
}

#[derive(PyGcTraversable)]
//~^ ERROR: evaluation panicked: `#[pyo3(gc = false)]` may not be used on fields which implement `PyGcTraversable`
struct InvalidGcFalseOnIntegrated {
    #[pyo3(gc = false)]
    field: Traversable,
}

#[derive(PyGcTraversable)]
struct DuplicateGc {
    #[pyo3(gc = false, gc = false)]
    //~^ ERROR: `gc` may only be specified once
    field: NotTraversable,
}

#[derive(PyGcTraversable)]
struct UnsupportedFieldAttribute {
    #[pyo3(attribute)]
    //~^ ERROR: expected `gc`
    field: NotTraversable,
}

fn main() {}
