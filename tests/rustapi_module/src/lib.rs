#![feature(proc_macro, specialization)]

extern crate pyo3;

use pyo3::{py, Py, Python, PyModule, PyResult};
use pyo3::prelude::PyDate;

#[py::modinit(datetime)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "make_date")]
    fn make_date(py: Python, year: u32, month: u32, day: u32) -> PyResult<Py<PyDate>> {
        Ok(PyDate::new(py, year, month, day))
    }

    Ok(())
}
