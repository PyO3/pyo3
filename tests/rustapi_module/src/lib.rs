#![feature(use_extern_macros, specialization)]

#[macro_use]
extern crate pyo3;

use pyo3::{Py, Python, PyResult};
use pyo3::{ObjectProtocol, ToPyObject};
use pyo3::prelude::{pyfunction, pymodinit};
use pyo3::prelude::{PyObject};
use pyo3::prelude::{PyModule};
use pyo3::prelude::{PyDate, PyTime, PyDateTime, PyDelta, PyTzInfo};
use pyo3::prelude::{PyDateComponentAccess, PyTimeComponentAccess};
use pyo3::prelude::{PyDeltaComponentAccess};
use pyo3::prelude::{PyTuple, PyDict};


macro_rules! to_pyobject {
    ($py:expr, $o:ident) => (match $o {
        Some(t) => t.to_object($py),
        None => $py.None()
    })
}


#[pyfunction]
fn make_date(py: Python, year: u32, month: u32, day: u32) -> PyResult<Py<PyDate>> {
    PyDate::new(py, year, month, day)
}

#[pyfunction]
fn get_date_tuple(py: Python, d: &PyDate) -> Py<PyTuple> {
    PyTuple::new(py, &[d.get_year(), d.get_month(), d.get_day()])
}

#[pyfunction]
fn date_from_timestamp(py: Python, ts: i64) -> PyResult<Py<PyDate>> {
    let timestamp = ts.to_object(py);
    let args = PyTuple::new(py, &[timestamp]);
    PyDate::from_timestamp(py, &args.to_object(py))
}

#[pyfunction]
fn make_time(py: Python, hour: u32, minute: u32, second: u32,
             microsecond: u32, tzinfo: Option<&PyTzInfo>) -> PyResult<Py<PyTime>> {
    let tzi: PyObject = to_pyobject!(py, tzinfo);
    PyTime::new(py, hour, minute, second, microsecond, &tzi)
}

#[pyfunction]
fn make_delta(py: Python, days: i32, seconds: i32, microseconds: i32) -> PyResult<Py<PyDelta>> {
    PyDelta::new(py, days, seconds, microseconds, true)
}

#[pyfunction]
fn get_delta_tuple(py: Python, delta: &PyDelta) -> Py<PyTuple> {
    PyTuple::new(py, &[delta.get_days(), delta.get_seconds(), delta.get_microseconds()])
}

#[pyfunction]
fn make_datetime(py: Python, year: u32, month: u32, day: u32,
                 hour: u32, minute: u32, second: u32, microsecond: u32,
                 tzinfo: Option<&PyTzInfo>) -> PyResult<Py<PyDateTime>> {
    let tzi : PyObject = match tzinfo {
        Some(t) => t.to_object(py),
        None => py.None(),
    };
    PyDateTime::new(py, year, month, day, hour, minute, second, microsecond, &tzi)
}

#[pyfunction]
fn get_datetime_tuple(py: Python, dt: &PyDateTime) -> Py<PyTuple> {
    PyTuple::new(py, &[dt.get_year(), dt.get_month(), dt.get_day(),
                       dt.get_hour(), dt.get_minute(), dt.get_second(),
                       dt.get_microsecond()])
}

#[cfg(Py_3_6)]
#[pyfunction]
fn get_datetime_tuple_fold(py: Python, dt: &PyDateTime) -> Py<PyTuple> {
    PyTuple::new(py, &[dt.get_year(), dt.get_month(), dt.get_day(),
                       dt.get_hour(), dt.get_minute(), dt.get_second(),
                       dt.get_microsecond(), dt.get_fold() as u32])
}

#[pyfunction]
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


#[cfg(Py_3_6)]
#[pyfunction]
fn time_with_fold(py: Python, hour: u32, minute: u32, second: u32,
                  microsecond: u32, tzinfo: Option<&PyTzInfo>,
                  fold: bool) -> PyResult<Py<PyTime>> {
    let tzi = to_pyobject!(py, tzinfo);
    PyTime::new_with_fold(py, hour, minute, second, microsecond, &tzi, fold)
}



#[pymodinit]
fn datetime(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_function!(make_date))?;
    m.add_function(wrap_function!(get_date_tuple))?;
    m.add_function(wrap_function!(date_from_timestamp))?;
    m.add_function(wrap_function!(make_time))?;
    m.add_function(wrap_function!(make_delta))?;
    m.add_function(wrap_function!(get_delta_tuple))?;
    m.add_function(wrap_function!(make_datetime))?;
    m.add_function(wrap_function!(get_datetime_tuple))?;
    m.add_function(wrap_function!(datetime_from_timestamp))?;

    // Python 3.6+ functions
    #[cfg(Py_3_6)]
    {
        m.add_function(wrap_function!(time_with_fold));
        m.add_function(wrap_function!(get_datetime_tuple_fold));
    }

    Ok(())
}
