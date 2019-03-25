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
#[cfg(Py_3_6)]
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
use crate::instance::Py;
use crate::object::PyObject;
use crate::types::PyTuple;
use crate::AsPyPointer;
use crate::Python;
use crate::ToPyObject;
use std::os::raw::c_int;
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
    #[cfg(Py_3_6)]
    fn get_fold(&self) -> u8;
}

/// Bindings around `datetime.date`
pub struct PyDate(PyObject);
pyobject_native_type!(PyDate, *PyDateTimeAPI.DateType, PyDate_Check);

impl PyDate {
    pub fn new(py: Python, year: i32, month: u8, day: u8) -> PyResult<Py<PyDate>> {
        unsafe {
            let ptr = (PyDateTimeAPI.Date_FromDate)(
                year,
                c_int::from(month),
                c_int::from(day),
                PyDateTimeAPI.DateType,
            );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    /// Construct a `datetime.date` from a POSIX timestamp
    ///
    /// This is equivalent to `datetime.date.fromtimestamp`
    pub fn from_timestamp(py: Python, timestamp: i64) -> PyResult<Py<PyDate>> {
        let time_tuple = PyTuple::new(py, &[timestamp]);

        unsafe {
            #[cfg(PyPy)]
            let ptr = PyDate_FromTimestamp(time_tuple.as_ptr());

            #[cfg(not(PyPy))]
            let ptr =
                (PyDateTimeAPI.Date_FromTimestamp)(PyDateTimeAPI.DateType, time_tuple.as_ptr());

            unsafe { Py::from_owned_ptr_or_err(py, ptr) }
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
pyobject_native_type!(PyDateTime, *PyDateTimeAPI.DateTimeType, PyDateTime_Check);

impl PyDateTime {
    pub fn new(
        py: Python,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
    ) -> PyResult<Py<PyDateTime>> {
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
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    /// Construct a `datetime` object from a POSIX timestamp
    ///
    /// This is equivalent to `datetime.datetime.from_timestamp`
    pub fn from_timestamp(
        py: Python,
        timestamp: f64,
        time_zone_info: Option<&PyTzInfo>,
    ) -> PyResult<Py<PyDateTime>> {
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

            Py::from_owned_ptr_or_err(py, ptr)
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

    #[cfg(Py_3_6)]
    fn get_fold(&self) -> u8 {
        unsafe { PyDateTime_DATE_GET_FOLD(self.as_ptr()) as u8 }
    }
}

/// Bindings for `datetime.time`
pub struct PyTime(PyObject);
pyobject_native_type!(PyTime, *PyDateTimeAPI.TimeType, PyTime_Check);

impl PyTime {
    pub fn new(
        py: Python,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
    ) -> PyResult<Py<PyTime>> {
        unsafe {
            let ptr = (PyDateTimeAPI.Time_FromTime)(
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                opt_to_pyobj(py, tzinfo),
                PyDateTimeAPI.TimeType,
            );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    #[cfg(Py_3_6)]
    /// Alternate constructor that takes a `fold` argument
    ///
    /// First available in Python 3.6.
    pub fn new_with_fold(
        py: Python,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
        fold: bool,
    ) -> PyResult<Py<PyTime>> {
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
            Py::from_owned_ptr_or_err(py, ptr)
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

    #[cfg(Py_3_6)]
    fn get_fold(&self) -> u8 {
        unsafe { PyDateTime_TIME_GET_FOLD(self.as_ptr()) as u8 }
    }
}

/// Bindings for `datetime.tzinfo`
///
/// This is an abstract base class and should not be constructed directly.
pub struct PyTzInfo(PyObject);
pyobject_native_type!(PyTzInfo, *PyDateTimeAPI.TZInfoType, PyTZInfo_Check);

/// Bindings for `datetime.timedelta`
pub struct PyDelta(PyObject);
pyobject_native_type!(PyDelta, *PyDateTimeAPI.DeltaType, PyDelta_Check);

impl PyDelta {
    pub fn new(
        py: Python,
        days: i32,
        seconds: i32,
        microseconds: i32,
        normalize: bool,
    ) -> PyResult<Py<PyDelta>> {
        unsafe {
            let ptr = (PyDateTimeAPI.Delta_FromDelta)(
                days as c_int,
                seconds as c_int,
                microseconds as c_int,
                normalize as c_int,
                PyDateTimeAPI.DeltaType,
            );
            Py::from_owned_ptr_or_err(py, ptr)
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
