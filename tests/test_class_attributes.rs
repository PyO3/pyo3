use pyo3::prelude::*;

mod common;

#[pyclass]
struct Foo {}

#[pymethods]
impl Foo {
    #[classattr]
    fn a() -> i32 {
        5
    }

    #[classattr]
    #[name = "B"]
    fn b() -> String {
        "bar".to_string()
    }
}

#[test]
fn class_attributes() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let foo_obj = py.get_type::<Foo>();
    py_assert!(py, foo_obj, "foo_obj.a == 5");
    py_assert!(py, foo_obj, "foo_obj.B == 'bar'");
}
