#![cfg(feature = "macros")]

use pyo3::prelude::*;
use std::fmt::{Display, Formatter};

#[path = "../src/tests/common.rs"]
mod common;

#[pyclass(eq, str = "MyEnum.{:?}")]
#[derive(Debug, PartialEq)]
pub enum MyEnum {
    Variant,
    OtherVariant,
}

#[pyclass(eq, str)]
#[derive(Debug, PartialEq)]
pub enum MyEnum2 {
    Variant,
    OtherVariant,
}

impl Display for MyEnum2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[test]
fn test_enum_class_attr() {
    Python::with_gil(|py| {
        let var1 = Py::new(py, MyEnum::Variant).unwrap();
        let var2 = Py::new(py, MyEnum2::Variant).unwrap();
        py_assert!(py, var1, "str(var1) == 'MyEnum.Variant'");
        py_assert!(py, var2, "str(var2) == 'Variant'");
    })
}

#[pyclass(str = "X: {x}, Y: {y}, Z: {z}")]
#[derive(PartialEq, Eq, Clone, PartialOrd)]
pub struct Point {
    x: i32,
    y: i32,
    z: i32,
}

#[test]
fn test_custom_str_representation() {
    Python::with_gil(|py| {
        let var1 = Py::new(py, Point { x: 1, y: 2, z: 3 }).unwrap();
        py_assert!(py, var1, "str(var1) == 'X: 1, Y: 2, Z: 3'");
    })
}

#[pyclass(str)]
#[derive(PartialEq, Eq, Clone, PartialOrd)]
pub struct Point2 {
    x: i32,
    y: i32,
    z: i32,
}

impl Display for Point2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[test]
fn test_display_trait_implementation() {
    Python::with_gil(|py| {
        let var1 = Py::new(py, Point2 { x: 1, y: 2, z: 3 }).unwrap();
        py_assert!(py, var1, "str(var1) == '(1, 2, 3)'");
    })
}

#[pyclass(str = "{:?}")]
#[derive(PartialEq, Debug)]
enum ComplexEnumWithHash {
    A(u32),
    B { msg: String },
}

#[test]
fn test_str_representation_complex_enum() {
    Python::with_gil(|py| {
        let var1 = Py::new(py, ComplexEnumWithHash::A(45)).unwrap();
        let var2 = Py::new(
            py,
            ComplexEnumWithHash::B {
                msg: "Hello".to_string(),
            },
        )
            .unwrap();
        py_assert!(py, var1, "str(var1) == 'A(45)'");
        py_assert!(py, var2, "str(var2) == 'B { msg: \"Hello\" }'");
    })
}

// #[pyclass(str)]
// struct StrOptAndManualStr {}
//
// impl Display for StrOptAndManualStr {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self)
//     }
// }
//
// #[pymethods]
// impl StrOptAndManualStr {
//     fn __repr__(
//         &self,
//     ) -> String {
//         todo!()
//     }
// }