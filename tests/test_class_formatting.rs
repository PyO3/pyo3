#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::py_run;
use std::fmt::{Display, Formatter};

mod test_utils;

#[pyclass(eq, str)]
#[derive(Debug, PartialEq)]
pub enum MyEnum2 {
    Variant,
    OtherVariant,
}

impl Display for MyEnum2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[pyclass(eq, str)]
#[derive(Debug, PartialEq)]
pub enum MyEnum3 {
    #[pyo3(name = "AwesomeVariant")]
    Variant,
    OtherVariant,
}

impl Display for MyEnum3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let variant = match self {
            MyEnum3::Variant => "AwesomeVariant",
            MyEnum3::OtherVariant => "OtherVariant",
        };
        write!(f, "MyEnum.{variant}")
    }
}

#[test]
fn test_enum_class_fmt() {
    Python::attach(|py| {
        let var2 = Py::new(py, MyEnum2::Variant).unwrap();
        let var3 = Py::new(py, MyEnum3::Variant).unwrap();
        let var4 = Py::new(py, MyEnum3::OtherVariant).unwrap();
        py_assert!(py, var2, "str(var2) == 'Variant'");
        py_assert!(py, var3, "str(var3) == 'MyEnum.AwesomeVariant'");
        py_assert!(py, var4, "str(var4) == 'MyEnum.OtherVariant'");
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
fn test_custom_struct_custom_str() {
    Python::attach(|py| {
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
fn test_struct_str() {
    Python::attach(|py| {
        let var1 = Py::new(py, Point2 { x: 1, y: 2, z: 3 }).unwrap();
        py_assert!(py, var1, "str(var1) == '(1, 2, 3)'");
    })
}

#[pyclass(str)]
#[derive(PartialEq, Debug)]
enum ComplexEnumWithStr {
    A(u32),
    B { msg: String },
}

impl Display for ComplexEnumWithStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[test]
fn test_custom_complex_enum_str() {
    Python::attach(|py| {
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
    Python::attach(|py| {
        let var1 = Py::new(py, Coord(1, 2, 3)).unwrap();
        let var2 = Py::new(py, Coord2(1, 2, 3)).unwrap();
        py_assert!(py, var1, "str(var1) == '1, 2, 3'");
        py_assert!(py, var2, "str(var2) == '{1, 2, 3}'");
    })
}

#[pyclass(str = "name: {name}: {name}, idn: {idn:03} with message: {msg}")]
#[derive(PartialEq, Debug)]
struct Point4 {
    name: String,
    msg: String,
    idn: u32,
}

#[test]
fn test_mixed_and_repeated_str_formats() {
    Python::attach(|py| {
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
        assert str(var1) == 'name: aaa: aaa, idn: 001 with message: hello'
        "#
        );
    })
}

#[pyclass(str = "type: {r#type}")]
struct Foo {
    r#type: u32,
}

#[test]
fn test_raw_identifier_struct_custom_str() {
    Python::attach(|py| {
        let var1 = Py::new(py, Foo { r#type: 3 }).unwrap();
        py_assert!(py, var1, "str(var1) == 'type: 3'");
    })
}
