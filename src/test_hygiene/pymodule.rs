#![no_implicit_prelude]
#![allow(unused_variables)]

#[crate::pyfunction]
#[pyo3(crate = "crate")]
fn do_something(x: i32) -> crate::PyResult<i32> {
    ::std::result::Result::Ok(x)
}

#[crate::pymodule]
#[pyo3(crate = "crate")]
fn foo(_py: crate::Python, _m: &crate::types::PyModule) -> crate::PyResult<()> {
    ::std::result::Result::Ok(())
}

#[crate::pymodule]
#[pyo3(crate = "crate")]
fn my_module(_py: crate::Python, m: &crate::types::PyModule) -> crate::PyResult<()> {
    m.add_function(crate::wrap_pyfunction!(do_something, m)?)?;
    m.add_wrapped(crate::wrap_pymodule!(foo))?;

    ::std::result::Result::Ok(())
}
