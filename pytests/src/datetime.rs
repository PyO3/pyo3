#![cfg(not(Py_LIMITED_API))]

use pyo3::prelude::*;
use pyo3::types::{
    PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess, PyTuple,
    PyTzInfo, PyTzInfoAccess,
};

#[pyfunction]
fn make_date(py: Python<'_>, year: i32, month: u8, day: u8) -> PyResult<Bound<'_, PyDate>> {
    PyDate::new(py, year, month, day)
}

#[pyfunction]
fn get_date_tuple<'py>(d: &Bound<'py, PyDate>) -> Bound<'py, PyTuple> {
    PyTuple::new(
        d.py(),
        [d.get_year(), d.get_month() as i32, d.get_day() as i32],
    )
}

#[pyfunction]
fn date_from_timestamp(py: Python<'_>, timestamp: i64) -> PyResult<Bound<'_, PyDate>> {
    PyDate::from_timestamp(py, timestamp)
}

#[pyfunction]
#[pyo3(signature=(hour, minute, second, microsecond, tzinfo=None))]
fn make_time<'py>(
    py: Python<'py>,
    hour: u8,
    minute: u8,
    second: u8,
    microsecond: u32,
    tzinfo: Option<&Bound<'py, PyTzInfo>>,
) -> PyResult<Bound<'py, PyTime>> {
    PyTime::new(py, hour, minute, second, microsecond, tzinfo)
}

#[pyfunction]
#[pyo3(signature = (hour, minute, second, microsecond, tzinfo, fold))]
fn time_with_fold<'py>(
    py: Python<'py>,
    hour: u8,
    minute: u8,
    second: u8,
    microsecond: u32,
    tzinfo: Option<&Bound<'py, PyTzInfo>>,
    fold: bool,
) -> PyResult<Bound<'py, PyTime>> {
    PyTime::new_with_fold(py, hour, minute, second, microsecond, tzinfo, fold)
}

#[pyfunction]
fn get_time_tuple<'py>(dt: &Bound<'py, PyTime>) -> Bound<'py, PyTuple> {
    PyTuple::new(
        dt.py(),
        [
            dt.get_hour() as u32,
            dt.get_minute() as u32,
            dt.get_second() as u32,
            dt.get_microsecond(),
        ],
    )
}

#[pyfunction]
fn get_time_tuple_fold<'py>(dt: &Bound<'py, PyTime>) -> Bound<'py, PyTuple> {
    PyTuple::new(
        dt.py(),
        [
            dt.get_hour() as u32,
            dt.get_minute() as u32,
            dt.get_second() as u32,
            dt.get_microsecond(),
            dt.get_fold() as u32,
        ],
    )
}

#[pyfunction]
fn make_delta(
    py: Python<'_>,
    days: i32,
    seconds: i32,
    microseconds: i32,
) -> PyResult<Bound<'_, PyDelta>> {
    PyDelta::new(py, days, seconds, microseconds, true)
}

#[pyfunction]
fn get_delta_tuple<'py>(delta: &Bound<'py, PyDelta>) -> Bound<'py, PyTuple> {
    PyTuple::new(
        delta.py(),
        [
            delta.get_days(),
            delta.get_seconds(),
            delta.get_microseconds(),
        ],
    )
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature=(year, month, day, hour, minute, second, microsecond, tzinfo=None))]
fn make_datetime<'py>(
    py: Python<'py>,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    microsecond: u32,
    tzinfo: Option<&Bound<'py, PyTzInfo>>,
) -> PyResult<Bound<'py, PyDateTime>> {
    PyDateTime::new(
        py,
        year,
        month,
        day,
        hour,
        minute,
        second,
        microsecond,
        tzinfo,
    )
}

#[pyfunction]
fn get_datetime_tuple<'py>(dt: &Bound<'py, PyDateTime>) -> Bound<'py, PyTuple> {
    PyTuple::new(
        dt.py(),
        [
            dt.get_year(),
            dt.get_month() as i32,
            dt.get_day() as i32,
            dt.get_hour() as i32,
            dt.get_minute() as i32,
            dt.get_second() as i32,
            dt.get_microsecond() as i32,
        ],
    )
}

#[pyfunction]
fn get_datetime_tuple_fold<'py>(dt: &Bound<'py, PyDateTime>) -> Bound<'py, PyTuple> {
    PyTuple::new(
        dt.py(),
        [
            dt.get_year(),
            dt.get_month() as i32,
            dt.get_day() as i32,
            dt.get_hour() as i32,
            dt.get_minute() as i32,
            dt.get_second() as i32,
            dt.get_microsecond() as i32,
            dt.get_fold() as i32,
        ],
    )
}

#[pyfunction]
#[pyo3(signature=(ts, tz=None))]
fn datetime_from_timestamp<'py>(
    py: Python<'py>,
    ts: f64,
    tz: Option<&Bound<'py, PyTzInfo>>,
) -> PyResult<Bound<'py, PyDateTime>> {
    PyDateTime::from_timestamp(py, ts, tz)
}

#[pyfunction]
fn get_datetime_tzinfo<'py>(dt: &Bound<'py, PyDateTime>) -> Option<Bound<'py, PyTzInfo>> {
    dt.get_tzinfo()
}

#[pyfunction]
fn get_time_tzinfo<'py>(dt: &Bound<'py, PyTime>) -> Option<Bound<'py, PyTzInfo>> {
    dt.get_tzinfo()
}

#[pyclass(extends=PyTzInfo)]
pub struct TzClass {}

#[pymethods]
impl TzClass {
    #[new]
    fn new() -> Self {
        TzClass {}
    }

    fn utcoffset<'py>(&self, dt: &Bound<'py, PyDateTime>) -> PyResult<Bound<'py, PyDelta>> {
        PyDelta::new(dt.py(), 0, 3600, 0, true)
    }

    fn tzname(&self, _dt: &Bound<'_, PyDateTime>) -> String {
        String::from("+01:00")
    }

    fn dst<'py>(&self, _dt: &Bound<'py, PyDateTime>) -> Option<Bound<'py, PyDelta>> {
        None
    }
}

#[pymodule]
pub fn datetime(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(make_date, m)?)?;
    m.add_function(wrap_pyfunction!(get_date_tuple, m)?)?;
    m.add_function(wrap_pyfunction!(date_from_timestamp, m)?)?;
    m.add_function(wrap_pyfunction!(make_time, m)?)?;
    m.add_function(wrap_pyfunction!(get_time_tuple, m)?)?;
    m.add_function(wrap_pyfunction!(make_delta, m)?)?;
    m.add_function(wrap_pyfunction!(get_delta_tuple, m)?)?;
    m.add_function(wrap_pyfunction!(make_datetime, m)?)?;
    m.add_function(wrap_pyfunction!(get_datetime_tuple, m)?)?;
    m.add_function(wrap_pyfunction!(datetime_from_timestamp, m)?)?;
    m.add_function(wrap_pyfunction!(get_datetime_tzinfo, m)?)?;
    m.add_function(wrap_pyfunction!(get_time_tzinfo, m)?)?;

    m.add_function(wrap_pyfunction!(time_with_fold, m)?)?;
    m.add_function(wrap_pyfunction!(get_time_tuple_fold, m)?)?;
    m.add_function(wrap_pyfunction!(get_datetime_tuple_fold, m)?)?;

    m.add_class::<TzClass>()?;

    Ok(())
}
