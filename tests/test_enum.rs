use pyo3::prelude::*;
use pyo3::{py_run, wrap_pyfunction};

mod common;

#[pyclass]
#[derive(Debug, PartialEq, Clone)]
pub enum MyEnum {
    Variant,
    OtherVariant,
}

#[test]
fn test_enum_class_attr() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let my_enum = py.get_type::<MyEnum>();
    py_assert!(py, my_enum, "getattr(my_enum, 'Variant', None) is not None");
    py_assert!(py, my_enum, "getattr(my_enum, 'foobar', None) is None");
    py_run!(py, my_enum, "my_enum.Variant = None");
}

#[pyfunction]
fn return_enum() -> MyEnum {
    MyEnum::Variant
}

#[test]
#[ignore] // need to implement __eq__
fn test_return_enum() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let f = wrap_pyfunction!(return_enum)(py).unwrap();
    let mynum = py.get_type::<MyEnum>();

    py_run!(py, f mynum, "assert f() == mynum.Variant")
}

#[pyfunction]
fn enum_arg(e: MyEnum) {
    assert_eq!(MyEnum::OtherVariant, e)
}

#[test]
#[ignore] // need to implement __eq__
fn test_enum_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let f = wrap_pyfunction!(enum_arg)(py).unwrap();
    let mynum = py.get_type::<MyEnum>();

    py_run!(py, f mynum, "f(mynum.Variant)")
}

#[test]
fn test_default_repr_correct() {
    Python::with_gil(|py| {
        let var1 = Py::new(py, MyEnum::Variant).unwrap();
        let var2 = Py::new(py, MyEnum::OtherVariant).unwrap();
        py_assert!(py, var1, "repr(var1) == 'MyEnum.Variant'");
        py_assert!(py, var2, "repr(var2) == 'MyEnum.OtherVariant'");
    })
}
