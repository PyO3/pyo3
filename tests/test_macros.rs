#![cfg(feature = "macros")]

//! Ensure that pyo3 macros can be used inside macro_rules!

use pyo3::prelude::*;

#[macro_use]
mod test_utils;

macro_rules! make_struct_using_macro {
    // Ensure that one doesn't need to fall back on the escape type: tt
    // in order to macro create pyclass.
    ($class_name:ident, $py_name:literal) => {
        #[pyclass(name=$py_name, subclass)]
        struct $class_name {}
    };
}

make_struct_using_macro!(MyBaseClass, "MyClass");

macro_rules! set_extends_via_macro {
    ($class_name:ident, $base_class:path) => {
        // Try and pass a variable into the extends parameter
        #[allow(dead_code)]
        #[pyclass(extends=$base_class)]
        struct $class_name {}
    };
}

set_extends_via_macro!(MyClass2, MyBaseClass);

//
// Check that pyfunctiona nd text_signature can be called with macro arguments.
//

macro_rules! fn_macro {
    ($sig:literal, $a_exp:expr, $b_exp:expr, $c_exp: expr) => {
        // Try and pass a variable into the signature parameter
        #[pyfunction(signature = ($a_exp, $b_exp, *, $c_exp))]
        #[pyo3(text_signature = $sig)]
        fn my_function_in_macro(a: i32, b: Option<i32>, c: i32) {
            let _ = (a, b, c);
        }
    };
}

fn_macro!("(a, b=None, *, c=42)", a, b = None, c = 42);

macro_rules! property_rename_via_macro {
    ($prop_name:ident) => {
        #[pyclass]
        struct ClassWithProperty {
            member: u64,
        }

        #[pymethods]
        impl ClassWithProperty {
            #[getter($prop_name)]
            fn get_member(&self) -> u64 {
                self.member
            }

            #[setter($prop_name)]
            fn set_member(&mut self, member: u64) {
                self.member = member;
            }
        }
    };
}

property_rename_via_macro!(my_new_property_name);

#[test]
fn test_macro_rules_interactions() {
    Python::attach(|py| {
        let my_base = py.get_type::<MyBaseClass>();
        py_assert!(py, my_base, "my_base.__name__ == 'MyClass'");

        let my_func = wrap_pyfunction!(my_function_in_macro, py).unwrap();
        py_assert!(
            py,
            my_func,
            "my_func.__text_signature__ == '(a, b=None, *, c=42)'"
        );

        let renamed_prop = py.get_type::<ClassWithProperty>();
        py_assert!(
            py,
            renamed_prop,
            "hasattr(renamed_prop, 'my_new_property_name')"
        );
    });
}
