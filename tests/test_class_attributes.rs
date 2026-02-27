#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::py_run;

mod test_utils;

#[pyclass]
struct Foo {
    #[pyo3(get)]
    x: i32,
}

#[pyclass]
struct Bar {
    #[pyo3(get)]
    x: i32,
}

#[pymethods]
impl Foo {
    #[classattr]
    const MY_CONST: &'static str = "foobar";

    #[classattr]
    #[pyo3(name = "RENAMED_CONST")]
    const MY_CONST_2: &'static str = "foobar_2";

    #[classattr]
    fn a() -> i32 {
        5
    }

    #[classattr]
    #[pyo3(name = "B")]
    fn b() -> String {
        "bar".to_string()
    }

    #[classattr]
    fn bar() -> Bar {
        Bar { x: 2 }
    }

    #[classattr]
    fn a_foo() -> Foo {
        Foo { x: 1 }
    }

    #[classattr]
    fn a_foo_with_py(py: Python<'_>) -> Py<Foo> {
        Py::new(py, Foo { x: 1 }).unwrap()
    }
}

#[test]
fn class_attributes() {
    Python::attach(|py| {
        let foo_obj = py.get_type::<Foo>();
        py_assert!(py, foo_obj, "foo_obj.MY_CONST == 'foobar'");
        py_assert!(py, foo_obj, "foo_obj.RENAMED_CONST == 'foobar_2'");
        py_assert!(py, foo_obj, "foo_obj.a == 5");
        py_assert!(py, foo_obj, "foo_obj.B == 'bar'");
        py_assert!(py, foo_obj, "foo_obj.a_foo.x == 1");
        py_assert!(py, foo_obj, "foo_obj.a_foo_with_py.x == 1");
    });
}

#[test]
fn class_attributes_mutable() {
    #[pyclass]
    struct Foo {}

    #[pymethods]
    impl Foo {
        #[classattr]
        const MY_CONST: &'static str = "foobar";

        #[classattr]
        fn a() -> i32 {
            5
        }
    }

    Python::attach(|py| {
        let obj = py.get_type::<Foo>();
        py_run!(py, obj, "obj.MY_CONST = 'BAZ'");
        py_run!(py, obj, "obj.a = 42");
        py_assert!(py, obj, "obj.MY_CONST == 'BAZ'");
        py_assert!(py, obj, "obj.a == 42");
    });
}

#[test]
#[cfg(any(Py_3_14, all(Py_3_10, not(Py_LIMITED_API))))]
fn immutable_type_object() {
    #[pyclass(immutable_type)]
    struct ImmutableType {}

    #[pymethods]
    impl ImmutableType {
        #[classattr]
        const MY_CONST: &'static str = "foobar";

        #[classattr]
        fn a() -> i32 {
            5
        }
    }

    #[pyclass(immutable_type)]
    enum SimpleImmutable {
        Variant = 42,
    }

    #[pyclass(immutable_type)]
    enum ComplexImmutable {
        Variant(u32),
    }

    Python::attach(|py| {
        let obj = py.get_type::<ImmutableType>();
        py_expect_exception!(py, obj, "obj.MY_CONST = 'FOOBAR'", PyTypeError);
        py_expect_exception!(py, obj, "obj.a = 6", PyTypeError);

        let obj = py.get_type::<SimpleImmutable>();
        py_expect_exception!(py, obj, "obj.Variant = 0", PyTypeError);

        let obj = py.get_type::<ComplexImmutable>();
        py_expect_exception!(py, obj, "obj.Variant = 0", PyTypeError);
    });
}

#[pymethods]
impl Bar {
    #[classattr]
    fn a_foo() -> Foo {
        Foo { x: 3 }
    }
}

#[test]
fn recursive_class_attributes() {
    Python::attach(|py| {
        let foo_obj = py.get_type::<Foo>();
        let bar_obj = py.get_type::<Bar>();
        py_assert!(py, foo_obj, "foo_obj.a_foo.x == 1");
        py_assert!(py, foo_obj, "foo_obj.bar.x == 2");
        py_assert!(py, bar_obj, "bar_obj.a_foo.x == 3");
    });
}

#[test]
#[cfg(all(Py_3_8, panic = "unwind"))] // sys.unraisablehook not available until Python 3.8
fn test_fallible_class_attribute() {
    use pyo3::exceptions::PyValueError;
    use test_utils::UnraisableCapture;

    #[pyclass]
    struct BrokenClass;

    #[pymethods]
    impl BrokenClass {
        #[classattr]
        fn fails_to_init() -> PyResult<i32> {
            Err(PyValueError::new_err("failed to create class attribute"))
        }
    }

    Python::attach(|py| {
        let (err, object) = UnraisableCapture::enter(py, |capture| {
            // Accessing the type will attempt to initialize the class attributes
            assert!(std::panic::catch_unwind(|| py.get_type::<BrokenClass>()).is_err());

            capture.take_capture().unwrap()
        });

        assert!(object.is_none());
        assert_eq!(
            err.to_string(),
            "RuntimeError: An error occurred while initializing class BrokenClass"
        );

        let cause = err.cause(py).unwrap();
        assert_eq!(
            cause.to_string(),
            "RuntimeError: An error occurred while initializing `BrokenClass.fails_to_init`"
        );

        let cause = cause.cause(py).unwrap();
        assert_eq!(
            cause.to_string(),
            "ValueError: failed to create class attribute"
        );
        assert!(cause.cause(py).is_none());
    });
}

#[pyclass(get_all, set_all, rename_all = "camelCase")]
struct StructWithRenamedFields {
    first_field: bool,
    second_field: u8,
    #[pyo3(name = "third_field")]
    fourth_field: bool,
}

#[pymethods]
impl StructWithRenamedFields {
    #[new]
    fn new() -> Self {
        Self {
            first_field: true,
            second_field: 5,
            fourth_field: false,
        }
    }
}

#[test]
fn test_renaming_all_struct_fields() {
    use pyo3::types::PyBool;

    Python::attach(|py| {
        let struct_class = py.get_type::<StructWithRenamedFields>();
        let struct_obj = struct_class.call0().unwrap();
        assert!(struct_obj
            .setattr("firstField", PyBool::new(py, false))
            .is_ok());
        py_assert!(py, struct_obj, "struct_obj.firstField == False");
        py_assert!(py, struct_obj, "struct_obj.secondField == 5");
        assert!(struct_obj
            .setattr("third_field", PyBool::new(py, true))
            .is_ok());
        py_assert!(py, struct_obj, "struct_obj.third_field == True");
    });
}

#[pyclass(get_all, set_all, new = "from_fields")]
struct AutoNewCls {
    a: i32,
    b: String,
    c: Option<f64>,
}

#[test]
fn new_impl() {
    Python::attach(|py| {
        // python should be able to do AutoNewCls(1, "two", 3.0)
        let cls = py.get_type::<AutoNewCls>();
        pyo3::py_run!(
            py,
            cls,
            "inst = cls(1, 'two', 3.0); assert inst.a == 1; assert inst.b == 'two'; assert inst.c == 3.0"
        );
    });
}

#[pyclass(new = "from_fields", get_all)]
struct Point2d(#[pyo3(name = "first")] f64, #[pyo3(name = "second")] f64);

#[test]
fn new_impl_tuple_struct() {
    Python::attach(|py| {
        // python should be able to do AutoNewCls(1, "two", 3.0)
        let cls = py.get_type::<Point2d>();
        pyo3::py_run!(
            py,
            cls,
            "inst = cls(0.2, 0.3); assert inst.first == 0.2; assert inst.second == 0.3"
        );
    });
}

macro_rules! test_case {
    ($struct_name: ident, $rule: literal, $field_name: ident, $renamed_field_name: literal, $test_name: ident) => {
        #[pyclass(get_all, set_all, rename_all = $rule)]
        #[allow(non_snake_case)]
        struct $struct_name {
            $field_name: u8,
        }
        #[pymethods]
        impl $struct_name {
            #[new]
            fn new() -> Self {
                Self { $field_name: 0 }
            }
        }
        #[test]
        fn $test_name() {
            //use pyo3::types::PyInt;

            Python::attach(|py| {
                let struct_class = py.get_type::<$struct_name>();
                let struct_obj = struct_class.call0().unwrap();
                assert!(struct_obj.setattr($renamed_field_name, 2).is_ok());
                let attr = struct_obj.getattr($renamed_field_name).unwrap();
                assert_eq!(2, attr.extract::<u8>().unwrap());
            });
        }
    };
}

test_case!(
    LowercaseTest,
    "lowercase",
    fieldOne,
    "fieldone",
    test_rename_all_lowercase
);
test_case!(
    CamelCaseTest,
    "camelCase",
    field_one,
    "fieldOne",
    test_rename_all_camel_case
);
test_case!(
    KebabCaseTest,
    "kebab-case",
    field_one,
    "field-one",
    test_rename_all_kebab_case
);
test_case!(
    PascalCaseTest,
    "PascalCase",
    field_one,
    "FieldOne",
    test_rename_all_pascal_case
);
test_case!(
    ScreamingSnakeCaseTest,
    "SCREAMING_SNAKE_CASE",
    field_one,
    "FIELD_ONE",
    test_rename_all_screaming_snake_case
);
test_case!(
    ScreamingKebabCaseTest,
    "SCREAMING-KEBAB-CASE",
    field_one,
    "FIELD-ONE",
    test_rename_all_screaming_kebab_case
);
test_case!(
    SnakeCaseTest,
    "snake_case",
    fieldOne,
    "field_one",
    test_rename_all_snake_case
);
test_case!(
    UppercaseTest,
    "UPPERCASE",
    fieldOne,
    "FIELDONE",
    test_rename_all_uppercase
);
