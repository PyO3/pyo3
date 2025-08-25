#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyo3::{types::PyType, wrap_pymodule};

mod test_utils;

#[test]
fn class_without_docs_or_signature() {
    #[pyclass]
    struct MyClass {}

    Python::attach(|py| {
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

    Python::attach(|py| {
        let typeobj = py.get_type::<MyClass>();

        py_assert!(py, typeobj, "typeobj.__doc__ == 'docs line1\\ndocs line2'");
        py_assert!(py, typeobj, "typeobj.__text_signature__ is None");
    });
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_10)), ignore)]
fn class_with_signature_no_doc() {
    #[pyclass]
    struct MyClass {}

    #[pymethods]
    impl MyClass {
        #[new]
        #[pyo3(signature = (a, b=None, *, c=42), text_signature = "(a, b=None, *, c=42)")]
        fn __new__(a: i32, b: Option<i32>, c: i32) -> Self {
            let _ = (a, b, c);
            Self {}
        }
    }

    Python::attach(|py| {
        let typeobj = py.get_type::<MyClass>();
        py_assert!(py, typeobj, "typeobj.__doc__ == ''");
        py_assert!(
            py,
            typeobj,
            "typeobj.__text_signature__ == '(a, b=None, *, c=42)'"
        );
    });
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_10)), ignore)]
fn class_with_docs_and_signature() {
    /// docs line1
    #[pyclass]
    /// docs line2
    struct MyClass {}

    #[pymethods]
    impl MyClass {
        #[new]
        #[pyo3(signature = (a, b=None, *, c=42), text_signature = "(a, b=None, *, c=42)")]
        fn __new__(a: i32, b: Option<i32>, c: i32) -> Self {
            let _ = (a, b, c);
            Self {}
        }
    }

    Python::attach(|py| {
        let typeobj = py.get_type::<MyClass>();

        py_assert!(py, typeobj, "typeobj.__doc__ == 'docs line1\\ndocs line2'");
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

    Python::attach(|py| {
        let f = wrap_pyfunction!(my_function)(py).unwrap();

        py_assert!(py, f, "f.__text_signature__ == '(a, b=None, *, c=42)'");
    });
}

#[test]
fn test_auto_test_signature_function() {
    #[pyfunction]
    fn my_function(a: i32, b: i32, c: i32) {
        let _ = (a, b, c);
    }

    #[pyfunction(pass_module)]
    fn my_function_2(module: &Bound<'_, PyModule>, a: i32, b: i32, c: i32) {
        let _ = (module, a, b, c);
    }

    #[pyfunction(signature = (a, /, b = None, *, c = 5))]
    fn my_function_3(a: i32, b: Option<i32>, c: i32) {
        let _ = (a, b, c);
    }

    #[pyfunction(signature = (a, /, b = None, *args, c, d=5, **kwargs))]
    fn my_function_4(
        a: i32,
        b: Option<i32>,
        args: &Bound<'_, PyTuple>,
        c: i32,
        d: i32,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) {
        let _ = (a, b, args, c, d, kwargs);
    }

    #[pyfunction(signature = (a = 1, /, b = None, c = 1.5, d=5, e = "pyo3", f = 'f', h = true))]
    fn my_function_5(a: i32, b: Option<i32>, c: f32, d: i32, e: &str, f: char, h: bool) {
        let _ = (a, b, c, d, e, f, h);
    }

    #[pyfunction]
    #[pyo3(signature=(a, b=None, c=None))]
    fn my_function_6(a: i32, b: Option<i32>, c: Option<i32>) {
        let _ = (a, b, c);
    }

    Python::attach(|py| {
        let f = wrap_pyfunction!(my_function)(py).unwrap();
        py_assert!(
            py,
            f,
            "f.__text_signature__ == '(a, b, c)', f.__text_signature__"
        );

        let f = wrap_pyfunction!(my_function_2)(py).unwrap();
        py_assert!(
            py,
            f,
            "f.__text_signature__ == '($module, a, b, c)', f.__text_signature__"
        );

        let f = wrap_pyfunction!(my_function_3)(py).unwrap();
        py_assert!(
            py,
            f,
            "f.__text_signature__ == '(a, /, b=None, *, c=5)', f.__text_signature__"
        );

        let f = wrap_pyfunction!(my_function_4)(py).unwrap();
        py_assert!(
            py,
            f,
            "f.__text_signature__ == '(a, /, b=None, *args, c, d=5, **kwargs)', f.__text_signature__"
        );

        let f = wrap_pyfunction!(my_function_5)(py).unwrap();
        py_assert!(
            py,
            f,
            "f.__text_signature__ == '(a=1, /, b=None, c=1.5, d=5, e=\"pyo3\", f=\\'f\\', h=True)', f.__text_signature__"
        );

        let f = wrap_pyfunction!(my_function_6)(py).unwrap();
        py_assert!(
            py,
            f,
            "f.__text_signature__ == '(a, b=None, c=None)', f.__text_signature__"
        );
    });
}

#[test]
fn test_auto_test_signature_method() {
    #[pyclass]
    struct MyClass {}

    #[pymethods]
    impl MyClass {
        #[new]
        fn new(a: i32, b: i32, c: i32) -> Self {
            let _ = (a, b, c);
            Self {}
        }

        fn method(&self, a: i32, b: i32, c: i32) {
            let _ = (a, b, c);
        }

        #[pyo3(signature = (a, /, b = None, *, c = 5))]
        fn method_2(&self, a: i32, b: Option<i32>, c: i32) {
            let _ = (a, b, c);
        }

        #[pyo3(signature = (a, /, b = None, *args, c, d=5, **kwargs))]
        fn method_3(
            &self,
            a: i32,
            b: Option<i32>,
            args: &Bound<'_, PyTuple>,
            c: i32,
            d: i32,
            kwargs: Option<&Bound<'_, PyDict>>,
        ) {
            let _ = (a, b, args, c, d, kwargs);
        }

        #[staticmethod]
        fn staticmethod(a: i32, b: i32, c: i32) {
            let _ = (a, b, c);
        }

        #[classmethod]
        fn classmethod(cls: &Bound<'_, PyType>, a: i32, b: i32, c: i32) {
            let _ = (cls, a, b, c);
        }
    }

    Python::attach(|py| {
        let cls = py.get_type::<MyClass>();
        #[cfg(any(not(Py_LIMITED_API), Py_3_10))]
        py_assert!(py, cls, "cls.__text_signature__ == '(a, b, c)'");
        py_assert!(
            py,
            cls,
            "cls.method.__text_signature__ == '($self, a, b, c)'"
        );
        py_assert!(
            py,
            cls,
            "cls.method_2.__text_signature__ == '($self, a, /, b=None, *, c=5)'"
        );
        py_assert!(
            py,
            cls,
            "cls.method_3.__text_signature__ == '($self, a, /, b=None, *args, c, d=5, **kwargs)'"
        );
        py_assert!(
            py,
            cls,
            "cls.staticmethod.__text_signature__ == '(a, b, c)'"
        );
        py_assert!(
            py,
            cls,
            "cls.classmethod.__text_signature__ == '($cls, a, b, c)'"
        );
    });
}

#[test]
fn test_auto_test_signature_opt_out() {
    #[pyfunction(text_signature = None)]
    fn my_function(a: i32, b: i32, c: i32) {
        let _ = (a, b, c);
    }

    #[pyfunction(signature = (a, /, b = None, *, c = 5), text_signature = None)]
    fn my_function_2(a: i32, b: Option<i32>, c: i32) {
        let _ = (a, b, c);
    }

    #[pyclass]
    struct MyClass {}

    #[pymethods]
    impl MyClass {
        #[new]
        #[pyo3(text_signature = None)]
        fn new(a: i32, b: i32, c: i32) -> Self {
            let _ = (a, b, c);
            Self {}
        }

        #[pyo3(text_signature = None)]
        fn method(&self, a: i32, b: i32, c: i32) {
            let _ = (a, b, c);
        }

        #[pyo3(signature = (a, /, b = None, *, c = 5), text_signature = None)]
        fn method_2(&self, a: i32, b: Option<i32>, c: i32) {
            let _ = (a, b, c);
        }

        #[staticmethod]
        #[pyo3(text_signature = None)]
        fn staticmethod(a: i32, b: i32, c: i32) {
            let _ = (a, b, c);
        }

        #[classmethod]
        #[pyo3(text_signature = None)]
        fn classmethod(cls: &Bound<'_, PyType>, a: i32, b: i32, c: i32) {
            let _ = (cls, a, b, c);
        }
    }

    Python::attach(|py| {
        let f = wrap_pyfunction!(my_function)(py).unwrap();
        py_assert!(py, f, "f.__text_signature__ == None");

        let f = wrap_pyfunction!(my_function_2)(py).unwrap();
        py_assert!(py, f, "f.__text_signature__ == None");

        let cls = py.get_type::<MyClass>();
        py_assert!(py, cls, "cls.__text_signature__ == None");
        py_assert!(py, cls, "cls.method.__text_signature__ == None");
        py_assert!(py, cls, "cls.method_2.__text_signature__ == None");
        py_assert!(py, cls, "cls.staticmethod.__text_signature__ == None");
        py_assert!(py, cls, "cls.classmethod.__text_signature__ == None");
    });
}

#[test]
fn test_pyfn() {
    #[pymodule]
    fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
        #[pyfn(m, signature = (a, b=None, *, c=42))]
        #[pyo3(text_signature = "(a, b=None, *, c=42)")]
        fn my_function(a: i32, b: Option<i32>, c: i32) {
            let _ = (a, b, c);
        }
        Ok(())
    }

    Python::attach(|py| {
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
        fn pyself_method(_this: &Bound<'_, Self>, b: i32) {
            let _ = b;
        }
        #[classmethod]
        #[pyo3(text_signature = "($cls, c)")]
        fn class_method(_cls: &Bound<'_, PyType>, c: i32) {
            let _ = c;
        }
        #[staticmethod]
        #[pyo3(text_signature = "(d)")]
        fn static_method(d: i32) {
            let _ = d;
        }
    }

    Python::attach(|py| {
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
    struct r#MyClass {}

    #[pymethods]
    impl MyClass {
        #[new]
        fn new() -> MyClass {
            MyClass {}
        }
        fn r#method(&self) {}
    }

    Python::attach(|py| {
        let typeobj = py.get_type::<MyClass>();

        py_assert!(py, typeobj, "typeobj.__text_signature__ == '()'");

        py_assert!(
            py,
            typeobj,
            "typeobj.method.__text_signature__ == '($self)'"
        );
    });
}
