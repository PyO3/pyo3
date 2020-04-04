//! Ensure that pyo3 macros can be used inside macro_rules!

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[macro_use]
mod common;

macro_rules! make_struct_using_macro {
    // Ensure that one doesn't need to fall back on the escape type: tt
    // in order to macro create pyclass.
    ($class_name:ident, $py_name:literal) => {
        #[pyclass(name=$py_name)]
        struct $class_name {}
    };
}

make_struct_using_macro!(MyBaseClass, "MyClass");

macro_rules! set_extends_via_macro {
    ($class_name:ident, $base_class:path) => {
        // Try and pass a variable into the extends parameter
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
        // Try and pass a variable into the extends parameter
        #[pyfunction($a_exp, $b_exp, "*", $c_exp)]
        #[pyo3(text_signature = $sig)]
        fn my_function_in_macro(a: i32, b: Option<i32>, c: i32) {
            let _ = (a, b, c);
        }
    };
}

fn_macro!("(a, b=None, *, c=42)", a, b = "None", c = 42);

#[test]
fn test_macro_rules_interactions() {
    Python::with_gil(|py| {
        let my_base = py.get_type::<MyBaseClass>();
        py_assert!(py, my_base, "my_base.__name__ == 'MyClass'");

        let my_func = wrap_pyfunction!(my_function_in_macro, py).unwrap();
        py_assert!(
            py,
            my_func,
            "my_func.__text_signature__ == '(a, b=None, *, c=42)'"
        );
    });
}
