use pyo3::prelude::*;
use pyo3::types::PyString;

#[pyfunction]
fn vec_to_vec_i32(vec: Vec<i32>) -> Vec<i32> {
    vec
}

#[pyfunction]
fn array_to_array_i32(arr: [i32; 3]) -> [i32; 3] {
    arr
}

#[pyfunction]
fn vec_to_vec_pystring(vec: Vec<&PyString>) -> Vec<&PyString> {
    vec
}

#[pymodule]
pub fn sequence(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(vec_to_vec_i32, m)?)?;
    m.add_function(wrap_pyfunction!(array_to_array_i32, m)?)?;
    m.add_function(wrap_pyfunction!(vec_to_vec_pystring, m)?)?;
    Ok(())
}
