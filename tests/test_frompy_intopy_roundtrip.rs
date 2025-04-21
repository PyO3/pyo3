#![cfg(feature = "macros")]

use pyo3::types::{PyDict, PyString};
use pyo3::{prelude::*, IntoPyObject, IntoPyObjectRef};
use std::collections::HashMap;
use std::hash::Hash;

#[macro_use]
#[path = "../src/tests/common.rs"]
mod common;

#[derive(Debug, Clone, IntoPyObject, IntoPyObjectRef, FromPyObject)]
pub struct A<'py> {
    #[pyo3(item)]
    s: String,
    #[pyo3(item)]
    t: Bound<'py, PyString>,
    #[pyo3(item("foo"))]
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
        let pya = (&a).into_pyobject(py).unwrap();
        let new_a = pya.extract::<A<'_>>().unwrap();

        assert_eq!(a.s, new_a.s);
        assert_eq!(a.t.to_cow().unwrap(), new_a.t.to_cow().unwrap());
        assert_eq!(
            a.p.extract::<i32>().unwrap(),
            new_a.p.extract::<i32>().unwrap()
        );

        let pya = a.clone().into_pyobject(py).unwrap();
        let new_a = pya.extract::<A<'_>>().unwrap();

        assert_eq!(a.s, new_a.s);
        assert_eq!(a.t.to_cow().unwrap(), new_a.t.to_cow().unwrap());
        assert_eq!(
            a.p.extract::<i32>().unwrap(),
            new_a.p.extract::<i32>().unwrap()
        );
    });
}

#[derive(Debug, Clone, PartialEq, IntoPyObject, IntoPyObjectRef, FromPyObject)]
#[pyo3(transparent)]
pub struct B {
    test: String,
}

#[test]
fn test_transparent_named_field_struct() {
    Python::with_gil(|py| {
        let b = B {
            test: "test".into(),
        };
        let pyb = (&b).into_pyobject(py).unwrap();
        let new_b = pyb.extract::<B>().unwrap();
        assert_eq!(b, new_b);

        let pyb = b.clone().into_pyobject(py).unwrap();
        let new_b = pyb.extract::<B>().unwrap();
        assert_eq!(b, new_b);
    });
}

#[derive(Debug, Clone, PartialEq, IntoPyObject, IntoPyObjectRef, FromPyObject)]
#[pyo3(transparent)]
pub struct D<T> {
    test: T,
}

#[test]
fn test_generic_transparent_named_field_struct() {
    Python::with_gil(|py| {
        let d = D {
            test: String::from("test"),
        };
        let pyd = (&d).into_pyobject(py).unwrap();
        let new_d = pyd.extract::<D<String>>().unwrap();
        assert_eq!(d, new_d);

        let d = D { test: 1usize };
        let pyd = (&d).into_pyobject(py).unwrap();
        let new_d = pyd.extract::<D<usize>>().unwrap();
        assert_eq!(d, new_d);

        let d = D {
            test: String::from("test"),
        };
        let pyd = d.clone().into_pyobject(py).unwrap();
        let new_d = pyd.extract::<D<String>>().unwrap();
        assert_eq!(d, new_d);

        let d = D { test: 1usize };
        let pyd = d.clone().into_pyobject(py).unwrap();
        let new_d = pyd.extract::<D<usize>>().unwrap();
        assert_eq!(d, new_d);
    });
}

#[derive(Debug, IntoPyObject, IntoPyObjectRef, FromPyObject)]
pub struct GenericWithBound<K: Hash + Eq, V>(HashMap<K, V>);

#[test]
fn test_generic_with_bound() {
    Python::with_gil(|py| {
        let mut hash_map = HashMap::<String, i32>::new();
        hash_map.insert("1".into(), 1);
        hash_map.insert("2".into(), 2);
        let map = GenericWithBound(hash_map);
        let py_map = (&map).into_pyobject(py).unwrap();
        assert_eq!(py_map.len(), 2);
        assert_eq!(
            py_map
                .get_item("1")
                .unwrap()
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            1
        );
        assert_eq!(
            py_map
                .get_item("2")
                .unwrap()
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            2
        );
        assert!(py_map.get_item("3").unwrap().is_none());

        let py_map = map.into_pyobject(py).unwrap();
        assert_eq!(py_map.len(), 2);
        assert_eq!(
            py_map
                .get_item("1")
                .unwrap()
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            1
        );
        assert_eq!(
            py_map
                .get_item("2")
                .unwrap()
                .unwrap()
                .extract::<i32>()
                .unwrap(),
            2
        );
        assert!(py_map.get_item("3").unwrap().is_none());
    });
}

#[derive(Debug, Clone, PartialEq, IntoPyObject, IntoPyObjectRef, FromPyObject)]
pub struct Tuple(String, usize);

#[test]
fn test_tuple_struct() {
    Python::with_gil(|py| {
        let tup = Tuple(String::from("test"), 1);
        let tuple = (&tup).into_pyobject(py).unwrap();
        let new_tup = tuple.extract::<Tuple>().unwrap();
        assert_eq!(tup, new_tup);

        let tuple = tup.clone().into_pyobject(py).unwrap();
        let new_tup = tuple.extract::<Tuple>().unwrap();
        assert_eq!(tup, new_tup);
    });
}

#[derive(Debug, Clone, PartialEq, IntoPyObject, IntoPyObjectRef, FromPyObject)]
pub struct TransparentTuple(String);

#[test]
fn test_transparent_tuple_struct() {
    Python::with_gil(|py| {
        let tup = TransparentTuple(String::from("test"));
        let tuple = (&tup).into_pyobject(py).unwrap();
        let new_tup = tuple.extract::<TransparentTuple>().unwrap();
        assert_eq!(tup, new_tup);

        let tuple = tup.clone().into_pyobject(py).unwrap();
        let new_tup = tuple.extract::<TransparentTuple>().unwrap();
        assert_eq!(tup, new_tup);
    });
}

#[derive(Debug, Clone, PartialEq, IntoPyObject, IntoPyObjectRef, FromPyObject)]
pub enum Foo {
    TupleVar(usize, String),
    StructVar {
        #[pyo3(item)]
        test: char,
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
        let tuple_var = Foo::TupleVar(1, "test".into());
        let foo = (&tuple_var).into_pyobject(py).unwrap();
        assert_eq!(tuple_var, foo.extract::<Foo>().unwrap());

        let foo = tuple_var.clone().into_pyobject(py).unwrap();
        assert_eq!(tuple_var, foo.extract::<Foo>().unwrap());

        let struct_var = Foo::StructVar { test: 'b' };
        let foo = (&struct_var)
            .into_pyobject(py)
            .unwrap()
            .downcast_into::<PyDict>()
            .unwrap();
        assert_eq!(struct_var, foo.extract::<Foo>().unwrap());

        let foo = struct_var
            .clone()
            .into_pyobject(py)
            .unwrap()
            .downcast_into::<PyDict>()
            .unwrap();

        assert_eq!(struct_var, foo.extract::<Foo>().unwrap());

        let transparent_tuple = Foo::TransparentTuple(1);
        let foo = (&transparent_tuple).into_pyobject(py).unwrap();
        assert_eq!(transparent_tuple, foo.extract::<Foo>().unwrap());

        let foo = transparent_tuple.clone().into_pyobject(py).unwrap();
        assert_eq!(transparent_tuple, foo.extract::<Foo>().unwrap());

        let transparent_struct_var = Foo::TransparentStructVar { a: None };
        let foo = (&transparent_struct_var).into_pyobject(py).unwrap();
        assert_eq!(transparent_struct_var, foo.extract::<Foo>().unwrap());

        let foo = transparent_struct_var.clone().into_pyobject(py).unwrap();
        assert_eq!(transparent_struct_var, foo.extract::<Foo>().unwrap());
    });
}
