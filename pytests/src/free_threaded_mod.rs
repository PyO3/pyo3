use pyo3::prelude::*;

#[pyfunction]
fn add_two(x: usize) -> usize {
    x + 2
}

#[pymodule(supports_free_threaded = true)]
pub fn free_threaded_mod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add_two, m)?)?;
    Ok(())
}
