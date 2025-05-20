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

#[test]
fn invoke_wrap_pyfunction() {
    crate::Python::with_gil(|py| {
        let func = crate::wrap_pyfunction!(do_something, py).unwrap();
        crate::py_run!(py, func, r#"func(5)"#);
    });
}
