#![no_implicit_prelude]
#![allow(unused_variables, clippy::unnecessary_wraps)]

#[crate::pyfunction]
#[pyo3(crate = "crate")]
fn do_something(x: i32) -> crate::PyResult<i32> {
    ::std::result::Result::Ok(x)
}

#[cfg(feature = "gil-refs")]
#[allow(deprecated)]
#[crate::pymodule]
#[pyo3(crate = "crate")]
fn foo(_py: crate::Python<'_>, _m: &crate::types::PyModule) -> crate::PyResult<()> {
    ::std::result::Result::Ok(())
}

#[crate::pymodule]
#[pyo3(crate = "crate")]
fn foo_bound(
    _py: crate::Python<'_>,
    _m: &crate::Bound<'_, crate::types::PyModule>,
) -> crate::PyResult<()> {
    ::std::result::Result::Ok(())
}

#[cfg(feature = "gil-refs")]
#[allow(deprecated)]
#[crate::pymodule]
#[pyo3(crate = "crate")]
fn my_module(_py: crate::Python<'_>, m: &crate::types::PyModule) -> crate::PyResult<()> {
    m.add_function(crate::wrap_pyfunction!(do_something, m)?)?;
    m.add_wrapped(crate::wrap_pymodule!(foo))?;

    ::std::result::Result::Ok(())
}

#[crate::pymodule]
#[pyo3(crate = "crate")]
fn my_module_bound(m: &crate::Bound<'_, crate::types::PyModule>) -> crate::PyResult<()> {
    <crate::Bound<'_, crate::types::PyModule> as crate::types::PyModuleMethods>::add_function(
        m,
        crate::wrap_pyfunction_bound!(do_something, m)?,
    )?;
    <crate::Bound<'_, crate::types::PyModule> as crate::types::PyModuleMethods>::add_wrapped(
        m,
        crate::wrap_pymodule!(foo_bound),
    )?;

    ::std::result::Result::Ok(())
}
