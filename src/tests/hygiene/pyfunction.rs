#[crate::pyfunction]
#[pyo3(crate = "crate")]
fn do_something(x: i32) -> crate::PyResult<i32> {
    ::std::result::Result::Ok(x)
}

#[crate::pyfunction]
#[pyo3(crate = "crate", name = "check5012")]
fn check_5012(x: i32) -> crate::PyResult<i32> {
    ::std::result::Result::Ok(x)
}

#[crate::pyfunction]
#[pyo3(crate = "crate")]
#[pyo3(warn(message = "This is a warning message"))]
fn function_with_warning() {}

#[crate::pyfunction(crate = "crate")]
#[pyo3(warn(message = "This is a warning message with custom category", category = crate::exceptions::PyFutureWarning))]
fn function_with_warning_and_category() {}

#[crate::pyfunction(crate = "crate")]
#[pyo3(warn(message = "This is a warning message"))]
#[pyo3(warn(message = "This is another warning message", category = crate::exceptions::PyFutureWarning))]
fn multiple_warning_function() {}

#[test]
fn invoke_wrap_pyfunction() {
    crate::Python::attach(|py| {
        let func = crate::wrap_pyfunction!(do_something, py).unwrap();
        crate::py_run!(py, func, r#"func(5)"#);
    });
}
