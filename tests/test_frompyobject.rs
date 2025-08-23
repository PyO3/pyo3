#![cfg(feature = "macros")]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict, PyList, PyString, PyTuple};

#[macro_use]
mod test_utils;

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
    Python::attach(|py| {
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
    Python::attach(|py| {
        let test = "test".into_pyobject(py).unwrap();
        let b = test
            .extract::<B>()
            .expect("Failed to extract B from String");
        assert_eq!(b.test, "test");
        let test = 1i32.into_pyobject(py).unwrap();
        let b = test.extract::<B>();
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
    Python::attach(|py| {
        let test = "test".into_pyobject(py).unwrap();
        let d = test
            .extract::<D<String>>()
            .expect("Failed to extract D<String> from String");
        assert_eq!(d.test, "test");
        let test = 1usize.into_pyobject(py).unwrap();
        let d = test
            .extract::<D<usize>>()
            .expect("Failed to extract D<usize> from String");
        assert_eq!(d.test, 1);
    });
}

#[derive(Debug, FromPyObject)]
pub struct GenericWithBound<K: std::hash::Hash + Eq, V>(std::collections::HashMap<K, V>);

#[test]
fn test_generic_with_bound() {
    Python::attach(|py| {
        let dict = [("1", 1), ("2", 2)].into_py_dict(py).unwrap();
        let map = dict.extract::<GenericWithBound<String, i32>>().unwrap().0;
        assert_eq!(map.len(), 2);
        assert_eq!(map["1"], 1);
        assert_eq!(map["2"], 2);
        assert!(!map.contains_key("3"));
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
    Python::attach(|py| {
        let pye = PyE {
            test: "test".into(),
            test2: 2,
        }
        .into_pyobject(py)
        .unwrap();

        let e = pye
            .extract::<E<String, usize>>()
            .expect("Failed to extract E<String, usize> from PyE");
        assert_eq!(e.test, "test");
        assert_eq!(e.test2, 2);
        let e = pye.extract::<E<usize, usize>>();
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
    Python::attach(|py| {
        let pyc = PyE {
            test: "foo".into(),
            test2: 0,
        }
        .into_pyobject(py)
        .unwrap();
        let c = pyc.extract::<C>().expect("Failed to extract C from PyE");
        assert_eq!(c.test, "foo");
    });
}

#[derive(Debug, FromPyObject)]
pub struct Tuple(String, usize);

#[test]
fn test_tuple_struct() {
    Python::attach(|py| {
        let tup = PyTuple::new(
            py,
            &[
                1i32.into_pyobject(py).unwrap().into_any(),
                "test".into_pyobject(py).unwrap().into_any(),
            ],
        )
        .unwrap();
        let tup = tup.extract::<Tuple>();
        assert!(tup.is_err());
        let tup = PyTuple::new(
            py,
            &[
                "test".into_pyobject(py).unwrap().into_any(),
                1i32.into_pyobject(py).unwrap().into_any(),
            ],
        )
        .unwrap();
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
    Python::attach(|py| {
        let tup = 1i32.into_pyobject(py).unwrap();
        let tup = tup.extract::<TransparentTuple>();
        assert!(tup.is_err());
        let test = "test".into_pyobject(py).unwrap();
        let tup = test
            .extract::<TransparentTuple>()
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
    Python::attach(|py| {
        let pybaz = PyBaz {
            tup: ("test".into(), "test".into()),
            e: PyE {
                test: "foo".into(),
                test2: 0,
            },
        }
        .into_pyobject(py)
        .unwrap();

        let test = pybaz.extract::<Baz<String, usize>>();
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
    Python::attach(|py| {
        let pybaz = PyBaz {
            tup: ("test".into(), "test".into()),
            e: PyE {
                test: "foo".into(),
                test2: 0,
            },
        }
        .into_pyobject(py)
        .unwrap();

        let test = pybaz.extract::<Baz<usize, usize>>();
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
    Python::attach(|py| {
        let tup = 1i32.into_pyobject(py).unwrap();
        let tup = tup.extract::<B>();
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
    Python::attach(|py| {
        let tup = (1, "test").into_pyobject(py).unwrap();
        let tup = tup.extract::<Tuple>();
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
    Python::attach(|py| {
        let tup = 1i32.into_pyobject(py).unwrap();
        let tup = tup.extract::<TransparentTuple>();
        assert!(tup.is_err());
        assert_eq!(
            extract_traceback(py, tup.unwrap_err()),
            "TypeError: failed to extract field TransparentTuple.0: TypeError: 'int' object \
         cannot be converted to 'PyString'",
        );
    });
}

#[pyclass]
struct RenameAllCls {}

#[pymethods]
impl RenameAllCls {
    #[getter]
    #[pyo3(name = "someField")]
    fn some_field(&self) -> &'static str {
        "Foo"
    }

    #[getter]
    #[pyo3(name = "customNumber")]
    fn custom_number(&self) -> i32 {
        42
    }

    fn __getitem__(&self, key: &str) -> PyResult<f32> {
        match key {
            "otherField" => Ok(42.0),
            _ => Err(pyo3::exceptions::PyKeyError::new_err("foo")),
        }
    }
}

#[test]
fn test_struct_rename_all() {
    #[derive(FromPyObject)]
    #[pyo3(rename_all = "camelCase")]
    struct RenameAll {
        some_field: String,
        #[pyo3(item)]
        other_field: f32,
        #[pyo3(attribute("customNumber"))]
        custom_name: i32,
    }

    Python::attach(|py| {
        let RenameAll {
            some_field,
            other_field,
            custom_name,
        } = RenameAllCls {}
            .into_pyobject(py)
            .unwrap()
            .extract()
            .unwrap();

        assert_eq!(some_field, "Foo");
        assert_eq!(other_field, 42.0);
        assert_eq!(custom_name, 42);
    });
}

#[test]
fn test_enum_rename_all() {
    #[derive(FromPyObject)]
    #[pyo3(rename_all = "camelCase")]
    enum RenameAll {
        Foo {
            some_field: String,
            #[pyo3(item)]
            other_field: f32,
            #[pyo3(attribute("customNumber"))]
            custom_name: i32,
        },
    }

    Python::attach(|py| {
        let RenameAll::Foo {
            some_field,
            other_field,
            custom_name,
        } = RenameAllCls {}
            .into_pyobject(py)
            .unwrap()
            .extract()
            .unwrap();

        assert_eq!(some_field, "Foo");
        assert_eq!(other_field, 42.0);
        assert_eq!(custom_name, 42);
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
    Python::attach(|py| {
        let tup = PyTuple::new(
            py,
            &[
                1i32.into_pyobject(py).unwrap().into_any(),
                "test".into_pyobject(py).unwrap().into_any(),
            ],
        )
        .unwrap();
        let f = tup
            .extract::<Foo<'_>>()
            .expect("Failed to extract Foo from tuple");
        match f {
            Foo::TupleVar(test, test2) => {
                assert_eq!(test, 1);
                assert_eq!(test2, "test");
            }
            _ => panic!("Expected extracting Foo::TupleVar, got {f:?}"),
        }

        let pye = PyE {
            test: "foo".into(),
            test2: 0,
        }
        .into_pyobject(py)
        .unwrap();
        let f = pye
            .extract::<Foo<'_>>()
            .expect("Failed to extract Foo from PyE");
        match f {
            Foo::StructVar { test } => assert_eq!(test.to_string_lossy(), "foo"),
            _ => panic!("Expected extracting Foo::StructVar, got {f:?}"),
        }

        let int = 1i32.into_pyobject(py).unwrap();
        let f = int
            .extract::<Foo<'_>>()
            .expect("Failed to extract Foo from int");
        match f {
            Foo::TransparentTuple(test) => assert_eq!(test, 1),
            _ => panic!("Expected extracting Foo::TransparentTuple, got {f:?}"),
        }
        let none = py.None();
        let f = none
            .extract::<Foo<'_>>(py)
            .expect("Failed to extract Foo from int");
        match f {
            Foo::TransparentStructVar { a } => assert!(a.is_none()),
            _ => panic!("Expected extracting Foo::TransparentStructVar, got {f:?}"),
        }

        let pybool = PyBool { bla: true }.into_pyobject(py).unwrap();
        let f = pybool
            .extract::<Foo<'_>>()
            .expect("Failed to extract Foo from PyBool");
        match f {
            Foo::StructVarGetAttrArg { a } => assert!(a),
            _ => panic!("Expected extracting Foo::StructVarGetAttrArg, got {f:?}"),
        }

        let dict = PyDict::new(py);
        dict.set_item("a", "test").expect("Failed to set item");
        let f = dict
            .extract::<Foo<'_>>()
            .expect("Failed to extract Foo from dict");
        match f {
            Foo::StructWithGetItem { a } => assert_eq!(a, "test"),
            _ => panic!("Expected extracting Foo::StructWithGetItem, got {f:?}"),
        }

        let dict = PyDict::new(py);
        dict.set_item("foo", "test").expect("Failed to set item");
        let f = dict
            .extract::<Foo<'_>>()
            .expect("Failed to extract Foo from dict");
        match f {
            Foo::StructWithGetItemArg { a } => assert_eq!(a, "test"),
            _ => panic!("Expected extracting Foo::StructWithGetItemArg, got {f:?}"),
        }
    });
}

#[test]
fn test_enum_error() {
    Python::attach(|py| {
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
    Python::attach(|py| {
        let dict = PyDict::new(py);
        let f = dict
            .extract::<EnumWithCatchAll<'_>>()
            .expect("Failed to extract EnumWithCatchAll from dict");
        match f {
            EnumWithCatchAll::CatchAll(any) => {
                let d = any.extract::<Bound<'_, PyDict>>().expect("Expected pydict");
                assert!(d.is_empty());
            }
            _ => panic!("Expected extracting EnumWithCatchAll::CatchAll, got {f:?}"),
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
    Python::attach(|py| {
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

    #[pyo3(from_py_with = Bound::<'_, PyAny>::len, item("my_object"))]
    some_object_length: usize,
}

#[test]
fn test_from_py_with() {
    Python::attach(|py| {
        let py_zap = py
            .eval(
                pyo3_ffi::c_str!(r#"{"name": "whatever", "my_object": [1, 2, 3]}"#),
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
    #[pyo3(from_py_with = Bound::<'_, PyAny>::len)] usize,
);

#[test]
fn test_from_py_with_tuple_struct() {
    Python::attach(|py| {
        let py_zap = py
            .eval(pyo3_ffi::c_str!(r#"("whatever", [1, 2, 3])"#), None, None)
            .expect("failed to create tuple");

        let zap = py_zap.extract::<ZapTuple>().unwrap();

        assert_eq!(zap.0, "whatever");
        assert_eq!(zap.1, 3usize);
    });
}

#[test]
fn test_from_py_with_tuple_struct_error() {
    Python::attach(|py| {
        let py_zap = py
            .eval(
                pyo3_ffi::c_str!(r#"("whatever", [1, 2, 3], "third")"#),
                None,
                None,
            )
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
    Zip(#[pyo3(from_py_with = Bound::<'_, PyAny>::len)] usize),
    Zap(
        String,
        #[pyo3(from_py_with = Bound::<'_, PyAny>::len)] usize,
    ),
}

#[test]
fn test_from_py_with_enum() {
    Python::attach(|py| {
        let py_zap = py
            .eval(pyo3_ffi::c_str!(r#"("whatever", [1, 2, 3])"#), None, None)
            .expect("failed to create tuple");

        let zap = py_zap.extract::<ZapEnum>().unwrap();
        let expected_zap = ZapEnum::Zip(2);

        assert_eq!(zap, expected_zap);
    });
}

#[derive(Debug, FromPyObject, PartialEq, Eq)]
#[pyo3(transparent)]
pub struct TransparentFromPyWith {
    #[pyo3(from_py_with = Bound::<'_, PyAny>::len)]
    len: usize,
}

#[test]
fn test_transparent_from_py_with() {
    Python::attach(|py| {
        let result = PyList::new(py, [1, 2, 3])
            .unwrap()
            .extract::<TransparentFromPyWith>()
            .unwrap();
        let expected = TransparentFromPyWith { len: 3 };

        assert_eq!(result, expected);
    });
}

#[derive(Debug, FromPyObject, PartialEq, Eq)]
pub struct WithKeywordAttr {
    r#box: usize,
}

#[pyclass]
pub struct WithKeywordAttrC {
    #[pyo3(get)]
    r#box: usize,
}

#[test]
fn test_with_keyword_attr() {
    Python::attach(|py| {
        let cls = WithKeywordAttrC { r#box: 3 }.into_pyobject(py).unwrap();
        let result = cls.extract::<WithKeywordAttr>().unwrap();
        let expected = WithKeywordAttr { r#box: 3 };
        assert_eq!(result, expected);
    });
}

#[derive(Debug, FromPyObject, PartialEq, Eq)]
pub struct WithKeywordItem {
    #[pyo3(item)]
    r#box: usize,
}

#[test]
fn test_with_keyword_item() {
    Python::attach(|py| {
        let dict = PyDict::new(py);
        dict.set_item("box", 3).unwrap();
        let result = dict.extract::<WithKeywordItem>().unwrap();
        let expected = WithKeywordItem { r#box: 3 };
        assert_eq!(result, expected);
    });
}

#[derive(Debug, FromPyObject, PartialEq, Eq)]
pub struct WithDefaultItem {
    #[pyo3(item, default)]
    opt: Option<usize>,
    #[pyo3(item)]
    value: usize,
}

#[test]
fn test_with_default_item() {
    Python::attach(|py| {
        let dict = PyDict::new(py);
        dict.set_item("value", 3).unwrap();
        let result = dict.extract::<WithDefaultItem>().unwrap();
        let expected = WithDefaultItem {
            value: 3,
            opt: None,
        };
        assert_eq!(result, expected);
    });
}

#[derive(Debug, FromPyObject, PartialEq, Eq)]
pub struct WithExplicitDefaultItem {
    #[pyo3(item, default = 1)]
    opt: usize,
    #[pyo3(item)]
    value: usize,
}

#[test]
fn test_with_explicit_default_item() {
    Python::attach(|py| {
        let dict = PyDict::new(py);
        dict.set_item("value", 3).unwrap();
        let result = dict.extract::<WithExplicitDefaultItem>().unwrap();
        let expected = WithExplicitDefaultItem { value: 3, opt: 1 };
        assert_eq!(result, expected);
    });
}

#[derive(Debug, FromPyObject, PartialEq, Eq)]
pub struct WithDefaultItemAndConversionFunction {
    #[pyo3(item, default, from_py_with = Bound::<'_, PyAny>::len)]
    opt: usize,
    #[pyo3(item)]
    value: usize,
}

#[test]
fn test_with_default_item_and_conversion_function() {
    Python::attach(|py| {
        // Filled case
        let dict = PyDict::new(py);
        dict.set_item("opt", (1,)).unwrap();
        dict.set_item("value", 3).unwrap();
        let result = dict
            .extract::<WithDefaultItemAndConversionFunction>()
            .unwrap();
        let expected = WithDefaultItemAndConversionFunction { opt: 1, value: 3 };
        assert_eq!(result, expected);

        // Empty case
        let dict = PyDict::new(py);
        dict.set_item("value", 3).unwrap();
        let result = dict
            .extract::<WithDefaultItemAndConversionFunction>()
            .unwrap();
        let expected = WithDefaultItemAndConversionFunction { opt: 0, value: 3 };
        assert_eq!(result, expected);

        // Error case
        let dict = PyDict::new(py);
        dict.set_item("value", 3).unwrap();
        dict.set_item("opt", 1).unwrap();
        assert!(dict
            .extract::<WithDefaultItemAndConversionFunction>()
            .is_err());
    });
}

#[derive(Debug, FromPyObject, PartialEq, Eq)]
pub enum WithDefaultItemEnum {
    #[pyo3(from_item_all)]
    Foo {
        a: usize,
        #[pyo3(default)]
        b: usize,
    },
    NeverUsedA {
        a: usize,
    },
}

#[test]
fn test_with_default_item_enum() {
    Python::attach(|py| {
        // A and B filled
        let dict = PyDict::new(py);
        dict.set_item("a", 1).unwrap();
        dict.set_item("b", 2).unwrap();
        let result = dict.extract::<WithDefaultItemEnum>().unwrap();
        let expected = WithDefaultItemEnum::Foo { a: 1, b: 2 };
        assert_eq!(result, expected);

        // A filled
        let dict = PyDict::new(py);
        dict.set_item("a", 1).unwrap();
        let result = dict.extract::<WithDefaultItemEnum>().unwrap();
        let expected = WithDefaultItemEnum::Foo { a: 1, b: 0 };
        assert_eq!(result, expected);
    });
}
