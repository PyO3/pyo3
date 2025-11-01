#![cfg(all(feature = "macros", not(PyPy)))]

use pyo3::prelude::*;

#[pyfunction]
fn foo() -> usize {
    123
}

#[pymodule]
fn module_fn_with_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(foo, m)?)?;
    Ok(())
}

#[pymodule]
mod module_mod_with_functions {
    #[pymodule_export]
    use super::foo;
}

#[cfg(not(any(PyPy, GraalPy)))]
#[test]
fn test_module_append_to_inittab() {
    use pyo3::append_to_inittab;

    append_to_inittab!(module_fn_with_functions);

    append_to_inittab!(module_mod_with_functions);

    Python::attach(|py| {
        py.run(
            cr#"
import module_fn_with_functions
assert module_fn_with_functions.foo() == 123
"#,
            None,
            None,
        )
        .map_err(|e| e.display(py))
        .unwrap();
    });

    Python::attach(|py| {
        py.run(
            cr#"
import module_mod_with_functions
assert module_mod_with_functions.foo() == 123
"#,
            None,
            None,
        )
        .map_err(|e| e.display(py))
        .unwrap();
    });
}
