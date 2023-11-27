#![cfg(not(Py_LIMITED_API))]

use pyo3::prelude::*;
use pyo3::types::{
    PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess, PyTuple,
    PyTzInfo, PyTzInfoAccess,
};

#[pyfunction]
fn make_date(py: Python<'_>, year: i32, month: u8, day: u8) -> PyResult<Bound<'_, PyDate>> {
    PyDate::new_bound(py, year, month, day)
}

#[pyfunction]
fn get_date_tuple<'p>(py: Python<'p>, d: &PyDate) -> Bound<'p, PyTuple> {
    PyTuple::new_bound(py, [d.get_year(), d.get_month() as i32, d.get_day() as i32])
}

#[pyfunction]
fn date_from_timestamp(py: Python<'_>, timestamp: i64) -> PyResult<Bound<'_, PyDate>> {
    PyDate::from_timestamp_bound(py, timestamp)
}

#[pyfunction]
fn make_time<'py>(
    py: Python<'py>,
    hour: u8,
    minute: u8,
    second: u8,
    microsecond: u32,
    tzinfo: Option<&Bound<'_, PyTzInfo>>,
) -> PyResult<Bound<'py, PyTime>> {
    PyTime::new_bound(py, hour, minute, second, microsecond, tzinfo)
}

#[pyfunction]
#[pyo3(signature = (hour, minute, second, microsecond, tzinfo, fold))]
fn time_with_fold<'py>(
    py: Python<'py>,
    hour: u8,
    minute: u8,
    second: u8,
    microsecond: u32,
    tzinfo: Option<&Bound<'_, PyTzInfo>>,
    fold: bool,
) -> PyResult<Bound<'py, PyTime>> {
    PyTime::new_bound_with_fold(py, hour, minute, second, microsecond, tzinfo, fold)
}

#[pyfunction]
fn get_time_tuple<'p>(py: Python<'p>, dt: &PyTime) -> Bound<'p, PyTuple> {
    PyTuple::new_bound(
        py,
        [
            dt.get_hour() as u32,
            dt.get_minute() as u32,
            dt.get_second() as u32,
            dt.get_microsecond(),
        ],
    )
}

#[pyfunction]
fn get_time_tuple_fold<'p>(py: Python<'p>, dt: &PyTime) -> Bound<'p, PyTuple> {
    PyTuple::new_bound(
        py,
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
    PyDelta::new_bound(py, days, seconds, microseconds, true)
}

#[pyfunction]
fn get_delta_tuple<'py>(py: Python<'py>, delta: &Bound<'_, PyDelta>) -> Bound<'py, PyTuple> {
    PyTuple::new_bound(
        py,
        [
            delta.get_days(),
            delta.get_seconds(),
            delta.get_microseconds(),
        ],
    )
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
fn make_datetime<'py>(
    py: Python<'py>,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    microsecond: u32,
    tzinfo: Option<&Bound<'_, PyTzInfo>>,
) -> PyResult<Bound<'py, PyDateTime>> {
    PyDateTime::new_bound(
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
fn get_datetime_tuple<'py>(py: Python<'py>, dt: &Bound<'_, PyDateTime>) -> Bound<'py, PyTuple> {
    PyTuple::new_bound(
        py,
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
fn get_datetime_tuple_fold<'py>(
    py: Python<'py>,
    dt: &Bound<'_, PyDateTime>,
) -> Bound<'py, PyTuple> {
    PyTuple::new_bound(
        py,
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
fn datetime_from_timestamp<'py>(
    py: Python<'py>,
    ts: f64,
    tz: Option<&Bound<'_, PyTzInfo>>,
) -> PyResult<Bound<'py, PyDateTime>> {
    PyDateTime::from_timestamp_bound(py, ts, tz)
}

#[pyfunction]
fn get_datetime_tzinfo(dt: &PyDateTime) -> Option<Bound<'_, PyTzInfo>> {
    dt.get_tzinfo_bound()
}

#[pyfunction]
fn get_time_tzinfo(dt: &PyTime) -> Option<Bound<'_, PyTzInfo>> {
    dt.get_tzinfo_bound()
}

#[pyclass(extends=PyTzInfo)]
pub struct TzClass {}

#[pymethods]
impl TzClass {
    #[new]
    fn new() -> Self {
        TzClass {}
    }

    fn utcoffset<'py>(
        &self,
        py: Python<'py>,
        _dt: &Bound<'_, PyDateTime>,
    ) -> PyResult<Bound<'py, PyDelta>> {
        PyDelta::new_bound(py, 0, 3600, 0, true)
    }

    fn tzname(&self, _dt: &Bound<'_, PyDateTime>) -> String {
        String::from("+01:00")
    }

    fn dst(&self, _dt: &Bound<'_, PyDateTime>) -> Option<&PyDelta> {
        None
    }
}

#[pymodule]
pub fn datetime(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
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
