#![feature(concat_idents)]

use pyo3::ffi::*;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyAny};

fn _get_subclasses<'p>(
    py: &'p Python,
    py_type: &str,
    args: &str,
) -> PyResult<(&'p PyAny, &'p PyAny, &'p PyAny)> {
    // Import the class from Python and create some subclasses
    let datetime = py.import("datetime")?;

    let locals = [(py_type, datetime.get(py_type)?)].into_py_dict(*py);

    let make_subclass_py = format!("class Subklass({}):\n    pass", py_type);

    let make_sub_subclass_py = "class SubSubklass(Subklass):\n    pass";

    py.run(&make_subclass_py, None, Some(&locals))?;
    py.run(&make_sub_subclass_py, None, Some(&locals))?;

    // Construct an instance of the base class
    let obj = py.eval(&format!("{}({})", py_type, args), None, Some(&locals))?;

    // Construct an instance of the subclass
    let sub_obj = py.eval(&format!("Subklass({})", args), None, Some(&locals))?;

    // Construct an instance of the sub-subclass
    let sub_sub_obj = py.eval(&format!("SubSubklass({})", args), None, Some(&locals))?;

    Ok((obj, sub_obj, sub_sub_obj))
}

macro_rules! assert_check_exact {
    ($check_func:ident, $obj: expr) => {
        unsafe {
            use pyo3::AsPyPointer;
            assert!($check_func(($obj).as_ptr()) != 0);
            assert!(concat_idents!($check_func, Exact)(($obj).as_ptr()) != 0);
        }
    };
}

macro_rules! assert_check_only {
    ($check_func:ident, $obj: expr) => {
        unsafe {
            use pyo3::AsPyPointer;
            assert!($check_func(($obj).as_ptr()) != 0);
            assert!(concat_idents!($check_func, Exact)(($obj).as_ptr()) == 0);
        }
    };
}

#[test]
fn test_date_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) = _get_subclasses(&py, "date", "2018, 1, 1").unwrap();

    assert_check_exact!(PyDate_Check, obj);
    assert_check_only!(PyDate_Check, sub_obj);
    assert_check_only!(PyDate_Check, sub_sub_obj);
}

#[test]
fn test_time_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) = _get_subclasses(&py, "time", "12, 30, 15").unwrap();

    assert_check_exact!(PyTime_Check, obj);
    assert_check_only!(PyTime_Check, sub_obj);
    assert_check_only!(PyTime_Check, sub_sub_obj);
}

#[test]
fn test_datetime_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) =
        _get_subclasses(&py, "datetime", "2018, 1, 1, 13, 30, 15").unwrap();

    assert_check_only!(PyDate_Check, obj);
    assert_check_exact!(PyDateTime_Check, obj);
    assert_check_only!(PyDateTime_Check, sub_obj);
    assert_check_only!(PyDateTime_Check, sub_sub_obj);
}

#[test]
fn test_delta_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) = _get_subclasses(&py, "timedelta", "1, -3").unwrap();

    assert_check_exact!(PyDelta_Check, obj);
    assert_check_only!(PyDelta_Check, sub_obj);
    assert_check_only!(PyDelta_Check, sub_sub_obj);
}

#[test]
fn test_datetime_utc() {
    use pyo3::types::PyDateTime;

    let gil = Python::acquire_gil();
    let py = gil.python();

    let datetime = py.import("datetime").map_err(|e| e.print(py)).unwrap();
    let timezone = datetime.get("timezone").unwrap();
    let utc = timezone.getattr("utc").unwrap().to_object(py);

    let dt = PyDateTime::new(py, 2018, 1, 1, 0, 0, 0, 0, Some(&utc)).unwrap();

    let locals = [("dt", dt)].into_py_dict(py);

    let offset: f32 = py
        .eval("dt.utcoffset().total_seconds()", None, Some(locals))
        .unwrap()
        .extract()
        .unwrap();
    assert_eq!(offset, 0f32);
}

static INVALID_DATES: &'static [(i32, u8, u8)] = &[
    (-1, 1, 1),
    (0, 1, 1),
    (10000, 1, 1),
    (2 << 30, 1, 1),
    (2018, 0, 1),
    (2018, 13, 1),
    (2018, 1, 0),
    (2017, 2, 29),
    (2018, 1, 32),
];

static INVALID_TIMES: &'static [(u8, u8, u8, u32)] =
    &[(25, 0, 0, 0), (255, 0, 0, 0), (0, 60, 0, 0), (0, 0, 61, 0)];

#[cfg(Py_3_6)]
#[test]
fn test_pydate_out_of_bounds() {
    use pyo3::types::PyDate;

    // This test is an XFAIL on Python < 3.6 until bounds checking is implemented
    let gil = Python::acquire_gil();
    let py = gil.python();
    for val in INVALID_DATES.into_iter() {
        let (year, month, day) = val;
        let dt = PyDate::new(py, *year, *month, *day);
        dt.unwrap_err();
    }
}

#[cfg(Py_3_6)]
#[test]
fn test_pytime_out_of_bounds() {
    use pyo3::types::PyTime;

    // This test is an XFAIL on Python < 3.6 until bounds checking is implemented
    let gil = Python::acquire_gil();
    let py = gil.python();
    for val in INVALID_TIMES {
        let (hour, minute, second, microsecond) = val;
        let dt = PyTime::new(py, *hour, *minute, *second, *microsecond, None);
        dt.unwrap_err();
    }
}

#[cfg(Py_3_6)]
#[test]
fn test_pydatetime_out_of_bounds() {
    use pyo3::types::PyDateTime;
    use std::iter;

    // This test is an XFAIL on Python < 3.6 until bounds checking is implemented
    let gil = Python::acquire_gil();
    let py = gil.python();
    let valid_time = (0, 0, 0, 0);
    let valid_date = (2018, 1, 1);

    let invalid_dates = INVALID_DATES.iter().zip(iter::repeat(&valid_time));
    let invalid_times = iter::repeat(&valid_date).zip(INVALID_TIMES.iter());

    let vals = invalid_dates.chain(invalid_times);

    for val in vals {
        let (date, time) = val;
        let (year, month, day) = date;
        let (hour, minute, second, microsecond) = time;
        let dt = PyDateTime::new(
            py,
            *year,
            *month,
            *day,
            *hour,
            *minute,
            *second,
            *microsecond,
            None,
        );
        dt.unwrap_err();
    }
}
