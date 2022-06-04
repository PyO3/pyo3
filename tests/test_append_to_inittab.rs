#![cfg(all(feature = "macros", not(PyPy)))]
use pyo3::prelude::*;

#[pyfunction]
fn foo() -> usize {
    123
}

#[pymodule]
fn module_with_functions(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(foo, m)?).unwrap();
    Ok(())
}

#[cfg(not(PyPy))]
#[test]
fn test_module_append_to_inittab() {
    use pyo3::append_to_inittab;
    append_to_inittab!(module_with_functions);
    Python::with_gil(|py| {
        py.run(
            r#"
import module_with_functions
assert module_with_functions.foo() == 123
"#,
            None,
            None,
        )
        .map_err(|e| e.print(py))
        .unwrap();
    })
}
