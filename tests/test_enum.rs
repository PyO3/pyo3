use pyo3::prelude::*;
use pyo3::{py_run, wrap_pyfunction};

mod common;

#[pyenum]
pub enum MyEnum {
    Variant = 1,
    OtherVariant = 2,
}

#[test]
fn test_reflexive() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let mynum = py.get_type::<MyEnum>();
    py_assert!(py, mynum, "mynum.Variant == mynum.Variant");
    py_assert!(py, mynum, "mynum.OtherVariant == mynum.OtherVariant");
}

#[pyfunction]
fn return_enum() -> MyEnum {
    MyEnum::Variant
}

#[test]
fn test_return_enum() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let f = wrap_pyfunction!(return_enum)(py);
    let mynum = py.get_type::<MyEnum>();

    py_run!(py, f mynum, "assert f() == mynum.Variant")
}
