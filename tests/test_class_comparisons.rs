#![cfg(feature = "macros")]

use pyo3::prelude::*;

mod test_utils;

#[pyclass(eq)]
#[derive(Debug, Clone, PartialEq)]
pub enum MyEnum {
    Variant,
    OtherVariant,
}

#[pyclass(eq, ord)]
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd)]
pub enum MyEnumOrd {
    Variant,
    OtherVariant,
}

#[test]
fn test_enum_eq_enum() {
    Python::attach(|py| {
        let var1 = Py::new(py, MyEnum::Variant).unwrap();
        let var2 = Py::new(py, MyEnum::Variant).unwrap();
        let other_var = Py::new(py, MyEnum::OtherVariant).unwrap();
        py_assert!(py, var1 var2, "var1 == var2");
        py_assert!(py, var1 other_var, "var1 != other_var");
        py_assert!(py, var1 var2, "(var1 != var2) == False");
    })
}

#[test]
fn test_enum_eq_incomparable() {
    Python::attach(|py| {
        let var1 = Py::new(py, MyEnum::Variant).unwrap();
        py_assert!(py, var1, "(var1 == 'foo') == False");
        py_assert!(py, var1, "(var1 != 'foo') == True");
    })
}

#[test]
fn test_enum_ord_comparable_opt_in_only() {
    Python::attach(|py| {
        let var1 = Py::new(py, MyEnum::Variant).unwrap();
        let var2 = Py::new(py, MyEnum::OtherVariant).unwrap();
        // ordering on simple enums if opt in only, thus raising an error below
        py_expect_exception!(py, var1 var2, "(var1 > var2) == False", PyTypeError);
    })
}

#[test]
fn test_simple_enum_ord_comparable() {
    Python::attach(|py| {
        let var1 = Py::new(py, MyEnumOrd::Variant).unwrap();
        let var2 = Py::new(py, MyEnumOrd::OtherVariant).unwrap();
        let var3 = Py::new(py, MyEnumOrd::OtherVariant).unwrap();
        py_assert!(py, var1 var2, "(var1 > var2) == False");
        py_assert!(py, var1 var2, "(var1 < var2) == True");
        py_assert!(py, var1 var2, "(var1 >= var2) == False");
        py_assert!(py, var2 var3, "(var3 >= var2) == True");
        py_assert!(py, var1 var2, "(var1 <= var2) == True");
    })
}

#[pyclass(eq, ord)]
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd)]
pub enum MyComplexEnumOrd {
    Variant(i32),
    OtherVariant(String),
}

#[pyclass(eq, ord)]
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd)]
pub enum MyComplexEnumOrd2 {
    Variant { msg: String, idx: u32 },
    OtherVariant { name: String, idx: u32 },
}

#[test]
fn test_complex_enum_ord_comparable() {
    Python::attach(|py| {
        let var1 = Py::new(py, MyComplexEnumOrd::Variant(-2)).unwrap();
        let var2 = Py::new(py, MyComplexEnumOrd::Variant(5)).unwrap();
        let var3 = Py::new(py, MyComplexEnumOrd::OtherVariant("a".to_string())).unwrap();
        let var4 = Py::new(py, MyComplexEnumOrd::OtherVariant("b".to_string())).unwrap();
        py_assert!(py, var1 var2, "(var1 > var2) == False");
        py_assert!(py, var1 var2, "(var1 < var2) == True");
        py_assert!(py, var1 var2, "(var1 >= var2) == False");
        py_assert!(py, var1 var2, "(var1 <= var2) == True");

        py_assert!(py, var1 var3, "(var1 >= var3) == False");
        py_assert!(py, var1 var3, "(var1 <= var3) == True");

        py_assert!(py, var3 var4, "(var3 >= var4) == False");
        py_assert!(py, var3 var4, "(var3 <= var4) == True");

        let var5 = Py::new(
            py,
            MyComplexEnumOrd2::Variant {
                msg: "hello".to_string(),
                idx: 1,
            },
        )
        .unwrap();
        let var6 = Py::new(
            py,
            MyComplexEnumOrd2::Variant {
                msg: "hello".to_string(),
                idx: 1,
            },
        )
        .unwrap();
        let var7 = Py::new(
            py,
            MyComplexEnumOrd2::Variant {
                msg: "goodbye".to_string(),
                idx: 7,
            },
        )
        .unwrap();
        let var8 = Py::new(
            py,
            MyComplexEnumOrd2::Variant {
                msg: "about".to_string(),
                idx: 0,
            },
        )
        .unwrap();
        let var9 = Py::new(
            py,
            MyComplexEnumOrd2::OtherVariant {
                name: "albert".to_string(),
                idx: 1,
            },
        )
        .unwrap();

        py_assert!(py, var5 var6, "(var5 == var6) == True");
        py_assert!(py, var5 var6, "(var5 <= var6) == True");
        py_assert!(py, var6 var7, "(var6 <= var7) == False");
        py_assert!(py, var6 var7, "(var6 >= var7) == True");
        py_assert!(py, var5 var8, "(var5 > var8) == True");
        py_assert!(py, var8 var9, "(var9 > var8) == True");
    })
}

#[pyclass(eq, ord)]
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd)]
pub struct Point {
    x: i32,
    y: i32,
    z: i32,
}

#[test]
fn test_struct_numeric_ord_comparable() {
    Python::attach(|py| {
        let var1 = Py::new(py, Point { x: 10, y: 2, z: 3 }).unwrap();
        let var2 = Py::new(py, Point { x: 2, y: 2, z: 3 }).unwrap();
        let var3 = Py::new(py, Point { x: 1, y: 22, z: 4 }).unwrap();
        let var4 = Py::new(py, Point { x: 1, y: 3, z: 4 }).unwrap();
        let var5 = Py::new(py, Point { x: 1, y: 3, z: 4 }).unwrap();
        py_assert!(py, var1 var2, "(var1 > var2) == True");
        py_assert!(py, var1 var2, "(var1 <= var2) == False");
        py_assert!(py, var2 var3, "(var3 < var2) == True");
        py_assert!(py, var3 var4, "(var3 > var4) == True");
        py_assert!(py, var4 var5, "(var4 == var5) == True");
        py_assert!(py, var3 var5, "(var3 != var5) == True");
    })
}

#[pyclass(eq, ord)]
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd)]
pub struct Person {
    surname: String,
    given_name: String,
}

#[test]
fn test_struct_string_ord_comparable() {
    Python::attach(|py| {
        let var1 = Py::new(
            py,
            Person {
                surname: "zzz".to_string(),
                given_name: "bob".to_string(),
            },
        )
        .unwrap();
        let var2 = Py::new(
            py,
            Person {
                surname: "aaa".to_string(),
                given_name: "sally".to_string(),
            },
        )
        .unwrap();
        let var3 = Py::new(
            py,
            Person {
                surname: "eee".to_string(),
                given_name: "qqq".to_string(),
            },
        )
        .unwrap();
        let var4 = Py::new(
            py,
            Person {
                surname: "ddd".to_string(),
                given_name: "aaa".to_string(),
            },
        )
        .unwrap();

        py_assert!(py, var1 var2, "(var1 > var2) == True");
        py_assert!(py, var1 var2, "(var1 <= var2) == False");
        py_assert!(py, var1 var3, "(var1 >= var3) == True");
        py_assert!(py, var2 var3, "(var2 >= var3) == False");
        py_assert!(py, var3 var4, "(var3 >= var4) == True");
        py_assert!(py, var3 var4, "(var3 != var4) == True");
    })
}

#[pyclass(eq, ord)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Record {
    name: String,
    title: String,
    idx: u32,
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.idx.partial_cmp(&other.idx).unwrap())
    }
}

#[test]
fn test_struct_custom_ord_comparable() {
    Python::attach(|py| {
        let var1 = Py::new(
            py,
            Record {
                name: "zzz".to_string(),
                title: "bbb".to_string(),
                idx: 9,
            },
        )
        .unwrap();
        let var2 = Py::new(
            py,
            Record {
                name: "ddd".to_string(),
                title: "aaa".to_string(),
                idx: 1,
            },
        )
        .unwrap();
        let var3 = Py::new(
            py,
            Record {
                name: "vvv".to_string(),
                title: "ggg".to_string(),
                idx: 19,
            },
        )
        .unwrap();
        let var4 = Py::new(
            py,
            Record {
                name: "vvv".to_string(),
                title: "ggg".to_string(),
                idx: 19,
            },
        )
        .unwrap();

        py_assert!(py, var1 var2, "(var1 > var2) == True");
        py_assert!(py, var1 var2, "(var1 <= var2) == False");
        py_assert!(py, var1 var3, "(var1 >= var3) == False");
        py_assert!(py, var2 var3, "(var2 >= var3) == False");
        py_assert!(py, var3 var4, "(var3 == var4) == True");
        py_assert!(py, var2 var4, "(var2 != var4) == True");
    })
}
