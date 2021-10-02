#![no_implicit_prelude]
#![allow(unused_variables)]

#[::pyo3::pymodule]
fn my_module(_py: ::pyo3::Python, m: &::pyo3::types::PyModule) -> ::pyo3::PyResult<()> {
    m.add_function(::pyo3::wrap_pyfunction!(do_something, m)?)?;
    ::std::result::Result::Ok(())
}
