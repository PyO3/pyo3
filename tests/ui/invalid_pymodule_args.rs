use pyo3::prelude::*;

#[pymodule(some_arg)]
fn module(_py: Python, m: &PyModule) -> PyResult<()> {
    Ok(())
}

fn main(){}
