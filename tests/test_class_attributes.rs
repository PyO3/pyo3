#![cfg(feature = "macros")]

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

    #[classattr]
    fn a_foo_with_py(py: Python<'_>) -> Py<Foo> {
        Py::new(py, Foo { x: 1 }).unwrap()
    }
}

#[test]
fn class_attributes() {
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

// Ignored because heap types are not immutable:
// https://github.com/python/cpython/blob/master/Objects/typeobject.c#L3399-L3409
#[test]
#[ignore]
fn class_attributes_are_immutable() {
    Python::with_gil(|py| {
        let foo_obj = py.get_type::<Foo>();
        py_expect_exception!(py, foo_obj, "foo_obj.a = 6", PyTypeError);
    });
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
    Python::with_gil(|py| {
        let foo_obj = py.get_type::<Foo>();
        let bar_obj = py.get_type::<Bar>();
        py_assert!(py, foo_obj, "foo_obj.a_foo.x == 1");
        py_assert!(py, foo_obj, "foo_obj.bar.x == 2");
        py_assert!(py, bar_obj, "bar_obj.a_foo.x == 3");
    });
}

#[test]
#[cfg(panic = "unwind")]
fn test_fallible_class_attribute() {
    use pyo3::{exceptions::PyValueError, types::PyString};

    struct CaptureStdErr<'py> {
        oldstderr: &'py PyAny,
        string_io: &'py PyAny,
    }

    impl<'py> CaptureStdErr<'py> {
        fn new(py: Python<'py>) -> PyResult<Self> {
            let sys = py.import("sys")?;
            let oldstderr = sys.getattr("stderr")?;
            let string_io = py.import("io")?.getattr("StringIO")?.call0()?;
            sys.setattr("stderr", string_io)?;
            Ok(Self {
                oldstderr,
                string_io,
            })
        }

        fn reset(self) -> PyResult<String> {
            let py = self.string_io.py();
            let payload = self
                .string_io
                .getattr("getvalue")?
                .call0()?
                .downcast::<PyString>()?
                .to_str()?
                .to_owned();
            let sys = py.import("sys")?;
            sys.setattr("stderr", self.oldstderr)?;
            Ok(payload)
        }
    }

    #[pyclass]
    struct BrokenClass;

    #[pymethods]
    impl BrokenClass {
        #[classattr]
        fn fails_to_init() -> PyResult<i32> {
            Err(PyValueError::new_err("failed to create class attribute"))
        }
    }

    Python::with_gil(|py| {
        let stderr = CaptureStdErr::new(py).unwrap();
        assert!(std::panic::catch_unwind(|| py.get_type::<BrokenClass>()).is_err());
        assert_eq!(
            stderr.reset().unwrap().trim(),
            "\
ValueError: failed to create class attribute

The above exception was the direct cause of the following exception:

RuntimeError: An error occurred while initializing `BrokenClass.fails_to_init`

The above exception was the direct cause of the following exception:

RuntimeError: An error occurred while initializing class BrokenClass"
        )
    });
}
