#![cfg(not(Py_LIMITED_API))]

use pyo3::types::{IntoPyDict, PyDate, PyDateTime, PyTime, PyTzInfo};
use pyo3::{ffi, prelude::*};
use pyo3_ffi::PyDateTime_IMPORT;
use std::ffi::CString;

fn _get_subclasses<'py>(
    py: Python<'py>,
    py_type: &str,
    args: &str,
) -> PyResult<(Bound<'py, PyAny>, Bound<'py, PyAny>, Bound<'py, PyAny>)> {
    // Import the class from Python and create some subclasses
    let datetime = py.import("datetime")?;

    let locals = [(py_type, datetime.getattr(py_type)?)]
        .into_py_dict(py)
        .unwrap();

    let make_subclass_py = CString::new(format!("class Subklass({py_type}):\n    pass"))?;

    let make_sub_subclass_py = ffi::c_str!("class SubSubklass(Subklass):\n    pass");

    py.run(&make_subclass_py, None, Some(&locals))?;
    py.run(make_sub_subclass_py, None, Some(&locals))?;

    // Construct an instance of the base class
    let obj = py.eval(
        &CString::new(format!("{py_type}({args})"))?,
        None,
        Some(&locals),
    )?;

    // Construct an instance of the subclass
    let sub_obj = py.eval(
        &CString::new(format!("Subklass({args})"))?,
        None,
        Some(&locals),
    )?;

    // Construct an instance of the sub-subclass
    let sub_sub_obj = py.eval(
        &CString::new(format!("SubSubklass({args})"))?,
        None,
        Some(&locals),
    )?;

    Ok((obj, sub_obj, sub_sub_obj))
}

macro_rules! assert_check_exact {
    ($check_func:ident, $check_func_exact:ident, $obj: expr) => {
        unsafe {
            use pyo3::ffi::*;
            assert!($check_func(($obj).as_ptr()) != 0);
            assert!($check_func_exact(($obj).as_ptr()) != 0);
        }
    };
}

macro_rules! assert_check_only {
    ($check_func:ident, $check_func_exact:ident, $obj: expr) => {
        unsafe {
            use pyo3::ffi::*;
            assert!($check_func(($obj).as_ptr()) != 0);
            assert!($check_func_exact(($obj).as_ptr()) == 0);
        }
    };
}

#[test]
fn test_date_check() {
    Python::attach(|py| {
        let (obj, sub_obj, sub_sub_obj) = _get_subclasses(py, "date", "2018, 1, 1").unwrap();
        unsafe { PyDateTime_IMPORT() }
        assert_check_exact!(PyDate_Check, PyDate_CheckExact, obj);
        assert_check_only!(PyDate_Check, PyDate_CheckExact, sub_obj);
        assert_check_only!(PyDate_Check, PyDate_CheckExact, sub_sub_obj);
        assert!(obj.is_instance_of::<PyDate>());
        assert!(!obj.is_instance_of::<PyTime>());
        assert!(!obj.is_instance_of::<PyDateTime>());
    });
}

#[test]
fn test_time_check() {
    Python::attach(|py| {
        let (obj, sub_obj, sub_sub_obj) = _get_subclasses(py, "time", "12, 30, 15").unwrap();
        unsafe { PyDateTime_IMPORT() }

        assert_check_exact!(PyTime_Check, PyTime_CheckExact, obj);
        assert_check_only!(PyTime_Check, PyTime_CheckExact, sub_obj);
        assert_check_only!(PyTime_Check, PyTime_CheckExact, sub_sub_obj);
        assert!(!obj.is_instance_of::<PyDate>());
        assert!(obj.is_instance_of::<PyTime>());
        assert!(!obj.is_instance_of::<PyDateTime>());
    });
}

#[test]
fn test_datetime_check() {
    Python::attach(|py| {
        let (obj, sub_obj, sub_sub_obj) = _get_subclasses(py, "datetime", "2018, 1, 1, 13, 30, 15")
            .map_err(|e| e.display(py))
            .unwrap();
        unsafe { PyDateTime_IMPORT() }

        assert_check_only!(PyDate_Check, PyDate_CheckExact, obj);
        assert_check_exact!(PyDateTime_Check, PyDateTime_CheckExact, obj);
        assert_check_only!(PyDateTime_Check, PyDateTime_CheckExact, sub_obj);
        assert_check_only!(PyDateTime_Check, PyDateTime_CheckExact, sub_sub_obj);
        assert!(obj.is_instance_of::<PyDate>());
        assert!(!obj.is_instance_of::<PyTime>());
        assert!(obj.is_instance_of::<PyDateTime>());
    });
}

#[test]
fn test_delta_check() {
    Python::attach(|py| {
        let (obj, sub_obj, sub_sub_obj) = _get_subclasses(py, "timedelta", "1, -3").unwrap();
        unsafe { PyDateTime_IMPORT() }

        assert_check_exact!(PyDelta_Check, PyDelta_CheckExact, obj);
        assert_check_only!(PyDelta_Check, PyDelta_CheckExact, sub_obj);
        assert_check_only!(PyDelta_Check, PyDelta_CheckExact, sub_sub_obj);
    });
}

#[test]
fn test_datetime_utc() {
    use assert_approx_eq::assert_approx_eq;
    use pyo3::types::PyDateTime;

    Python::attach(|py| {
        let utc = PyTzInfo::utc(py).unwrap();

        let dt = PyDateTime::new(py, 2018, 1, 1, 0, 0, 0, 0, Some(&utc)).unwrap();

        let locals = [("dt", dt)].into_py_dict(py).unwrap();

        let offset: f32 = py
            .eval(
                ffi::c_str!("dt.utcoffset().total_seconds()"),
                None,
                Some(&locals),
            )
            .unwrap()
            .extract()
            .unwrap();
        assert_approx_eq!(offset, 0f32);
    });
}

static INVALID_DATES: &[(i32, u8, u8)] = &[
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

static INVALID_TIMES: &[(u8, u8, u8, u32)] =
    &[(25, 0, 0, 0), (255, 0, 0, 0), (0, 60, 0, 0), (0, 0, 61, 0)];

#[test]
fn test_pydate_out_of_bounds() {
    use pyo3::types::PyDate;

    Python::attach(|py| {
        for val in INVALID_DATES {
            let (year, month, day) = val;
            let dt = PyDate::new(py, *year, *month, *day);
            dt.unwrap_err();
        }
    });
}

#[test]
fn test_pytime_out_of_bounds() {
    use pyo3::types::PyTime;

    Python::attach(|py| {
        for val in INVALID_TIMES {
            let (hour, minute, second, microsecond) = val;
            let dt = PyTime::new(py, *hour, *minute, *second, *microsecond, None);
            dt.unwrap_err();
        }
    });
}

#[test]
fn test_pydatetime_out_of_bounds() {
    use pyo3::types::PyDateTime;
    use std::iter;

    Python::attach(|py| {
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
    });
}
