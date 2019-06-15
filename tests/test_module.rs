use pyo3::prelude::*;

use pyo3::types::IntoPyDict;

mod common;

#[pyclass]
struct AnonClass {}

#[pyclass(module = "module")]
struct LocatedClass {}

fn sum_as_string(a: i64, b: i64) -> String {
    format!("{}", a + b).to_string()
}

#[pyfunction]
/// Doubles the given value
fn double(x: usize) -> usize {
    x * 2
}

/// This module is implemented in Rust.
#[pymodule]
fn module_with_functions(py: Python, m: &PyModule) -> PyResult<()> {
    use pyo3::wrap_pyfunction;

    #[pyfn(m, "sum_as_string")]
    fn sum_as_string_py(_py: Python, a: i64, b: i64) -> PyResult<String> {
        let out = sum_as_string(a, b);
        Ok(out)
    }

    #[pyfn(m, "no_parameters")]
    fn no_parameters() -> PyResult<usize> {
        Ok(42)
    }

    m.add_class::<AnonClass>().unwrap();
    m.add_class::<LocatedClass>().unwrap();

    m.add("foo", "bar").unwrap();

    m.add_wrapped(wrap_pyfunction!(double)).unwrap();
    m.add("also_double", wrap_pyfunction!(double)(py)).unwrap();

    Ok(())
}

#[test]
fn test_module_with_functions() {
    use pyo3::wrap_pymodule;

    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = [(
        "module_with_functions",
        wrap_pymodule!(module_with_functions)(py),
    )]
    .into_py_dict(py);

    let run = |code| py.run(code, None, Some(d)).unwrap();

    run("assert module_with_functions.__doc__ == 'This module is implemented in Rust.'");
    run("assert module_with_functions.sum_as_string(1, 2) == '3'");
    run("assert module_with_functions.no_parameters() == 42");
    run("assert module_with_functions.foo == 'bar'");
    run("assert module_with_functions.AnonClass != None");
    run("assert module_with_functions.LocatedClass != None");
    run("assert module_with_functions.LocatedClass.__module__ == 'module'");
    run("assert module_with_functions.double(3) == 6");
    run("assert module_with_functions.double.__doc__ == 'Doubles the given value'");
    run("assert module_with_functions.also_double(3) == 6");
    run("assert module_with_functions.also_double.__doc__ == 'Doubles the given value'");
}

#[pymodule(other_name)]
fn some_name(_: Python, _: &PyModule) -> PyResult<()> {
    Ok(())
}

#[test]
fn test_module_renaming() {
    use pyo3::wrap_pymodule;

    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = [("different_name", wrap_pymodule!(other_name)(py))].into_py_dict(py);

    py.run(
        "assert different_name.__name__ == 'other_name'",
        None,
        Some(d),
    )
    .unwrap();
}

#[test]
fn test_module_from_code() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let adder_mod = PyModule::from_code(
        py,
        "def add(a,b):\n\treturn a+b",
        "adder_mod.py",
        "adder_mod",
    )
    .expect("Module code should be loaded");

    let add_func = adder_mod
        .get("add")
        .expect("Add fucntion should be in the module")
        .to_object(py);

    let ret_value: i32 = add_func
        .call1(py, (1, 2))
        .expect("A value should be returned")
        .extract(py)
        .expect("The value should be able to be converted to an i32");

    assert_eq!(ret_value, 3);
}

#[pyfunction]
fn r#move() -> usize {
    42
}

#[pymodule]
fn raw_ident_module(_py: Python, module: &PyModule) -> PyResult<()> {
    use pyo3::wrap_pyfunction;

    module.add_wrapped(wrap_pyfunction!(r#move))
}

#[test]
fn test_raw_idents() {
    use pyo3::wrap_pymodule;

    let gil = Python::acquire_gil();
    let py = gil.python();

    let module = wrap_pymodule!(raw_ident_module)(py);

    py_assert!(py, module, "module.move() == 42");
}

#[pyfunction]
fn subfunction() -> String {
    "Subfunction".to_string()
}

#[pymodule]
fn submodule(_py: Python, module: &PyModule) -> PyResult<()> {
    use pyo3::wrap_pyfunction;

    module.add_wrapped(wrap_pyfunction!(subfunction))?;
    Ok(())
}

#[pyfunction]
fn superfunction() -> String {
    "Superfunction".to_string()
}

#[pymodule]
fn supermodule(_py: Python, module: &PyModule) -> PyResult<()> {
    use pyo3::{wrap_pyfunction, wrap_pymodule};

    module.add_wrapped(wrap_pyfunction!(superfunction))?;
    module.add_wrapped(wrap_pymodule!(submodule))?;
    Ok(())
}

#[test]
fn test_module_nesting() {
    use pyo3::wrap_pymodule;

    let gil = GILGuard::acquire();
    let py = gil.python();
    let supermodule = wrap_pymodule!(supermodule)(py);

    py_assert!(
        py,
        supermodule,
        "supermodule.superfunction() == 'Superfunction'"
    );
    py_assert!(
        py,
        supermodule,
        "supermodule.submodule.subfunction() == 'Subfunction'"
    );
}
