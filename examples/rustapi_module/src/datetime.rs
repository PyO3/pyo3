use pyo3::prelude::*;
use pyo3::types::{
    PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess, PyTuple,
    PyTzInfo,
};

#[pyfunction]
fn make_date(py: Python, year: i32, month: u8, day: u8) -> PyResult<Py<PyDate>> {
    PyDate::new(py, year, month, day)
}

#[pyfunction]
fn get_date_tuple(py: Python, d: &PyDate) -> Py<PyTuple> {
    PyTuple::new(
        py,
        &[d.get_year(), d.get_month() as i32, d.get_day() as i32],
    )
}

#[pyfunction]
fn date_from_timestamp(py: Python, timestamp: i64) -> PyResult<Py<PyDate>> {
    PyDate::from_timestamp(py, timestamp)
}

#[pyfunction]
fn make_time(
    py: Python,
    hour: u8,
    minute: u8,
    second: u8,
    microsecond: u32,
    tzinfo: Option<&PyTzInfo>,
) -> PyResult<Py<PyTime>> {
    PyTime::new(
        py,
        hour,
        minute,
        second,
        microsecond,
        tzinfo.map(|o| o.to_object(py)).as_ref(),
    )
}

#[cfg(Py_3_6)]
#[pyfunction]
fn time_with_fold(
    py: Python,
    hour: u8,
    minute: u8,
    second: u8,
    microsecond: u32,
    tzinfo: Option<&PyTzInfo>,
    fold: bool,
) -> PyResult<Py<PyTime>> {
    PyTime::new_with_fold(
        py,
        hour,
        minute,
        second,
        microsecond,
        tzinfo.map(|o| o.to_object(py)).as_ref(),
        fold,
    )
}

#[pyfunction]
fn get_time_tuple(py: Python, dt: &PyTime) -> Py<PyTuple> {
    PyTuple::new(
        py,
        &[
            dt.get_hour() as u32,
            dt.get_minute() as u32,
            dt.get_second() as u32,
            dt.get_microsecond(),
        ],
    )
}

#[cfg(Py_3_6)]
#[pyfunction]
fn get_time_tuple_fold(py: Python, dt: &PyTime) -> Py<PyTuple> {
    PyTuple::new(
        py,
        &[
            dt.get_hour() as u32,
            dt.get_minute() as u32,
            dt.get_second() as u32,
            dt.get_microsecond(),
            dt.get_fold() as u32,
        ],
    )
}

#[pyfunction]
fn make_delta(py: Python, days: i32, seconds: i32, microseconds: i32) -> PyResult<Py<PyDelta>> {
    PyDelta::new(py, days, seconds, microseconds, true)
}

#[pyfunction]
fn get_delta_tuple(py: Python, delta: &PyDelta) -> Py<PyTuple> {
    PyTuple::new(
        py,
        &[
            delta.get_days(),
            delta.get_seconds(),
            delta.get_microseconds(),
        ],
    )
}

#[pyfunction]
fn make_datetime(
    py: Python,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    microsecond: u32,
    tzinfo: Option<&PyTzInfo>,
) -> PyResult<Py<PyDateTime>> {
    PyDateTime::new(
        py,
        year,
        month,
        day,
        hour,
        minute,
        second,
        microsecond,
        tzinfo.map(|o| (o.to_object(py))).as_ref(),
    )
}

#[pyfunction]
fn get_datetime_tuple(py: Python, dt: &PyDateTime) -> Py<PyTuple> {
    PyTuple::new(
        py,
        &[
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

#[cfg(Py_3_6)]
#[pyfunction]
fn get_datetime_tuple_fold(py: Python, dt: &PyDateTime) -> Py<PyTuple> {
    PyTuple::new(
        py,
        &[
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
fn datetime_from_timestamp(py: Python, ts: f64, tz: Option<&PyTzInfo>) -> PyResult<Py<PyDateTime>> {
    PyDateTime::from_timestamp(py, ts, tz)
}

#[pyfunction]
fn issue_219() -> PyResult<()> {
    let gil = Python::acquire_gil();
    let _py = gil.python();
    Ok(())
}

#[pyclass(extends=PyTzInfo)]
pub struct TzClass {}

#[pymethods]
impl TzClass {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|| TzClass {})
    }

    fn utcoffset(&self, py: Python, _dt: &PyDateTime) -> PyResult<Py<PyDelta>> {
        PyDelta::new(py, 0, 3600, 0, true)
    }

    fn tzname(&self, _py: Python, _dt: &PyDateTime) -> PyResult<String> {
        Ok(String::from("+01:00"))
    }

    fn dst(&self, _py: Python, _dt: &PyDateTime) -> PyResult<Option<&PyDelta>> {
        Ok(None)
    }
}

#[pymodule]
fn datetime(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_function!(make_date))?;
    m.add_wrapped(wrap_function!(get_date_tuple))?;
    m.add_wrapped(wrap_function!(date_from_timestamp))?;
    m.add_wrapped(wrap_function!(make_time))?;
    m.add_wrapped(wrap_function!(get_time_tuple))?;
    m.add_wrapped(wrap_function!(make_delta))?;
    m.add_wrapped(wrap_function!(get_delta_tuple))?;
    m.add_wrapped(wrap_function!(make_datetime))?;
    m.add_wrapped(wrap_function!(get_datetime_tuple))?;
    m.add_wrapped(wrap_function!(datetime_from_timestamp))?;

    // Python 3.6+ functions
    #[cfg(Py_3_6)]
    {
        m.add_wrapped(wrap_function!(time_with_fold))?;
        m.add_wrapped(wrap_function!(get_time_tuple_fold))?;
        m.add_wrapped(wrap_function!(get_datetime_tuple_fold))?;
    }

    m.add_wrapped(wrap_function!(issue_219))?;

    m.add_class::<TzClass>()?;
    Ok(())
}
