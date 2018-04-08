#![feature(proc_macro, specialization)]

extern crate pyo3;
use pyo3::{py, Py, Python, PyModule, PyResult};
use pyo3::{ToPyObject};
use pyo3::prelude::{PyObject};
use pyo3::prelude::{PyTuple, PyDict};
use pyo3::prelude::{PyDate, PyTime, PyDateTime, PyDelta, PyTzInfo};


#[py::modinit(datetime)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {


    #[pyfn(m, "make_date")]
    fn make_date(py: Python, year: u32, month: u32, day: u32) -> PyResult<Py<PyDate>> {
        PyDate::new(py, year, month, day)
    }

    #[pyfn(m, "date_from_timestamp")]
    fn date_from_timestamp(py: Python, ts: i64) -> PyResult<Py<PyDate>> {
        let timestamp = ts.to_object(py);
        let args = PyTuple::new(py, &[timestamp]);
        PyDate::from_timestamp(py, &args.to_object(py))
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

    #[pyfn(m, "make_delta")]
    fn make_delta(py: Python, days: i32, seconds: i32, microseconds: i32) -> PyResult<Py<PyDelta>> {
        PyDelta::new(py, days, seconds, microseconds, true)
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

    #[pyfn(m, "datetime_from_timestamp")]
    fn datetime_from_timestamp(py: Python, ts: f64, tz: Option<&PyTzInfo>) -> PyResult<Py<PyDateTime>> {
        let timestamp : PyObject = ts.to_object(py);
        let tzi : PyObject = match tz {
            Some(t) => t.to_object(py),
            None => py.None()
        };

        let args = PyTuple::new(py, &[timestamp, tzi]);
        let kwargs = PyDict::new(py);

        PyDateTime::from_timestamp(py, &args.to_object(py), &kwargs.to_object(py))
    }

    Ok(())
}
