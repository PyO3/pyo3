#![cfg(feature = "macros")]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyTuple};

#[macro_use]
#[path = "../src/tests/common.rs"]
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
pub struct A<'py> {
    #[pyo3(attribute)]
    s: String,
    #[pyo3(item)]
    t: Bound<'py, PyString>,
    #[pyo3(attribute("foo"))]
    p: Bound<'py, PyAny>,
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
        let a = py_c
            .extract::<A<'_>>(py)
            .expect("Failed to extract A from PyA");
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
        let b = test
            .extract::<B>(py)
            .expect("Failed to extract B from String");
        assert_eq!(b.test, "test");
        let test: PyObject = 1.into_py(py);
        let b = test.extract::<B>(py);
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
        let d = test
            .extract::<D<String>>(py)
            .expect("Failed to extract D<String> from String");
        assert_eq!(d.test, "test");
        let test = 1usize.into_py(py);
        let d = test
            .extract::<D<usize>>(py)
            .expect("Failed to extract D<usize> from String");
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

        let e = pye
            .extract::<E<String, usize>>(py)
            .expect("Failed to extract E<String, usize> from PyE");
        assert_eq!(e.test, "test");
        assert_eq!(e.test2, 2);
        let e = pye.extract::<E<usize, usize>>(py);
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
        let c = pyc.extract::<C>(py).expect("Failed to extract C from PyE");
        assert_eq!(c.test, "foo");
    });
}

#[derive(Debug, FromPyObject)]
pub struct Tuple(String, usize);

#[test]
fn test_tuple_struct() {
    Python::with_gil(|py| {
        let tup = PyTuple::new(py, &[1.into_py(py), "test".into_py(py)]);
        let tup = tup.extract::<Tuple>();
        assert!(tup.is_err());
        let tup = PyTuple::new(py, &["test".into_py(py), 1.into_py(py)]);
        let tup = tup
            .extract::<Tuple>()
            .expect("Failed to extract Tuple from PyTuple");
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
        let tup = tup.extract::<TransparentTuple>(py);
        assert!(tup.is_err());
        let test: PyObject = "test".into_py(py);
        let tup = test
            .extract::<TransparentTuple>(py)
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

        let test = pybaz.extract::<Baz<String, usize>>(py);
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

        let test = pybaz.extract::<Baz<usize, usize>>(py);
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
        let tup = tup.extract::<B>(py);
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
        let tup = tup.extract::<Tuple>(py);
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
        let tup = tup.extract::<TransparentTuple>(py);
        assert!(tup.is_err());
        assert_eq!(
            extract_traceback(py, tup.unwrap_err()),
            "TypeError: failed to extract field TransparentTuple.0: TypeError: 'int' object \
         cannot be converted to 'PyString'",
        );
    });
}

#[derive(Debug, FromPyObject)]
pub enum Foo<'py> {
    TupleVar(usize, String),
    StructVar {
        test: Bound<'py, PyString>,
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
        let f = tup
            .extract::<Foo<'_>>()
            .expect("Failed to extract Foo from tuple");
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
        let f = pye
            .extract::<Foo<'_>>(py)
            .expect("Failed to extract Foo from PyE");
        match f {
            Foo::StructVar { test } => assert_eq!(test.to_string_lossy(), "foo"),
            _ => panic!("Expected extracting Foo::StructVar, got {:?}", f),
        }

        let int: PyObject = 1.into_py(py);
        let f = int
            .extract::<Foo<'_>>(py)
            .expect("Failed to extract Foo from int");
        match f {
            Foo::TransparentTuple(test) => assert_eq!(test, 1),
            _ => panic!("Expected extracting Foo::TransparentTuple, got {:?}", f),
        }
        let none = py.None();
        let f = none
            .extract::<Foo<'_>>(py)
            .expect("Failed to extract Foo from int");
        match f {
            Foo::TransparentStructVar { a } => assert!(a.is_none()),
            _ => panic!("Expected extracting Foo::TransparentStructVar, got {:?}", f),
        }

        let pybool = PyBool { bla: true }.into_py(py);
        let f = pybool
            .extract::<Foo<'_>>(py)
            .expect("Failed to extract Foo from PyBool");
        match f {
            Foo::StructVarGetAttrArg { a } => assert!(a),
            _ => panic!("Expected extracting Foo::StructVarGetAttrArg, got {:?}", f),
        }

        let dict = PyDict::new(py);
        dict.set_item("a", "test").expect("Failed to set item");
        let f = dict
            .extract::<Foo<'_>>()
            .expect("Failed to extract Foo from dict");
        match f {
            Foo::StructWithGetItem { a } => assert_eq!(a, "test"),
            _ => panic!("Expected extracting Foo::StructWithGetItem, got {:?}", f),
        }

        let dict = PyDict::new(py);
        dict.set_item("foo", "test").expect("Failed to set item");
        let f = dict
            .extract::<Foo<'_>>()
            .expect("Failed to extract Foo from dict");
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
        let err = dict.extract::<Foo<'_>>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "\
TypeError: failed to extract enum Foo ('TupleVar | StructVar | TransparentTuple | TransparentStructVar | StructVarGetAttrArg | StructWithGetItem | StructWithGetItemArg')
- variant TupleVar (TupleVar): TypeError: 'dict' object cannot be converted to 'PyTuple'
- variant StructVar (StructVar): AttributeError: 'dict' object has no attribute 'test'
- variant TransparentTuple (TransparentTuple): TypeError: failed to extract field Foo::TransparentTuple.0, caused by TypeError: 'dict' object cannot be interpreted as an integer
- variant TransparentStructVar (TransparentStructVar): TypeError: failed to extract field Foo::TransparentStructVar.a, caused by TypeError: 'dict' object cannot be converted to 'PyString'
- variant StructVarGetAttrArg (StructVarGetAttrArg): AttributeError: 'dict' object has no attribute 'bla'
- variant StructWithGetItem (StructWithGetItem): KeyError: 'a'
- variant StructWithGetItemArg (StructWithGetItemArg): KeyError: 'foo'"
        );

        let tup = PyTuple::empty(py);
        let err = tup.extract::<Foo<'_>>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "\
TypeError: failed to extract enum Foo ('TupleVar | StructVar | TransparentTuple | TransparentStructVar | StructVarGetAttrArg | StructWithGetItem | StructWithGetItemArg')
- variant TupleVar (TupleVar): ValueError: expected tuple of length 2, but got tuple of length 0
- variant StructVar (StructVar): AttributeError: 'tuple' object has no attribute 'test'
- variant TransparentTuple (TransparentTuple): TypeError: failed to extract field Foo::TransparentTuple.0, caused by TypeError: 'tuple' object cannot be interpreted as an integer
- variant TransparentStructVar (TransparentStructVar): TypeError: failed to extract field Foo::TransparentStructVar.a, caused by TypeError: 'tuple' object cannot be converted to 'PyString'
- variant StructVarGetAttrArg (StructVarGetAttrArg): AttributeError: 'tuple' object has no attribute 'bla'
- variant StructWithGetItem (StructWithGetItem): TypeError: tuple indices must be integers or slices, not str
- variant StructWithGetItemArg (StructWithGetItemArg): TypeError: tuple indices must be integers or slices, not str"
        );
    });
}

#[derive(Debug, FromPyObject)]
enum EnumWithCatchAll<'py> {
    #[allow(dead_code)]
    #[pyo3(transparent)]
    Foo(Foo<'py>),
    #[pyo3(transparent)]
    CatchAll(Bound<'py, PyAny>),
}

#[test]
fn test_enum_catch_all() {
    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        let f = dict
            .extract::<EnumWithCatchAll<'_>>()
            .expect("Failed to extract EnumWithCatchAll from dict");
        match f {
            EnumWithCatchAll::CatchAll(any) => {
                let d = any.extract::<Bound<'_, PyDict>>().expect("Expected pydict");
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
        let f = dict.extract::<Bar>();
        assert!(f.is_err());
        assert_eq!(
            f.unwrap_err().to_string(),
            "\
TypeError: failed to extract enum Bar ('str | uint | int')
- variant A (str): TypeError: failed to extract field Bar::A.0, caused by TypeError: 'dict' object cannot be converted to 'PyString'
- variant B (uint): TypeError: failed to extract field Bar::B.0, caused by TypeError: 'dict' object cannot be interpreted as an integer
- variant C (int): TypeError: failed to extract field Bar::C.0, caused by TypeError: 'dict' object cannot be interpreted as an integer"
        );
    });
}

#[derive(Debug, FromPyObject)]
pub struct Zap {
    #[pyo3(item)]
    name: String,

    #[pyo3(from_py_with = "Bound::<'_, PyAny>::len", item("my_object"))]
    some_object_length: usize,
}

#[test]
fn test_from_py_with() {
    Python::with_gil(|py| {
        let py_zap = py
            .eval_bound(
                r#"{"name": "whatever", "my_object": [1, 2, 3]}"#,
                None,
                None,
            )
            .expect("failed to create dict");

        let zap = py_zap.extract::<Zap>().unwrap();

        assert_eq!(zap.name, "whatever");
        assert_eq!(zap.some_object_length, 3usize);
    });
}

#[derive(Debug, FromPyObject)]
pub struct ZapTuple(
    String,
    #[pyo3(from_py_with = "Bound::<'_, PyAny>::len")] usize,
);

#[test]
fn test_from_py_with_tuple_struct() {
    Python::with_gil(|py| {
        let py_zap = py
            .eval_bound(r#"("whatever", [1, 2, 3])"#, None, None)
            .expect("failed to create tuple");

        let zap = py_zap.extract::<ZapTuple>().unwrap();

        assert_eq!(zap.0, "whatever");
        assert_eq!(zap.1, 3usize);
    });
}

#[test]
fn test_from_py_with_tuple_struct_error() {
    Python::with_gil(|py| {
        let py_zap = py
            .eval_bound(r#"("whatever", [1, 2, 3], "third")"#, None, None)
            .expect("failed to create tuple");

        let f = py_zap.extract::<ZapTuple>();

        assert!(f.is_err());
        assert_eq!(
            f.unwrap_err().to_string(),
            "ValueError: expected tuple of length 2, but got tuple of length 3"
        );
    });
}

#[derive(Debug, FromPyObject, PartialEq, Eq)]
pub enum ZapEnum {
    Zip(#[pyo3(from_py_with = "Bound::<'_, PyAny>::len")] usize),
    Zap(
        String,
        #[pyo3(from_py_with = "Bound::<'_, PyAny>::len")] usize,
    ),
}

#[test]
fn test_from_py_with_enum() {
    Python::with_gil(|py| {
        let py_zap = py
            .eval_bound(r#"("whatever", [1, 2, 3])"#, None, None)
            .expect("failed to create tuple");

        let zap = py_zap.extract::<ZapEnum>().unwrap();
        let expected_zap = ZapEnum::Zip(2);

        assert_eq!(zap, expected_zap);
    });
}

#[derive(Debug, FromPyObject, PartialEq, Eq)]
#[pyo3(transparent)]
pub struct TransparentFromPyWith {
    #[pyo3(from_py_with = "Bound::<'_, PyAny>::len")]
    len: usize,
}

#[test]
fn test_transparent_from_py_with() {
    Python::with_gil(|py| {
        let result = PyList::new(py, [1, 2, 3])
            .extract::<TransparentFromPyWith>()
            .unwrap();
        let expected = TransparentFromPyWith { len: 3 };

        assert_eq!(result, expected);
    });
}
