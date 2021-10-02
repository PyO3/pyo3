#![no_implicit_prelude]
#![allow(unused_variables)]

#[::pyo3::pyfunction]
fn do_something(x: i32) -> ::pyo3::PyResult<i32> {
    ::std::result::Result::Ok(x)
}

#[::pyo3::pymodule]
fn foo(_py: ::pyo3::Python, _m: &::pyo3::types::PyModule) -> ::pyo3::PyResult<()> {
    ::std::result::Result::Ok(())
}

#[::pyo3::pymodule]
fn my_module(_py: ::pyo3::Python, m: &::pyo3::types::PyModule) -> ::pyo3::PyResult<()> {
    m.add_function(::pyo3::wrap_pyfunction!(do_something, m)?)?;
    m.add_wrapped(::pyo3::wrap_pymodule!(foo))?;

    ::std::result::Result::Ok(())
}
