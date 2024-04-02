use pyo3::prelude::*;

#[pymodule(some_arg)]
fn module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}

fn main(){}
