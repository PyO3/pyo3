#![cfg(all(feature = "macros", not(PyPy)))]
use pyo3::prelude::*;

#[pyfunction]
fn foo() -> usize {
    123
}

#[pymodule]
fn module_fn_with_functions(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(foo, m)?).unwrap();
    Ok(())
}

#[pymodule]
mod module_mod_with_functions {
    #[pyo3]
    use super::foo;
}

#[cfg(not(PyPy))]
#[test]
fn test_module_append_to_inittab() {
    use pyo3::append_to_inittab;
    append_to_inittab!(module_fn_with_functions);
    append_to_inittab!(module_mod_with_functions);
    Python::with_gil(|py| {
        py.run_bound(
            r#"
import module_fn_with_functions
assert module_fn_with_functions.foo() == 123
"#,
            None,
            None,
        )
        .map_err(|e| e.display(py))
        .unwrap();
    });
    Python::with_gil(|py| {
        py.run(
            r#"
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
