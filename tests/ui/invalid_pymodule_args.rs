use pyo3::prelude::*;

#[pymodule(some_arg)]
//~^ ERROR: expected one of: `name`, `crate`, `module`, `submodule`, `gil_used`
fn module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}

#[pyo3::pymodule(gil_used = false, gil_used = true, name = "foo", name = "bar")]
//~^ ERROR: `name` may only be specified once
//~| ERROR: `gil_used` may only be specified once
fn module_fn_multiple_errors(m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}

#[pyo3::pymodule(gil_used = false, gil_used = true, name = "foo", name = "bar")]
//~^ ERROR: `name` may only be specified once
//~| ERROR: `gil_used` may only be specified once
mod pyo3_module_multiple_errors {}

fn main() {}
