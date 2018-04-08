#![feature(proc_macro, specialization)]

extern crate pyo3;
use pyo3::{py, Py, Python, PyModule, PyResult};
use pyo3::{ToPyObject};
use pyo3::prelude::{PyObject};
use pyo3::prelude::{PyDate, PyTime, PyDateTime, PyTzInfo};

#[py::modinit(datetime)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "make_date")]
    fn make_date(py: Python, year: u32, month: u32, day: u32) -> PyResult<Py<PyDate>> {
        PyDate::new(py, year, month, day)
    }

    #[pyfn(m, "make_time")]
    fn make_time(py: Python, hour: u32, minute: u32, second: u32,
                 microsecond: u32, tzinfo: Option<&PyTzInfo>) -> PyResult<Py<PyTime>> {
        let tzi: PyObject = match tzinfo {
            Some(t) => t.to_object(py),
            None => py.None(),
        };

        PyTime::new(py, hour, minute, second, microsecond, &tzi)
    }

    #[pyfn(m, "make_datetime")]
    fn make_datetime(py: Python, year: u32, month: u32, day: u32,
                     hour: u32, minute: u32, second: u32, microsecond: u32,
                     tzinfo: Option<&PyTzInfo>) -> PyResult<Py<PyDateTime>> {
        let tzi : PyObject = match tzinfo {
            Some(t) => t.to_object(py),
            None => py.None(),
        };
        PyDateTime::new(py, year, month, day, hour, minute, second, microsecond, &tzi)
    }

    Ok(())
}
