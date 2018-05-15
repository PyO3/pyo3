#![feature(proc_macro, specialization, const_fn)]
extern crate pyo3;

use pyo3::prelude::*;
use pyo3::py::modinit as pymodinit;
mod dict_size;

#[pymodinit(_test_dict)]
fn init_mod(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<dict_size::DictSize>()?;
    Ok(())
}

