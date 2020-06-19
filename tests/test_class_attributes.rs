use pyo3::prelude::*;

mod common;

#[pyclass]
struct Foo {
    #[pyo3(get)]
    x: i32,
}

#[pyclass]
struct Bar {
    #[pyo3(get)]
    x: i32,
}

#[pymethods]
impl Foo {
    #[classattr]
    const MY_CONST: &'static str = "foobar";

    #[classattr]
    fn a() -> i32 {
        5
    }

    #[classattr]
    #[name = "B"]
    fn b() -> String {
        "bar".to_string()
    }

    #[classattr]
    fn bar() -> Bar {
        Bar { x: 2 }
    }
}

#[test]
fn class_attributes() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let foo_obj = py.get_type::<Foo>();
    py_assert!(py, foo_obj, "foo_obj.a == 5");
    py_assert!(py, foo_obj, "foo_obj.B == 'bar'");
    py_assert!(py, foo_obj, "foo_obj.MY_CONST == 'foobar'");
}

#[test]
fn class_attributes_are_immutable() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let foo_obj = py.get_type::<Foo>();
    py_expect_exception!(py, foo_obj, "foo_obj.a = 6", TypeError);
}

#[pyclass]
struct SelfClassAttribute {
    #[pyo3(get)]
    x: i32,
}

#[pymethods]
impl SelfClassAttribute {
    #[classattr]
    const SELF: SelfClassAttribute = SelfClassAttribute { x: 1 };
}

#[test]
#[should_panic(expected = "Recursive initialization of type_object for SelfClassAttribute")]
fn recursive_class_attributes() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    py.get_type::<SelfClassAttribute>();
}
