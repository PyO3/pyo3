use pyo3::prelude::*;
use std::path::{Path, PathBuf};

#[pyfunction]
fn make_path() -> PathBuf {
    Path::new("/root").to_owned()
}

#[pyfunction]
fn take_pathbuf(path: PathBuf) -> PathBuf {
    path
}

#[pymodule(gil_used = false)]
pub fn path(m: &Bound<'_, PyModule>) -> PyResult<()> {
    PyModule::add_function(m, wrap_pyfunction!(make_path, m)?)?;
    PyModule::add_function(m, wrap_pyfunction!(take_pathbuf, m)?)?;

    Ok(())
}
