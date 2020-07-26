use pyo3::prelude::*;

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
