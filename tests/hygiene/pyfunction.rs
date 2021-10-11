#![no_implicit_prelude]
#![allow(unused_variables)]

#[::pyo3::pyfunction]
fn do_something(x: i32) -> ::pyo3::PyResult<i32> {
    ::std::result::Result::Ok(x)
}

#[test]
fn invoke_wrap_pyfunction() {
    ::pyo3::Python::with_gil(|py| {
        let func = ::pyo3::wrap_pyfunction!(do_something)(py).unwrap();
        ::pyo3::py_run!(py, func, r#"func(5)"#);
    });
}
