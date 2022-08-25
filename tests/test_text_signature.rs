#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::{types::PyType, wrap_pymodule, PyCell};

mod common;

#[test]
fn class_without_docs_or_signature() {
    #[pyclass]
    struct MyClass {}

    Python::with_gil(|py| {
        let typeobj = py.get_type::<MyClass>();

        py_assert!(py, typeobj, "typeobj.__doc__ is None");
        py_assert!(py, typeobj, "typeobj.__text_signature__ is None");
    });
}

#[test]
fn class_with_docs() {
    /// docs line1
    #[pyclass]
    /// docs line2
    struct MyClass {}

    Python::with_gil(|py| {
        let typeobj = py.get_type::<MyClass>();

        py_assert!(py, typeobj, "typeobj.__doc__ == 'docs line1\\ndocs line2'");
        py_assert!(py, typeobj, "typeobj.__text_signature__ is None");
    });
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_10)), ignore)]
fn class_with_docs_and_signature() {
    /// docs line1
    #[pyclass]
    /// docs line2
    #[pyo3(text_signature = "(a, b=None, *, c=42)")]
    /// docs line3
    struct MyClass {}

    #[pymethods]
    impl MyClass {
        #[new]
        #[pyo3(signature = (a, b=None, *, c=42))]
        fn __new__(a: i32, b: Option<i32>, c: i32) -> Self {
            let _ = (a, b, c);
            Self {}
        }
    }

    Python::with_gil(|py| {
        let typeobj = py.get_type::<MyClass>();

        py_assert!(
            py,
            typeobj,
            "typeobj.__doc__ == 'docs line1\\ndocs line2\\ndocs line3'"
        );
        py_assert!(
            py,
            typeobj,
            "typeobj.__text_signature__ == '(a, b=None, *, c=42)'"
        );
    });
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_10)), ignore)]
fn class_with_signature() {
    #[pyclass]
    #[pyo3(text_signature = "(a, b=None, *, c=42)")]
    struct MyClass {}

    #[pymethods]
    impl MyClass {
        #[new]
        #[pyo3(signature = (a, b=None, *, c=42))]
        fn __new__(a: i32, b: Option<i32>, c: i32) -> Self {
            let _ = (a, b, c);
            Self {}
        }
    }

    Python::with_gil(|py| {
        let typeobj = py.get_type::<MyClass>();

        py_assert!(
            py,
            typeobj,
            "typeobj.__doc__ is None or typeobj.__doc__ == ''"
        );
        py_assert!(
            py,
            typeobj,
            "typeobj.__text_signature__ == '(a, b=None, *, c=42)'"
        );
    });
}

#[test]
fn test_function() {
    #[pyfunction(signature = (a, b=None, *, c=42))]
    #[pyo3(text_signature = "(a, b=None, *, c=42)")]
    fn my_function(a: i32, b: Option<i32>, c: i32) {
        let _ = (a, b, c);
    }

    Python::with_gil(|py| {
        let f = wrap_pyfunction!(my_function)(py).unwrap();

        py_assert!(py, f, "f.__text_signature__ == '(a, b=None, *, c=42)'");
    });
}

#[test]
fn test_pyfn() {
    #[pymodule]
    fn my_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
        #[pyfn(m, signature = (a, b=None, *, c=42))]
        #[pyo3(text_signature = "(a, b=None, *, c=42)")]
        fn my_function(a: i32, b: Option<i32>, c: i32) {
            let _ = (a, b, c);
        }
        Ok(())
    }

    Python::with_gil(|py| {
        let m = wrap_pymodule!(my_module)(py);

        py_assert!(
            py,
            m,
            "m.my_function.__text_signature__ == '(a, b=None, *, c=42)'"
        );
    });
}

#[test]
fn test_methods() {
    #[pyclass]
    struct MyClass {}

    #[pymethods]
    impl MyClass {
        #[pyo3(text_signature = "($self, a)")]
        fn method(&self, a: i32) {
            let _ = a;
        }
        #[pyo3(text_signature = "($self, b)")]
        fn pyself_method(_this: &PyCell<Self>, b: i32) {
            let _ = b;
        }
        #[classmethod]
        #[pyo3(text_signature = "($cls, c)")]
        fn class_method(_cls: &PyType, c: i32) {
            let _ = c;
        }
        #[staticmethod]
        #[pyo3(text_signature = "(d)")]
        fn static_method(d: i32) {
            let _ = d;
        }
    }

    Python::with_gil(|py| {
        let typeobj = py.get_type::<MyClass>();

        py_assert!(
            py,
            typeobj,
            "typeobj.method.__text_signature__ == '($self, a)'"
        );
        py_assert!(
            py,
            typeobj,
            "typeobj.pyself_method.__text_signature__ == '($self, b)'"
        );
        py_assert!(
            py,
            typeobj,
            "typeobj.class_method.__text_signature__ == '($cls, c)'"
        );
        py_assert!(
            py,
            typeobj,
            "typeobj.static_method.__text_signature__ == '(d)'"
        );
    });
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_10)), ignore)]
fn test_raw_identifiers() {
    #[pyclass]
    #[pyo3(text_signature = "($self)")]
    struct r#MyClass {}

    #[pymethods]
    impl MyClass {
        #[new]
        fn new() -> MyClass {
            MyClass {}
        }
        #[pyo3(text_signature = "($self)")]
        fn r#method(&self) {}
    }

    Python::with_gil(|py| {
        let typeobj = py.get_type::<MyClass>();

        py_assert!(py, typeobj, "typeobj.__text_signature__ == '($self)'");

        py_assert!(
            py,
            typeobj,
            "typeobj.method.__text_signature__ == '($self)'"
        );
    });
}
