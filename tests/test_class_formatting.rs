#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::py_run;
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
enum ComplexEnumWithStr {
    A(u32),
    B { msg: String },
}

#[test]
fn test_str_representation_complex_enum() {
    Python::with_gil(|py| {
        let var1 = Py::new(py, ComplexEnumWithStr::A(45)).unwrap();
        let var2 = Py::new(
            py,
            ComplexEnumWithStr::B {
                msg: "Hello".to_string(),
            },
        )
        .unwrap();
        py_assert!(py, var1, "str(var1) == 'A(45)'");
        py_assert!(py, var2, "str(var2) == 'B { msg: \"Hello\" }'");
    })
}

#[pyclass(str = "{0}, {1}, {2}")]
#[derive(PartialEq)]
struct Coord(u32, u32, u32);

#[pyclass(str = "{{{0}, {1}, {2}}}")]
#[derive(PartialEq)]
struct Coord2(u32, u32, u32);

#[test]
fn test_str_representation_by_position() {
    Python::with_gil(|py| {
        let var1 = Py::new(py, Coord(1, 2, 3)).unwrap();
        let var2 = Py::new(py, Coord2(1, 2, 3)).unwrap();
        py_assert!(py, var1, "str(var1) == '1, 2, 3'");
        py_assert!(py, var2, "str(var2) == '{1, 2, 3}'");
    })
}

#[pyclass(str = "name: {name}: {name}, idn: {idn:03} with message: {msg} full output: {:?}")]
#[derive(PartialEq, Debug)]
struct Point4 {
    name: String,
    msg: String,
    idn: u32,
}

#[test]
fn test_mixed_and_repeated_str_formats() {
    Python::with_gil(|py| {
        let var1 = Py::new(
            py,
            Point4 {
                name: "aaa".to_string(),
                msg: "hello".to_string(),
                idn: 1,
            },
        )
        .unwrap();
        py_run!(
            py,
            var1,
            r#"
        assert str(var1) == 'name: aaa: aaa, idn: 001 with message: hello full output: Point4 { name: "aaa", msg: "hello", idn: 1 }'
        "#
        );
    })
}
