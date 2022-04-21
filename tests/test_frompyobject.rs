#![cfg(feature = "macros")]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyTuple};

#[macro_use]
mod common;

/// Helper function that concatenates the error message from
/// each error in the traceback into a single string that can
/// be tested.
fn extract_traceback(py: Python<'_>, mut error: PyErr) -> String {
    let mut error_msg = error.to_string();
    while let Some(cause) = error.cause(py) {
        error_msg.push_str(": ");
        error_msg.push_str(&cause.to_string());
        error = cause
    }
    error_msg
}

#[derive(Debug, FromPyObject)]
pub struct A<'a> {
    #[pyo3(attribute)]
    s: String,
    #[pyo3(item)]
    t: &'a PyString,
    #[pyo3(attribute("foo"))]
    p: &'a PyAny,
}

#[pyclass]
pub struct PyA {
    #[pyo3(get)]
    s: String,
    #[pyo3(get)]
    foo: Option<String>,
}

#[pymethods]
impl PyA {
    fn __getitem__(&self, key: String) -> pyo3::PyResult<String> {
        if key == "t" {
            Ok("bar".into())
        } else {
            Err(PyValueError::new_err("Failed"))
        }
    }
}

#[test]
fn test_named_fields_struct() {
    Python::with_gil(|py| {
        let pya = PyA {
            s: "foo".into(),
            foo: None,
        };
        let py_c = Py::new(py, pya).unwrap();
        let a: A<'_> =
            FromPyObject::extract(py_c.as_ref(py)).expect("Failed to extract A from PyA");
        assert_eq!(a.s, "foo");
        assert_eq!(a.t.to_string_lossy(), "bar");
        assert!(a.p.is_none());
    });
}

#[derive(Debug, FromPyObject)]
#[pyo3(transparent)]
pub struct B {
    test: String,
}

#[test]
fn test_transparent_named_field_struct() {
    Python::with_gil(|py| {
        let test: PyObject = "test".into_py(py);
        let b: B = FromPyObject::extract(test.as_ref(py)).expect("Failed to extract B from String");
        assert_eq!(b.test, "test");
        let test: PyObject = 1.into_py(py);
        let b = B::extract(test.as_ref(py));
        assert!(b.is_err());
    });
}

#[derive(Debug, FromPyObject)]
#[pyo3(transparent)]
pub struct D<T> {
    test: T,
}

#[test]
fn test_generic_transparent_named_field_struct() {
    Python::with_gil(|py| {
        let test: PyObject = "test".into_py(py);
        let d: D<String> =
            D::extract(test.as_ref(py)).expect("Failed to extract D<String> from String");
        assert_eq!(d.test, "test");
        let test = 1usize.into_py(py);
        let d: D<usize> =
            D::extract(test.as_ref(py)).expect("Failed to extract D<usize> from String");
        assert_eq!(d.test, 1);
    });
}

#[derive(Debug, FromPyObject)]
pub struct E<T, T2> {
    test: T,
    test2: T2,
}

#[pyclass]
#[derive(Clone)]
pub struct PyE {
    #[pyo3(get)]
    test: String,
    #[pyo3(get)]
    test2: usize,
}

#[test]
fn test_generic_named_fields_struct() {
    Python::with_gil(|py| {
        let pye = PyE {
            test: "test".into(),
            test2: 2,
        }
        .into_py(py);

        let e: E<String, usize> =
            E::extract(pye.as_ref(py)).expect("Failed to extract E<String, usize> from PyE");
        assert_eq!(e.test, "test");
        assert_eq!(e.test2, 2);
        let e = E::<usize, usize>::extract(pye.as_ref(py));
        assert!(e.is_err());
    });
}

#[derive(Debug, FromPyObject)]
pub struct C {
    #[pyo3(attribute("test"))]
    test: String,
}

#[test]
fn test_named_field_with_ext_fn() {
    Python::with_gil(|py| {
        let pyc = PyE {
            test: "foo".into(),
            test2: 0,
        }
        .into_py(py);
        let c = C::extract(pyc.as_ref(py)).expect("Failed to extract C from PyE");
        assert_eq!(c.test, "foo");
    });
}

#[derive(Debug, FromPyObject)]
pub struct Tuple(String, usize);

#[test]
fn test_tuple_struct() {
    Python::with_gil(|py| {
        let tup = PyTuple::new(py, &[1.into_py(py), "test".into_py(py)]);
        let tup = Tuple::extract(tup.as_ref());
        assert!(tup.is_err());
        let tup = PyTuple::new(py, &["test".into_py(py), 1.into_py(py)]);
        let tup = Tuple::extract(tup.as_ref()).expect("Failed to extract Tuple from PyTuple");
        assert_eq!(tup.0, "test");
        assert_eq!(tup.1, 1);
    });
}

#[derive(Debug, FromPyObject)]
pub struct TransparentTuple(String);

#[test]
fn test_transparent_tuple_struct() {
    Python::with_gil(|py| {
        let tup: PyObject = 1.into_py(py);
        let tup = TransparentTuple::extract(tup.as_ref(py));
        assert!(tup.is_err());
        let test: PyObject = "test".into_py(py);
        let tup = TransparentTuple::extract(test.as_ref(py))
            .expect("Failed to extract TransparentTuple from PyTuple");
        assert_eq!(tup.0, "test");
    });
}

#[pyclass]
struct PyBaz {
    #[pyo3(get)]
    tup: (String, String),
    #[pyo3(get)]
    e: PyE,
}

#[derive(Debug, FromPyObject)]
#[allow(dead_code)]
struct Baz<U, T> {
    e: E<U, T>,
    tup: Tuple,
}

#[test]
fn test_struct_nested_type_errors() {
    Python::with_gil(|py| {
        let pybaz = PyBaz {
            tup: ("test".into(), "test".into()),
            e: PyE {
                test: "foo".into(),
                test2: 0,
            },
        }
        .into_py(py);

        let test: PyResult<Baz<String, usize>> = FromPyObject::extract(pybaz.as_ref(py));
        assert!(test.is_err());
        assert_eq!(
            extract_traceback(py,test.unwrap_err()),
            "TypeError: failed to extract field Baz.tup: TypeError: failed to extract field Tuple.1: \
         TypeError: \'str\' object cannot be interpreted as an integer"
        );
    });
}

#[test]
fn test_struct_nested_type_errors_with_generics() {
    Python::with_gil(|py| {
        let pybaz = PyBaz {
            tup: ("test".into(), "test".into()),
            e: PyE {
                test: "foo".into(),
                test2: 0,
            },
        }
        .into_py(py);

        let test: PyResult<Baz<usize, String>> = FromPyObject::extract(pybaz.as_ref(py));
        assert!(test.is_err());
        assert_eq!(
            extract_traceback(py, test.unwrap_err()),
            "TypeError: failed to extract field Baz.e: TypeError: failed to extract field E.test: \
         TypeError: \'str\' object cannot be interpreted as an integer",
        );
    });
}

#[test]
fn test_transparent_struct_error_message() {
    Python::with_gil(|py| {
        let tup: PyObject = 1.into_py(py);
        let tup = B::extract(tup.as_ref(py));
        assert!(tup.is_err());
        assert_eq!(
            extract_traceback(py,tup.unwrap_err()),
            "TypeError: failed to extract field B.test: TypeError: \'int\' object cannot be converted \
         to \'PyString\'"
        );
    });
}

#[test]
fn test_tuple_struct_error_message() {
    Python::with_gil(|py| {
        let tup: PyObject = (1, "test").into_py(py);
        let tup = Tuple::extract(tup.as_ref(py));
        assert!(tup.is_err());
        assert_eq!(
            extract_traceback(py, tup.unwrap_err()),
            "TypeError: failed to extract field Tuple.0: TypeError: \'int\' object cannot be \
         converted to \'PyString\'"
        );
    });
}

#[test]
fn test_transparent_tuple_error_message() {
    Python::with_gil(|py| {
        let tup: PyObject = 1.into_py(py);
        let tup = TransparentTuple::extract(tup.as_ref(py));
        assert!(tup.is_err());
        assert_eq!(
            extract_traceback(py, tup.unwrap_err()),
            "TypeError: failed to extract inner field of TransparentTuple: 'int' object \
         cannot be converted to 'PyString'",
        );
    });
}

#[derive(Debug, FromPyObject)]
pub enum Foo<'a> {
    TupleVar(usize, String),
    StructVar {
        test: &'a PyString,
    },
    #[pyo3(transparent)]
    TransparentTuple(usize),
    #[pyo3(transparent)]
    TransparentStructVar {
        a: Option<String>,
    },
    StructVarGetAttrArg {
        #[pyo3(attribute("bla"))]
        a: bool,
    },
    StructWithGetItem {
        #[pyo3(item)]
        a: String,
    },
    StructWithGetItemArg {
        #[pyo3(item("foo"))]
        a: String,
    },
}

#[pyclass]
pub struct PyBool {
    #[pyo3(get)]
    bla: bool,
}

#[test]
fn test_enum() {
    Python::with_gil(|py| {
        let tup = PyTuple::new(py, &[1.into_py(py), "test".into_py(py)]);
        let f = Foo::extract(tup.as_ref()).expect("Failed to extract Foo from tuple");
        match f {
            Foo::TupleVar(test, test2) => {
                assert_eq!(test, 1);
                assert_eq!(test2, "test");
            }
            _ => panic!("Expected extracting Foo::TupleVar, got {:?}", f),
        }

        let pye = PyE {
            test: "foo".into(),
            test2: 0,
        }
        .into_py(py);
        let f = Foo::extract(pye.as_ref(py)).expect("Failed to extract Foo from PyE");
        match f {
            Foo::StructVar { test } => assert_eq!(test.to_string_lossy(), "foo"),
            _ => panic!("Expected extracting Foo::StructVar, got {:?}", f),
        }

        let int: PyObject = 1.into_py(py);
        let f = Foo::extract(int.as_ref(py)).expect("Failed to extract Foo from int");
        match f {
            Foo::TransparentTuple(test) => assert_eq!(test, 1),
            _ => panic!("Expected extracting Foo::TransparentTuple, got {:?}", f),
        }
        let none = py.None();
        let f = Foo::extract(none.as_ref(py)).expect("Failed to extract Foo from int");
        match f {
            Foo::TransparentStructVar { a } => assert!(a.is_none()),
            _ => panic!("Expected extracting Foo::TransparentStructVar, got {:?}", f),
        }

        let pybool = PyBool { bla: true }.into_py(py);
        let f = Foo::extract(pybool.as_ref(py)).expect("Failed to extract Foo from PyBool");
        match f {
            Foo::StructVarGetAttrArg { a } => assert!(a),
            _ => panic!("Expected extracting Foo::StructVarGetAttrArg, got {:?}", f),
        }

        let dict = PyDict::new(py);
        dict.set_item("a", "test").expect("Failed to set item");
        let f = Foo::extract(dict.as_ref()).expect("Failed to extract Foo from dict");
        match f {
            Foo::StructWithGetItem { a } => assert_eq!(a, "test"),
            _ => panic!("Expected extracting Foo::StructWithGetItem, got {:?}", f),
        }

        let dict = PyDict::new(py);
        dict.set_item("foo", "test").expect("Failed to set item");
        let f = Foo::extract(dict.as_ref()).expect("Failed to extract Foo from dict");
        match f {
            Foo::StructWithGetItemArg { a } => assert_eq!(a, "test"),
            _ => panic!("Expected extracting Foo::StructWithGetItemArg, got {:?}", f),
        }
    });
}

#[test]
fn test_enum_error() {
    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        let err = Foo::extract(dict.as_ref()).unwrap_err();
        assert_eq!(
            err.to_string(),
            "\
TypeError: failed to extract enum Foo ('TupleVar | StructVar | TransparentTuple | TransparentStructVar | StructVarGetAttrArg | StructWithGetItem | StructWithGetItemArg')
- variant TupleVar (TupleVar): 'dict' object cannot be converted to 'PyTuple'
- variant StructVar (StructVar): 'dict' object has no attribute 'test'
- variant TransparentTuple (TransparentTuple): 'dict' object cannot be interpreted as an integer
- variant TransparentStructVar (TransparentStructVar): failed to extract field Foo :: TransparentStructVar.a
- variant StructVarGetAttrArg (StructVarGetAttrArg): 'dict' object has no attribute 'bla'
- variant StructWithGetItem (StructWithGetItem): 'a'
- variant StructWithGetItemArg (StructWithGetItemArg): 'foo'"
        );
    });
}

#[derive(Debug, FromPyObject)]
enum EnumWithCatchAll<'a> {
    #[pyo3(transparent)]
    Foo(Foo<'a>),
    #[pyo3(transparent)]
    CatchAll(&'a PyAny),
}

#[test]
fn test_enum_catch_all() {
    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        let f = EnumWithCatchAll::extract(dict.as_ref())
            .expect("Failed to extract EnumWithCatchAll from dict");
        match f {
            EnumWithCatchAll::CatchAll(any) => {
                let d = <&PyDict>::extract(any).expect("Expected pydict");
                assert!(d.is_empty());
            }
            _ => panic!(
                "Expected extracting EnumWithCatchAll::CatchAll, got {:?}",
                f
            ),
        }
    });
}

#[derive(Debug, FromPyObject)]
pub enum Bar {
    #[pyo3(annotation = "str")]
    A(String),
    #[pyo3(annotation = "uint")]
    B(usize),
    #[pyo3(annotation = "int", transparent)]
    C(isize),
}

#[test]
fn test_err_rename() {
    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        let f = Bar::extract(dict.as_ref());
        assert!(f.is_err());
        assert_eq!(
            f.unwrap_err().to_string(),
            "\
TypeError: failed to extract enum Bar (\'str | uint | int\')
- variant A (str): \'dict\' object cannot be converted to \'PyString\'
- variant B (uint): \'dict\' object cannot be interpreted as an integer
- variant C (int): \'dict\' object cannot be interpreted as an integer"
        );
    });
}

#[derive(Debug, FromPyObject)]
pub struct Zap {
    #[pyo3(item)]
    name: String,

    #[pyo3(from_py_with = "PyAny::len", item("my_object"))]
    some_object_length: usize,
}

#[test]
fn test_from_py_with() {
    Python::with_gil(|py| {
        let py_zap = py
            .eval(
                r#"{"name": "whatever", "my_object": [1, 2, 3]}"#,
                None,
                None,
            )
            .expect("failed to create dict");

        let zap = Zap::extract(py_zap).unwrap();

        assert_eq!(zap.name, "whatever");
        assert_eq!(zap.some_object_length, 3usize);
    });
}

#[derive(Debug, FromPyObject)]
pub struct ZapTuple(String, #[pyo3(from_py_with = "PyAny::len")] usize);

#[test]
fn test_from_py_with_tuple_struct() {
    Python::with_gil(|py| {
        let py_zap = py
            .eval(r#"("whatever", [1, 2, 3])"#, None, None)
            .expect("failed to create tuple");

        let zap = ZapTuple::extract(py_zap).unwrap();

        assert_eq!(zap.0, "whatever");
        assert_eq!(zap.1, 3usize);
    });
}

#[derive(Debug, FromPyObject, PartialEq)]
pub enum ZapEnum {
    Zip(#[pyo3(from_py_with = "PyAny::len")] usize),
    Zap(String, #[pyo3(from_py_with = "PyAny::len")] usize),
}

#[test]
fn test_from_py_with_enum() {
    Python::with_gil(|py| {
        let py_zap = py
            .eval(r#"("whatever", [1, 2, 3])"#, None, None)
            .expect("failed to create tuple");

        let zap = ZapEnum::extract(py_zap).unwrap();
        let expected_zap = ZapEnum::Zap(String::from("whatever"), 3usize);

        assert_eq!(zap, expected_zap);
    });
}
