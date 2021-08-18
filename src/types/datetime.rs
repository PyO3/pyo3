//! Safe Rust wrappers for types defined in the Python `datetime` library
//!
//! For more details about these types, see the [Python
//! documentation](https://docs.python.org/3/library/datetime.html)

use crate::err::PyResult;
use crate::ffi;
#[cfg(PyPy)]
use crate::ffi::datetime::{PyDateTime_FromTimestamp, PyDate_FromTimestamp};
use crate::ffi::PyDateTimeAPI;
use crate::ffi::{PyDateTime_Check, PyDate_Check, PyDelta_Check, PyTZInfo_Check, PyTime_Check};
#[cfg(not(PyPy))]
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
use crate::types::PyTuple;
use crate::{AsPyPointer, PyAny, PyObject, Python, ToPyObject};
use std::os::raw::c_int;
#[cfg(not(PyPy))]
use std::ptr;

// Access traits

/// Trait for accessing the date components of a struct containing a date.
pub trait PyDateAccess {
    /// Returns the year, as a positive int.
    ///
    /// Implementations should conform to the upstream documentation:
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_GET_YEAR>
    fn get_year(&self) -> i32;
    /// Returns the month, as an int from 1 through 12.
    ///
    /// Implementations should conform to the upstream documentation:
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_GET_MONTH>
    fn get_month(&self) -> u8;
    /// Returns the day, as an int from 1 through 31.
    ///
    /// Implementations should conform to the upstream documentation:
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_GET_DAY>
    fn get_day(&self) -> u8;
}

/// Trait for accessing the components of a struct containing a timedelta.
///
/// Note: These access the individual components of a (day, second,
/// microsecond) representation of the delta, they are *not* intended as
/// aliases for calculating the total duration in each of these units.
pub trait PyDeltaAccess {
    /// Returns the number of days, as an int from -999999999 to 999999999.
    ///
    /// Implementations should conform to the upstream documentation:
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_DELTA_GET_DAYS>
    fn get_days(&self) -> i32;
    /// Returns the number of seconds, as an int from 0 through 86399.
    ///
    /// Implementations should conform to the upstream documentation:
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_DELTA_GET_DAYS>
    fn get_seconds(&self) -> i32;
    /// Returns the number of microseconds, as an int from 0 through 999999.
    ///
    /// Implementations should conform to the upstream documentation:
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_DELTA_GET_DAYS>
    fn get_microseconds(&self) -> i32;
}

/// Trait for accessing the time components of a struct containing a time.
pub trait PyTimeAccess {
    /// Returns the hour, as an int from 0 through 23.
    ///
    /// Implementations should conform to the upstream documentation:
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_DATE_GET_HOUR>
    fn get_hour(&self) -> u8;
    /// Returns the minute, as an int from 0 through 59.
    ///
    /// Implementations should conform to the upstream documentation:
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_DATE_GET_MINUTE>
    fn get_minute(&self) -> u8;
    /// Returns the second, as an int from 0 through 59.
    ///
    /// Implementations should conform to the upstream documentation:
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_DATE_GET_SECOND>
    fn get_second(&self) -> u8;
    /// Returns the microsecond, as an int from 0 through 999999.
    ///
    /// Implementations should conform to the upstream documentation:
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_DATE_GET_MICROSECOND>
    fn get_microsecond(&self) -> u32;
    /// Returns whether this date is the later of two moments with the
    /// same representation, during a repeated interval.
    ///
    /// This typically occurs at the end of daylight savings time, or during
    /// leap seconds. Only valid if the represented time is ambiguous. See
    /// [PEP 495](https://www.python.org/dev/peps/pep-0495/) for more detail.
    #[cfg(not(PyPy))]
    fn get_fold(&self) -> bool;
}

/// Bindings around `datetime.date`
#[repr(transparent)]
pub struct PyDate(PyAny);
pyobject_native_type!(
    PyDate,
    crate::ffi::PyDateTime_Date,
    *PyDateTimeAPI.DateType,
    #module=Some("datetime"),
    #checkfunction=PyDate_Check
);

impl PyDate {
    /// Creates a new `datetime.date`.
    pub fn new(py: Python, year: i32, month: u8, day: u8) -> PyResult<&PyDate> {
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
    pub fn from_timestamp(py: Python, timestamp: i64) -> PyResult<&PyDate> {
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
#[repr(transparent)]
pub struct PyDateTime(PyAny);
pyobject_native_type!(
    PyDateTime,
    crate::ffi::PyDateTime_DateTime,
    *PyDateTimeAPI.DateTimeType,
    #module=Some("datetime"),
    #checkfunction=PyDateTime_Check
);

impl PyDateTime {
    #[allow(clippy::too_many_arguments)]
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

    /// Alternate constructor that takes a `fold` parameter. A `true` value for this parameter
    /// signifies a leap second
    #[cfg(not(PyPy))]
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_fold<'p>(
        py: Python<'p>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
        fold: bool,
    ) -> PyResult<&'p PyDateTime> {
        unsafe {
            let ptr = (PyDateTimeAPI.DateTime_FromDateAndTimeAndFold)(
                year,
                c_int::from(month),
                c_int::from(day),
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                opt_to_pyobj(py, tzinfo),
                c_int::from(fold),
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

    #[cfg(not(PyPy))]
    fn get_fold(&self) -> bool {
        unsafe { PyDateTime_DATE_GET_FOLD(self.as_ptr()) > 0 }
    }
}

/// Bindings for `datetime.time`
#[repr(transparent)]
pub struct PyTime(PyAny);
pyobject_native_type!(
    PyTime,
    crate::ffi::PyDateTime_Time,
    *PyDateTimeAPI.TimeType,
    #module=Some("datetime"),
    #checkfunction=PyTime_Check
);

impl PyTime {
    /// Creates a new `datetime.time` object.
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

    #[cfg(not(PyPy))]
    /// Alternate constructor that takes a `fold` argument
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

    #[cfg(not(PyPy))]
    fn get_fold(&self) -> bool {
        unsafe { PyDateTime_TIME_GET_FOLD(self.as_ptr()) != 0 }
    }
}

/// Bindings for `datetime.tzinfo`
///
/// This is an abstract base class and should not be constructed directly.
#[repr(transparent)]
pub struct PyTzInfo(PyAny);
pyobject_native_type!(
    PyTzInfo,
    crate::ffi::PyObject,
    *PyDateTimeAPI.TZInfoType,
    #module=Some("datetime"),
    #checkfunction=PyTZInfo_Check
);

/// Bindings for `datetime.timedelta`
#[repr(transparent)]
pub struct PyDelta(PyAny);
pyobject_native_type!(
    PyDelta,
    crate::ffi::PyDateTime_Delta,
    *PyDateTimeAPI.DeltaType,
    #module=Some("datetime"),
    #checkfunction=PyDelta_Check
);

impl PyDelta {
    /// Creates a new `timedelta`.
    pub fn new(
        py: Python,
        days: i32,
        seconds: i32,
        microseconds: i32,
        normalize: bool,
    ) -> PyResult<&PyDelta> {
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
fn opt_to_pyobj(py: Python, opt: Option<&PyObject>) -> *mut ffi::PyObject {
    // Convenience function for unpacking Options to either an Object or None
    match opt {
        Some(tzi) => tzi.as_ptr(),
        None => py.None().as_ptr(),
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(PyPy))]
    #[test]
    fn test_new_with_fold() {
        crate::Python::with_gil(|py| {
            use crate::types::{PyDateTime, PyTimeAccess};

            let a = PyDateTime::new_with_fold(py, 2021, 1, 23, 20, 32, 40, 341516, None, false);
            let b = PyDateTime::new_with_fold(py, 2021, 1, 23, 20, 32, 40, 341516, None, true);

            assert!(!a.unwrap().get_fold());
            assert!(b.unwrap().get_fold());
        });
    }
}
