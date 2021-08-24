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
    #[pyo3(name = "RENAMED_CONST")]
    const MY_CONST_2: &'static str = "foobar_2";

    #[classattr]
    fn a() -> i32 {
        5
    }

    #[classattr]
    #[pyo3(name = "B")]
    fn b() -> String {
        "bar".to_string()
    }

    #[classattr]
    fn bar() -> Bar {
        Bar { x: 2 }
    }

    #[classattr]
    fn a_foo() -> Foo {
        Foo { x: 1 }
    }
}

#[test]
fn class_attributes() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let foo_obj = py.get_type::<Foo>();
    py_assert!(py, foo_obj, "foo_obj.MY_CONST == 'foobar'");
    py_assert!(py, foo_obj, "foo_obj.RENAMED_CONST == 'foobar_2'");
    py_assert!(py, foo_obj, "foo_obj.a == 5");
    py_assert!(py, foo_obj, "foo_obj.B == 'bar'");
    py_assert!(py, foo_obj, "foo_obj.a_foo.x == 1");
}

// Ignored because heap types are not immutable:
// https://github.com/python/cpython/blob/master/Objects/typeobject.c#L3399-L3409
#[test]
#[ignore]
fn class_attributes_are_immutable() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let foo_obj = py.get_type::<Foo>();
    py_expect_exception!(py, foo_obj, "foo_obj.a = 6", PyTypeError);
}

#[pymethods]
impl Bar {
    #[classattr]
    fn a_foo() -> Foo {
        Foo { x: 3 }
    }
}

#[test]
fn recursive_class_attributes() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let foo_obj = py.get_type::<Foo>();
    let bar_obj = py.get_type::<Bar>();
    py_assert!(py, foo_obj, "foo_obj.a_foo.x == 1");
    py_assert!(py, foo_obj, "foo_obj.bar.x == 2");
    py_assert!(py, bar_obj, "bar_obj.a_foo.x == 3");
}
