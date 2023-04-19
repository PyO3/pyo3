//! This is test_class_attributes.rs wrapped in cfg_attr
#![cfg(all(feature = "macros", feature = "pyo3"))]

use pyo3::prelude::*;

mod common;

#[cfg_attr(feature = "pyo3", pyfunction)]
fn double(x: usize) -> usize {
    x * 2
}

#[cfg_attr(feature = "pyo3", pyclass)]
struct Foo {
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    x: i32,
}

#[cfg_attr(feature = "pyo3", pyclass)]
struct Bar {
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    x: i32,
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl Foo {
    #[cfg_attr(feature = "pyo3", classattr)]
    const MY_CONST: &'static str = "foobar";

    // Here we can combine the two lines into one due to cfg_attr
    #[cfg_attr(feature = "pyo3", classattr, pyo3(name = "RENAMED_CONST"))]
    const MY_CONST_2: &'static str = "foobar_2";

    #[cfg_attr(feature = "pyo3", classattr)]
    fn a() -> i32 {
        5
    }

    // Here we don't merge them
    #[cfg_attr(feature = "pyo3", classattr)]
    #[cfg_attr(feature = "pyo3", pyo3(name = "B"))]
    fn b() -> String {
        "bar".to_string()
    }

    #[cfg_attr(feature = "pyo3", classattr)]
    fn bar() -> Bar {
        Bar { x: 2 }
    }

    #[cfg_attr(feature = "pyo3", classattr)]
    fn a_foo() -> Foo {
        Foo { x: 1 }
    }

    #[cfg_attr(feature = "pyo3", classattr)]
    fn a_foo_with_py(py: Python<'_>) -> Py<Foo> {
        Py::new(py, Foo { x: 1 }).unwrap()
    }
}

#[cfg_attr(feature = "pyo3", pymodule)]
fn optional_bindings_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Foo>()?;
    m.add_class::<Bar>()?;
    m.add_function(wrap_pyfunction!(double, m)?)?;
    Ok(())
}

#[test]
fn optional_bindings() {
    Python::with_gil(|py| {
        let foo_obj = py.get_type::<Foo>();
        py_assert!(py, foo_obj, "foo_obj.MY_CONST == 'foobar'");
        py_assert!(py, foo_obj, "foo_obj.RENAMED_CONST == 'foobar_2'");
        py_assert!(py, foo_obj, "foo_obj.a == 5");
        py_assert!(py, foo_obj, "foo_obj.B == 'bar'");
        py_assert!(py, foo_obj, "foo_obj.a_foo.x == 1");
        py_assert!(py, foo_obj, "foo_obj.a_foo_with_py.x == 1");
    });
}
