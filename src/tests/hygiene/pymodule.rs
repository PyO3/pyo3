#[crate::pyfunction]
#[pyo3(crate = "crate")]
fn do_something(x: i32) -> crate::PyResult<i32> {
    ::std::result::Result::Ok(x)
}

#[crate::pymodule]
#[pyo3(crate = "crate")]
fn foo(
    _py: crate::Python<'_>,
    _m: &crate::Bound<'_, crate::types::PyModule>,
) -> crate::PyResult<()> {
    ::std::result::Result::Ok(())
}

#[crate::pymodule]
#[pyo3(crate = "crate")]
fn my_module(m: &crate::Bound<'_, crate::types::PyModule>) -> crate::PyResult<()> {
    m.add_function(crate::wrap_pyfunction!(do_something, m)?)?;
    m.add_wrapped(crate::wrap_pymodule!(foo))?;

    ::std::result::Result::Ok(())
}

#[crate::pymodule(submodule)]
#[pyo3(crate = "crate")]
mod my_module_declarative {
    #[pymodule_export]
    use super::{do_something, foo};
}
