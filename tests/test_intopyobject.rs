#![cfg(feature = "macros")]

use pyo3::types::{PyDict, PyList, PyString};
use pyo3::{prelude::*, py_run, IntoPyObject, IntoPyObjectExt};
use std::collections::HashMap;
use std::hash::Hash;

#[macro_use]
#[path = "../src/tests/common.rs"]
mod common;

#[derive(Debug, IntoPyObject)]
pub struct A<'py> {
    s: String,
    t: Bound<'py, PyString>,
    p: Bound<'py, PyAny>,
}

#[test]
fn test_named_fields_struct() {
    Python::with_gil(|py| {
        let a = A {
            s: "Hello".into(),
            t: PyString::new(py, "World"),
            p: 42i32.into_pyobject(py).unwrap().into_any(),
        };
        let pya = a.into_pyobject(py).unwrap();
        assert_eq!(
            pya.get_item("s")
                .unwrap()
                .unwrap()
                .downcast::<PyString>()
                .unwrap(),
            "Hello"
        );
        assert_eq!(
            pya.get_item("t")
                .unwrap()
                .unwrap()
                .downcast::<PyString>()
                .unwrap(),
            "World"
        );
        assert_eq!(
            pya.get_item("p")
                .unwrap()
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            42
        );
    });
}

#[derive(Debug, IntoPyObject)]
#[pyo3(transparent)]
pub struct B<'a> {
    test: &'a str,
}

#[test]
fn test_transparent_named_field_struct() {
    Python::with_gil(|py| {
        let pyb = B { test: "test" }.into_pyobject(py).unwrap();
        let b = pyb.extract::<String>().unwrap();
        assert_eq!(b, "test");
    });
}

#[derive(Debug, IntoPyObject)]
#[pyo3(transparent)]
pub struct D<T> {
    test: T,
}

#[test]
fn test_generic_transparent_named_field_struct() {
    Python::with_gil(|py| {
        let pyd = D {
            test: String::from("test"),
        }
        .into_pyobject(py)
        .unwrap();
        let d = pyd.extract::<String>().unwrap();
        assert_eq!(d, "test");

        let pyd = D { test: 1usize }.into_pyobject(py).unwrap();
        let d = pyd.extract::<usize>().unwrap();
        assert_eq!(d, 1);
    });
}

#[derive(Debug, IntoPyObject)]
pub struct GenericWithBound<K: Hash + Eq, V>(HashMap<K, V>);

#[test]
fn test_generic_with_bound() {
    Python::with_gil(|py| {
        let mut hash_map = HashMap::<String, i32>::new();
        hash_map.insert("1".into(), 1);
        hash_map.insert("2".into(), 2);
        let map = GenericWithBound(hash_map).into_pyobject(py).unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get_item("1")
                .unwrap()
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            1
        );
        assert_eq!(
            map.get_item("2")
                .unwrap()
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            2
        );
        assert!(map.get_item("3").unwrap().is_none());
    });
}

#[derive(Debug, IntoPyObject)]
pub struct Tuple(String, usize);

#[test]
fn test_tuple_struct() {
    Python::with_gil(|py| {
        let tup = Tuple(String::from("test"), 1).into_pyobject(py).unwrap();
        assert!(tup.extract::<(usize, String)>().is_err());
        let tup = tup.extract::<(String, usize)>().unwrap();
        assert_eq!(tup.0, "test");
        assert_eq!(tup.1, 1);
    });
}

#[derive(Debug, IntoPyObject)]
pub struct TransparentTuple(String);

#[test]
fn test_transparent_tuple_struct() {
    Python::with_gil(|py| {
        let tup = TransparentTuple(String::from("test"))
            .into_pyobject(py)
            .unwrap();
        assert!(tup.extract::<(String,)>().is_err());
        let tup = tup.extract::<String>().unwrap();
        assert_eq!(tup, "test");
    });
}

fn phantom_into_py<'py, T>(
    _: std::borrow::Cow<'_, std::marker::PhantomData<T>>,
    py: Python<'py>,
) -> PyResult<Bound<'py, PyAny>> {
    std::any::type_name::<T>().into_bound_py_any(py)
}

#[derive(Debug, IntoPyObject, IntoPyObjectRef)]
pub enum Foo<'py> {
    TupleVar(
        usize,
        String,
        #[pyo3(into_py_with = phantom_into_py::<()>)] std::marker::PhantomData<()>,
    ),
    StructVar {
        test: Bound<'py, PyString>,
    },
    #[pyo3(transparent)]
    TransparentTuple(usize),
    #[pyo3(transparent)]
    TransparentStructVar {
        a: Option<String>,
    },
}

#[test]
fn test_enum() {
    Python::with_gil(|py| {
        let foo = Foo::TupleVar(1, "test".into(), std::marker::PhantomData)
            .into_pyobject(py)
            .unwrap();
        assert_eq!(
            foo.extract::<(usize, String, String)>().unwrap(),
            (1, String::from("test"), String::from("()"))
        );

        let foo = Foo::StructVar {
            test: PyString::new(py, "test"),
        }
        .into_pyobject(py)
        .unwrap()
        .downcast_into::<PyDict>()
        .unwrap();

        assert_eq!(
            foo.get_item("test")
                .unwrap()
                .unwrap()
                .downcast_into::<PyString>()
                .unwrap(),
            "test"
        );

        let foo = Foo::TransparentTuple(1).into_pyobject(py).unwrap();
        assert_eq!(foo.extract::<usize>().unwrap(), 1);

        let foo = Foo::TransparentStructVar { a: None }
            .into_pyobject(py)
            .unwrap();
        assert!(foo.is_none());
    });
}

#[derive(Debug, IntoPyObject, IntoPyObjectRef)]
pub struct Zap {
    #[pyo3(item)]
    name: String,

    #[pyo3(into_py_with = zap_into_py, item("my_object"))]
    some_object_length: usize,
}

fn zap_into_py<'py>(
    len: std::borrow::Cow<'_, usize>,
    py: Python<'py>,
) -> PyResult<Bound<'py, PyAny>> {
    Ok(PyList::new(py, 1..*len + 1)?.into_any())
}

#[test]
fn test_into_py_with() {
    Python::with_gil(|py| {
        let zap = Zap {
            name: "whatever".into(),
            some_object_length: 3,
        };

        let py_zap_ref = (&zap).into_pyobject(py).unwrap();
        let py_zap = zap.into_pyobject(py).unwrap();

        py_run!(
            py,
            py_zap_ref,
            "assert py_zap_ref == {'name': 'whatever', 'my_object': [1, 2, 3]},f'{py_zap_ref}'"
        );
        py_run!(
            py,
            py_zap,
            "assert py_zap == {'name': 'whatever', 'my_object': [1, 2, 3]},f'{py_zap}'"
        );
    });
}
