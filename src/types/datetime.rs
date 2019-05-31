//! Safe Rust wrappers for types defined in the Python `datetime` library
//!
//! For more details about these types, see the [Python
//! documentation](https://docs.python.org/3/library/datetime.html)

#![allow(clippy::too_many_arguments)]

use crate::err::PyResult;
use crate::ffi;
#[cfg(PyPy)]
use crate::ffi::datetime::{PyDateTime_FromTimestamp, PyDate_FromTimestamp};
use crate::ffi::PyDateTimeAPI;
use crate::ffi::{PyDateTime_Check, PyDate_Check, PyDelta_Check, PyTZInfo_Check, PyTime_Check};
#[cfg(all(Py_3_6, not(PyPy)))]
use crate::ffi::{PyDateTime_DATE_GET_FOLD, PyDateTime_TIME_GET_FOLD};
use crate::ffi::{
    PyDateTime_DATE_GET_HOUR, PyDateTime_DATE_GET_MICROSECOND, PyDateTime_DATE_GET_MINUTE,
    PyDateTime_DATE_GET_SECOND,
};
use crate::ffi::{
    PyDateTime_DELTA_GET_DAYS, PyDateTime_DELTA_GET_MICROSECONDS, PyDateTime_DELTA_GET_SECONDS,
};
use crate::ffi::{PyDateTime_GET_DAY, PyDateTime_GET_MONTH, PyDateTime_GET_YEAR};
use crate::ffi::{
    PyDateTime_TIME_GET_HOUR, PyDateTime_TIME_GET_MICROSECOND, PyDateTime_TIME_GET_MINUTE,
    PyDateTime_TIME_GET_SECOND,
};
use crate::object::PyObject;
use crate::types::PyTuple;
use crate::AsPyPointer;
use crate::Python;
use crate::ToPyObject;
use std::os::raw::c_int;
#[cfg(not(PyPy))]
use std::ptr;

/// Access traits

/// Trait for accessing the date components of a struct containing a date.
pub trait PyDateAccess {
    fn get_year(&self) -> i32;
    fn get_month(&self) -> u8;
    fn get_day(&self) -> u8;
}

/// Trait for accessing the components of a struct containing a timedelta.
///
/// Note: These access the individual components of a (day, second,
/// microsecond) representation of the delta, they are *not* intended as
/// aliases for calculating the total duration in each of these units.
pub trait PyDeltaAccess {
    fn get_days(&self) -> i32;
    fn get_seconds(&self) -> i32;
    fn get_microseconds(&self) -> i32;
}

/// Trait for accessing the time components of a struct containing a time.
pub trait PyTimeAccess {
    fn get_hour(&self) -> u8;
    fn get_minute(&self) -> u8;
    fn get_second(&self) -> u8;
    fn get_microsecond(&self) -> u32;
    #[cfg(all(Py_3_6, not(PyPy)))]
    fn get_fold(&self) -> u8;
}

/// Bindings around `datetime.date`
pub struct PyDate(PyObject);
pyobject_native_type!(
    PyDate,
    *PyDateTimeAPI.DateType,
    Some("datetime"),
    PyDate_Check
);

impl PyDate {
    pub fn new<'p>(py: Python<'p>, year: i32, month: u8, day: u8) -> PyResult<&'p PyDate> {
        unsafe {
            let ptr = (PyDateTimeAPI.Date_FromDate)(
                year,
                c_int::from(month),
                c_int::from(day),
                PyDateTimeAPI.DateType,
            );
            py.from_owned_ptr_or_err(ptr)
        }
    }

    /// Construct a `datetime.date` from a POSIX timestamp
    ///
    /// This is equivalent to `datetime.date.fromtimestamp`
    pub fn from_timestamp<'p>(py: Python<'p>, timestamp: i64) -> PyResult<&'p PyDate> {
        let time_tuple = PyTuple::new(py, &[timestamp]);

        unsafe {
            #[cfg(PyPy)]
            let ptr = PyDate_FromTimestamp(time_tuple.as_ptr());

            #[cfg(not(PyPy))]
            let ptr =
                (PyDateTimeAPI.Date_FromTimestamp)(PyDateTimeAPI.DateType, time_tuple.as_ptr());

            py.from_owned_ptr_or_err(ptr)
        }
    }
}

impl PyDateAccess for PyDate {
    fn get_year(&self) -> i32 {
        unsafe { PyDateTime_GET_YEAR(self.as_ptr()) as i32 }
    }

    fn get_month(&self) -> u8 {
        unsafe { PyDateTime_GET_MONTH(self.as_ptr()) as u8 }
    }

    fn get_day(&self) -> u8 {
        unsafe { PyDateTime_GET_DAY(self.as_ptr()) as u8 }
    }
}

/// Bindings for `datetime.datetime`
pub struct PyDateTime(PyObject);
pyobject_native_type!(
    PyDateTime,
    *PyDateTimeAPI.DateTimeType,
    Some("datetime"),
    PyDateTime_Check
);

impl PyDateTime {
    pub fn new<'p>(
        py: Python<'p>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
    ) -> PyResult<&'p PyDateTime> {
        unsafe {
            let ptr = (PyDateTimeAPI.DateTime_FromDateAndTime)(
                year,
                c_int::from(month),
                c_int::from(day),
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                opt_to_pyobj(py, tzinfo),
                PyDateTimeAPI.DateTimeType,
            );
            py.from_owned_ptr_or_err(ptr)
        }
    }

    /// Construct a `datetime` object from a POSIX timestamp
    ///
    /// This is equivalent to `datetime.datetime.from_timestamp`
    pub fn from_timestamp<'p>(
        py: Python<'p>,
        timestamp: f64,
        time_zone_info: Option<&PyTzInfo>,
    ) -> PyResult<&'p PyDateTime> {
        let timestamp: PyObject = timestamp.to_object(py);

        let time_zone_info: PyObject = match time_zone_info {
            Some(time_zone_info) => time_zone_info.to_object(py),
            None => py.None(),
        };

        let args = PyTuple::new(py, &[timestamp, time_zone_info]);

        unsafe {
            #[cfg(PyPy)]
            let ptr = PyDateTime_FromTimestamp(args.as_ptr());

            #[cfg(not(PyPy))]
            let ptr = {
                (PyDateTimeAPI.DateTime_FromTimestamp)(
                    PyDateTimeAPI.DateTimeType,
                    args.as_ptr(),
                    ptr::null_mut(),
                )
            };

            py.from_owned_ptr_or_err(ptr)
        }
    }
}

impl PyDateAccess for PyDateTime {
    fn get_year(&self) -> i32 {
        unsafe { PyDateTime_GET_YEAR(self.as_ptr()) as i32 }
    }

    fn get_month(&self) -> u8 {
        unsafe { PyDateTime_GET_MONTH(self.as_ptr()) as u8 }
    }

    fn get_day(&self) -> u8 {
        unsafe { PyDateTime_GET_DAY(self.as_ptr()) as u8 }
    }
}

impl PyTimeAccess for PyDateTime {
    fn get_hour(&self) -> u8 {
        unsafe { PyDateTime_DATE_GET_HOUR(self.as_ptr()) as u8 }
    }

    fn get_minute(&self) -> u8 {
        unsafe { PyDateTime_DATE_GET_MINUTE(self.as_ptr()) as u8 }
    }

    fn get_second(&self) -> u8 {
        unsafe { PyDateTime_DATE_GET_SECOND(self.as_ptr()) as u8 }
    }

    fn get_microsecond(&self) -> u32 {
        unsafe { PyDateTime_DATE_GET_MICROSECOND(self.as_ptr()) as u32 }
    }

    #[cfg(all(Py_3_6, not(PyPy)))]
    fn get_fold(&self) -> u8 {
        unsafe { PyDateTime_DATE_GET_FOLD(self.as_ptr()) as u8 }
    }
}

/// Bindings for `datetime.time`
pub struct PyTime(PyObject);
pyobject_native_type!(
    PyTime,
    *PyDateTimeAPI.TimeType,
    Some("datetime"),
    PyTime_Check
);

impl PyTime {
    pub fn new<'p>(
        py: Python<'p>,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
    ) -> PyResult<&'p PyTime> {
        unsafe {
            let ptr = (PyDateTimeAPI.Time_FromTime)(
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                opt_to_pyobj(py, tzinfo),
                PyDateTimeAPI.TimeType,
            );
            py.from_owned_ptr_or_err(ptr)
        }
    }

    #[cfg(Py_3_6)]
    /// Alternate constructor that takes a `fold` argument
    ///
    /// First available in Python 3.6.
    pub fn new_with_fold<'p>(
        py: Python<'p>,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
        fold: bool,
    ) -> PyResult<&'p PyTime> {
        unsafe {
            let ptr = (PyDateTimeAPI.Time_FromTimeAndFold)(
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                opt_to_pyobj(py, tzinfo),
                fold as c_int,
                PyDateTimeAPI.TimeType,
            );
            py.from_owned_ptr_or_err(ptr)
        }
    }
}

impl PyTimeAccess for PyTime {
    fn get_hour(&self) -> u8 {
        unsafe { PyDateTime_TIME_GET_HOUR(self.as_ptr()) as u8 }
    }

    fn get_minute(&self) -> u8 {
        unsafe { PyDateTime_TIME_GET_MINUTE(self.as_ptr()) as u8 }
    }

    fn get_second(&self) -> u8 {
        unsafe { PyDateTime_TIME_GET_SECOND(self.as_ptr()) as u8 }
    }

    fn get_microsecond(&self) -> u32 {
        unsafe { PyDateTime_TIME_GET_MICROSECOND(self.as_ptr()) as u32 }
    }

    #[cfg(all(Py_3_6, not(PyPy)))]
    fn get_fold(&self) -> u8 {
        unsafe { PyDateTime_TIME_GET_FOLD(self.as_ptr()) as u8 }
    }
}

/// Bindings for `datetime.tzinfo`
///
/// This is an abstract base class and should not be constructed directly.
pub struct PyTzInfo(PyObject);
pyobject_native_type!(
    PyTzInfo,
    *PyDateTimeAPI.TZInfoType,
    Some("datetime"),
    PyTZInfo_Check
);

/// Bindings for `datetime.timedelta`
pub struct PyDelta(PyObject);
pyobject_native_type!(
    PyDelta,
    *PyDateTimeAPI.DeltaType,
    Some("datetime"),
    PyDelta_Check
);

impl PyDelta {
    pub fn new<'p>(
        py: Python<'p>,
        days: i32,
        seconds: i32,
        microseconds: i32,
        normalize: bool,
    ) -> PyResult<&'p PyDelta> {
        unsafe {
            let ptr = (PyDateTimeAPI.Delta_FromDelta)(
                days as c_int,
                seconds as c_int,
                microseconds as c_int,
                normalize as c_int,
                PyDateTimeAPI.DeltaType,
            );
            py.from_owned_ptr_or_err(ptr)
        }
    }
}

impl PyDeltaAccess for PyDelta {
    fn get_days(&self) -> i32 {
        unsafe { PyDateTime_DELTA_GET_DAYS(self.as_ptr()) as i32 }
    }

    fn get_seconds(&self) -> i32 {
        unsafe { PyDateTime_DELTA_GET_SECONDS(self.as_ptr()) as i32 }
    }

    fn get_microseconds(&self) -> i32 {
        unsafe { PyDateTime_DELTA_GET_MICROSECONDS(self.as_ptr()) as i32 }
    }
}

// Utility function
unsafe fn opt_to_pyobj(py: Python, opt: Option<&PyObject>) -> *mut ffi::PyObject {
    // Convenience function for unpacking Options to either an Object or None
    match opt {
        Some(tzi) => tzi.as_ptr(),
        None => py.None().as_ptr(),
    }
}
