#![cfg(feature = "macros")]

use pyo3::native_enum::NativeEnum;
use pyo3::prelude::*;
use pyo3::py_native_enum;
use pyo3::py_run;

#[py_native_enum]
enum Color {
    Red,
    Green,
    Blue,
}

#[py_native_enum(base = "IntEnum")]
enum Status {
    Active = 1,
    Inactive = 2,
    Pending = 3,
}

#[py_native_enum(base = "Flag")]
enum Permission {
    Read = 1,
    Write = 2,
    Exec = 4,
}

#[py_native_enum(base = "IntFlag")]
enum Bits {
    A = 1,
    B = 2,
    C = 4,
}

#[py_native_enum(name = "Colour")]
enum RenamedColor {
    Red,
    Green,
}

#[py_native_enum]
enum Named {
    #[native_enum(name = "FIRST")]
    First,
    Second,
}

#[py_native_enum(module = "mymod")]
enum Modded {
    A,
    B,
}

#[pyfunction]
fn accept_color(c: Color) -> Color {
    c
}

#[test]
fn test_isinstance_enum() {
    Python::attach(|py| {
        let cls = Color::py_enum_class(py).unwrap();
        let red = Color::Red.to_py_member(py).unwrap();
        py_run!(py, cls red, r#"
            import enum
            assert isinstance(red, enum.Enum)
            assert isinstance(red, cls)
        "#);
    });
}

#[test]
fn test_enum_name_and_value() {
    Python::attach(|py| {
        let cls = Color::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            assert cls.Red.name == "Red"
            assert cls.Green.name == "Green"
            assert cls.Blue.name == "Blue"
        "#
        );
    });
}

#[test]
fn test_enum_len_iter_contains() {
    Python::attach(|py| {
        let cls = Color::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            assert len(cls) == 3
            members = list(cls)
            assert members == [cls.Red, cls.Green, cls.Blue]
            assert cls.Red in cls
            assert cls.Blue in cls
        "#
        );
    });
}

#[test]
fn test_enum_members_mapping() {
    Python::attach(|py| {
        let cls = Color::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            assert "Red" in cls._member_names_
            assert "Green" in cls._member_names_
            assert "Blue" in cls._member_names_
            assert len(cls._member_names_) == 3
        "#
        );
    });
}

#[test]
fn test_enum_lookup_by_name() {
    Python::attach(|py| {
        let cls = Color::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            assert cls["Red"] is cls.Red
            assert cls["Green"] is cls.Green
            assert cls["Blue"] is cls.Blue
        "#
        );
    });
}

#[test]
fn test_enum_lookup_by_value() {
    Python::attach(|py| {
        let cls = Color::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            assert cls(cls.Red.value) is cls.Red
            assert cls(cls.Blue.value) is cls.Blue
        "#
        );
    });
}

#[test]
fn test_class_identity() {
    Python::attach(|py| {
        let cls1 = Color::py_enum_class(py).unwrap();
        let cls2 = Color::py_enum_class(py).unwrap();
        py_run!(py, cls1 cls2, "assert cls1 is cls2");
    });
}

#[test]
fn test_member_identity() {
    Python::attach(|py| {
        let red1 = Color::Red.to_py_member(py).unwrap();
        let red2 = Color::Red.to_py_member(py).unwrap();
        py_run!(py, red1 red2, "assert red1 is red2");
    });
}

#[test]
fn test_into_pyobject() {
    Python::attach(|py| {
        let obj = Color::Green.into_pyobject(py).unwrap();
        let cls = Color::py_enum_class(py).unwrap();
        py_run!(py, obj cls, "assert obj is cls.Green");
    });
}

#[test]
fn test_from_py_object() {
    Python::attach(|py| {
        let blue = Color::Blue.to_py_member(py).unwrap();
        let extracted: Color = blue.extract().unwrap();
        assert!(matches!(extracted, Color::Blue));
    });
}

#[test]
fn test_pyfunction_roundtrip() {
    Python::attach(|py| {
        let f = wrap_pyfunction!(accept_color)(py).unwrap();
        let cls = Color::py_enum_class(py).unwrap();
        py_run!(py, f cls, r#"
            result = f(cls.Red)
            assert result is cls.Red
        "#);
    });
}

#[test]
fn test_int_enum_isinstance() {
    Python::attach(|py| {
        let cls = Status::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            import enum
            assert isinstance(cls.Active, enum.IntEnum)
            assert isinstance(cls.Active, int)
        "#
        );
    });
}

#[test]
fn test_int_enum_values() {
    Python::attach(|py| {
        let cls = Status::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            assert cls.Active == 1
            assert cls.Inactive == 2
            assert cls.Pending == 3
            assert cls(1) is cls.Active
            assert cls(2) is cls.Inactive
        "#
        );
    });
}

#[test]
fn test_flag_enum() {
    Python::attach(|py| {
        let cls = Permission::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            import enum
            assert isinstance(cls.Read, enum.Flag)
            rw = cls.Read | cls.Write
            assert cls.Read in rw
            assert cls.Write in rw
            assert cls.Exec not in rw
        "#
        );
    });
}

#[test]
fn test_rename_class() {
    Python::attach(|py| {
        let cls = RenamedColor::py_enum_class(py).unwrap();
        py_run!(py, cls, "assert cls.__name__ == 'Colour'");
    });
}

#[test]
fn test_rename_variant() {
    Python::attach(|py| {
        let cls = Named::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            assert hasattr(cls, "FIRST")
            assert cls.FIRST.name == "FIRST"
            assert cls.Second.name == "Second"
        "#
        );
    });
}

#[test]
fn test_module_attribute() {
    Python::attach(|py| {
        let cls = Modded::py_enum_class(py).unwrap();
        py_run!(py, cls, "assert cls.__module__ == 'mymod'");
    });
}

#[test]
fn test_int_flag_enum() {
    Python::attach(|py| {
        let cls = Bits::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            import enum
            assert isinstance(cls.A, enum.IntFlag)
            assert isinstance(cls.A, int)
            ab = cls.A | cls.B
            assert cls.A in ab
            assert cls.B in ab
            assert cls.C not in ab
        "#
        );
    });
}

#[test]
fn test_build_native_enum_qualname() {
    use pyo3::native_enum::{build_native_enum, NativeEnumBase, NativeEnumSpec, VariantValue};
    Python::attach(|py| {
        let spec = NativeEnumSpec {
            name: "Inner",
            base: NativeEnumBase::Enum,
            variants: &[("A", VariantValue::Auto), ("B", VariantValue::Auto)],
            module: None,
            qualname: Some("Outer.Inner"),
        };
        let cls = build_native_enum(py, &spec).unwrap();
        py_run!(py, cls, "assert cls.__qualname__ == 'Outer.Inner'");
    });
}

#[cfg(Py_3_11)]
#[test]
fn test_str_enum() {
    #[py_native_enum(base = "StrEnum")]
    enum Size {
        Small,
        Medium,
        Large,
    }

    Python::attach(|py| {
        let cls = Size::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            import enum
            assert isinstance(cls.Small, enum.StrEnum)
            assert isinstance(cls.Small, str)
            assert cls.Small == "Small"
            assert cls.Medium == "Medium"
            assert cls.Large == "Large"
        "#
        );
    });
}

#[cfg(Py_3_11)]
#[test]
fn test_str_variant_explicit_value() {
    #[py_native_enum(base = "StrEnum")]
    enum Tag {
        #[native_enum(value = "tag-alpha")]
        Alpha,
        #[native_enum(value = "tag-beta")]
        Beta,
    }

    Python::attach(|py| {
        let cls = Tag::py_enum_class(py).unwrap();
        py_run!(
            py,
            cls,
            r#"
            assert cls.Alpha.value == "tag-alpha"
            assert cls.Beta.value == "tag-beta"
            assert cls("tag-alpha") is cls.Alpha
        "#
        );
    });
}
