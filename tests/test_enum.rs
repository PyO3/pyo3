#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::py_run;

#[path = "../src/tests/common.rs"]
mod common;

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MyEnum {
    Variant,
    OtherVariant,
}

#[test]
fn test_enum_class_attr() {
    Python::with_gil(|py| {
        let my_enum = py.get_type_bound::<MyEnum>();
        let var = Py::new(py, MyEnum::Variant).unwrap();
        py_assert!(py, my_enum var, "my_enum.Variant == var");
    })
}

#[pyfunction]
fn return_enum() -> MyEnum {
    MyEnum::Variant
}

#[test]
fn test_return_enum() {
    Python::with_gil(|py| {
        let f = wrap_pyfunction_bound!(return_enum)(py).unwrap();
        let mynum = py.get_type_bound::<MyEnum>();

        py_run!(py, f mynum, "assert f() == mynum.Variant")
    });
}

#[pyfunction]
fn enum_arg(e: MyEnum) {
    assert_eq!(MyEnum::OtherVariant, e)
}

#[test]
fn test_enum_arg() {
    Python::with_gil(|py| {
        let f = wrap_pyfunction_bound!(enum_arg)(py).unwrap();
        let mynum = py.get_type_bound::<MyEnum>();

        py_run!(py, f mynum, "f(mynum.OtherVariant)")
    })
}

#[test]
fn test_enum_eq_enum() {
    Python::with_gil(|py| {
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
    Python::with_gil(|py| {
        let var1 = Py::new(py, MyEnum::Variant).unwrap();
        py_assert!(py, var1, "(var1 == 'foo') == False");
        py_assert!(py, var1, "(var1 != 'foo') == True");
    })
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Eq, Clone)]
enum CustomDiscriminant {
    One = 1,
    Two = 2,
}

#[test]
fn test_custom_discriminant() {
    Python::with_gil(|py| {
        #[allow(non_snake_case)]
        let CustomDiscriminant = py.get_type_bound::<CustomDiscriminant>();
        let one = Py::new(py, CustomDiscriminant::One).unwrap();
        let two = Py::new(py, CustomDiscriminant::Two).unwrap();
        py_run!(py, CustomDiscriminant one two, r#"
        assert CustomDiscriminant.One == one
        assert CustomDiscriminant.Two == two
        assert one != two
        "#);
    })
}

#[test]
fn test_enum_to_int() {
    Python::with_gil(|py| {
        let one = Py::new(py, CustomDiscriminant::One).unwrap();
        py_assert!(py, one, "int(one) == 1");
        let v = Py::new(py, MyEnum::Variant).unwrap();
        let v_value = MyEnum::Variant as isize;
        py_run!(py, v v_value, "int(v) == v_value");
    })
}

#[test]
fn test_enum_compare_int() {
    Python::with_gil(|py| {
        let one = Py::new(py, CustomDiscriminant::One).unwrap();
        py_run!(
            py,
            one,
            r#"
            assert one == 1
            assert 1 == one
            assert one != 2
        "#
        )
    })
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)]
enum SmallEnum {
    V = 1,
}

#[test]
fn test_enum_compare_int_no_throw_when_overflow() {
    Python::with_gil(|py| {
        let v = Py::new(py, SmallEnum::V).unwrap();
        py_assert!(py, v, "v != 1<<30")
    })
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(usize)]
#[allow(clippy::enum_clike_unportable_variant)]
enum BigEnum {
    V = usize::MAX,
}

#[test]
fn test_big_enum_no_overflow() {
    Python::with_gil(|py| {
        let usize_max = usize::MAX;
        let v = Py::new(py, BigEnum::V).unwrap();

        py_assert!(py, usize_max v, "v == usize_max");
        py_assert!(py, usize_max v, "int(v) == usize_max");
    })
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u16, align(8))]
enum TestReprParse {
    V,
}

#[test]
fn test_repr_parse() {
    assert_eq!(std::mem::align_of::<TestReprParse>(), 8);
}

#[pyclass(eq, eq_int, name = "MyEnum")]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RenameEnum {
    Variant,
}

#[test]
fn test_rename_enum_repr_correct() {
    Python::with_gil(|py| {
        let var1 = Py::new(py, RenameEnum::Variant).unwrap();
        py_assert!(py, var1, "repr(var1) == 'MyEnum.Variant'");
    })
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RenameVariantEnum {
    #[pyo3(name = "VARIANT")]
    Variant,
}

#[test]
fn test_rename_variant_repr_correct() {
    Python::with_gil(|py| {
        let var1 = Py::new(py, RenameVariantEnum::Variant).unwrap();
        py_assert!(py, var1, "repr(var1) == 'RenameVariantEnum.VARIANT'");
    })
}

#[pyclass(eq, eq_int, rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(clippy::enum_variant_names)]
enum RenameAllVariantsEnum {
    VariantOne,
    VariantTwo,
    #[pyo3(name = "VariantThree")]
    VariantFour,
}

#[test]
fn test_renaming_all_enum_variants() {
    Python::with_gil(|py| {
        let enum_obj = py.get_type_bound::<RenameAllVariantsEnum>();
        py_assert!(py, enum_obj, "enum_obj.VARIANT_ONE == enum_obj.VARIANT_ONE");
        py_assert!(py, enum_obj, "enum_obj.VARIANT_TWO == enum_obj.VARIANT_TWO");
        py_assert!(
            py,
            enum_obj,
            "enum_obj.VariantThree == enum_obj.VariantThree"
        );
    });
}
