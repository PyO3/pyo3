//! Safe Rust wrappers for types defined in the Python `datetime` library
//!
//! For more details about these types, see the [Python
//! documentation](https://docs.python.org/3/library/datetime.html)

#![allow(clippy::too_many_arguments)]

use crate::{
    ffi,
    objects::{PyTuple, PyAny},
    owned::PyOwned,
    types::{Date, DateTime, Time, TimeDelta, TzInfo},
    AsPyPointer, PyObject, PyResult, Python, ToPyObject,
};
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
    #[cfg(not(PyPy))]
    fn get_fold(&self) -> u8;
}

/// Bindings around `datetime.date`
#[repr(transparent)]
pub struct PyDate<'py>(Date, Python<'py>);

pyo3_native_object!(PyDate<'py>, Date, 'py);

impl<'py> PyDate<'py> {
    pub fn new<'p>(py: Python<'py>, year: i32, month: u8, day: u8) -> PyResult<PyOwned<'py, Date>> {
        unsafe {
            let ptr = (ffi::PyDateTimeAPI.Date_FromDate)(
                year,
                c_int::from(month),
                c_int::from(day),
                ffi::PyDateTimeAPI.DateType,
            );
            PyOwned::from_raw_or_fetch_err(py, ptr)
        }
    }

    /// Construct a `datetime.date` from a POSIX timestamp
    ///
    /// This is equivalent to `datetime.date.fromtimestamp`
    pub fn from_timestamp(py: Python<'py>, timestamp: i64) -> PyResult<PyOwned<'py, Date>> {
        let time_tuple = PyTuple::new(py, &[timestamp]);

        unsafe {
            #[cfg(PyPy)]
            let ptr = PyDate_FromTimestamp(time_tuple.as_ptr());

            #[cfg(not(PyPy))]
            let ptr = (ffi::PyDateTimeAPI.Date_FromTimestamp)(
                ffi::PyDateTimeAPI.DateType,
                time_tuple.as_ptr(),
            );

            PyOwned::from_raw_or_fetch_err(py, ptr)
        }
    }
}

impl PyDateAccess for PyDate<'_> {
    fn get_year(&self) -> i32 {
        unsafe { ffi::PyDateTime_GET_YEAR(self.as_ptr()) as i32 }
    }

    fn get_month(&self) -> u8 {
        unsafe { ffi::PyDateTime_GET_MONTH(self.as_ptr()) as u8 }
    }

    fn get_day(&self) -> u8 {
        unsafe { ffi::PyDateTime_GET_DAY(self.as_ptr()) as u8 }
    }
}

/// Bindings for `datetime.datetime`
#[repr(transparent)]
pub struct PyDateTime<'py>(DateTime, Python<'py>);
pyo3_native_object!(PyDateTime<'py>, DateTime, 'py);

impl<'py> PyDateTime<'py> {
    pub fn new(
        py: Python<'py>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyAny>,
    ) -> PyResult<PyOwned<'py, DateTime>> {
        unsafe {
            let ptr = (ffi::PyDateTimeAPI.DateTime_FromDateAndTime)(
                year,
                c_int::from(month),
                c_int::from(day),
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                tzinfo.map_or(std::ptr::null_mut(), |any| any.as_ptr()),
                ffi::PyDateTimeAPI.DateTimeType,
            );
            PyOwned::from_raw_or_fetch_err(py, ptr)
        }
    }

    /// Construct a `datetime` object from a POSIX timestamp
    ///
    /// This is equivalent to `datetime.datetime.from_timestamp`
    pub fn from_timestamp(
        py: Python<'py>,
        timestamp: f64,
        time_zone_info: Option<&PyTzInfo>,
    ) -> PyResult<PyOwned<'py, DateTime>> {
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
                (ffi::PyDateTimeAPI.DateTime_FromTimestamp)(
                    ffi::PyDateTimeAPI.DateTimeType,
                    args.as_ptr(),
                    ptr::null_mut(),
                )
            };

            PyOwned::from_raw_or_fetch_err(py, ptr)
        }
    }
}

impl PyDateAccess for PyDateTime<'_> {
    fn get_year(&self) -> i32 {
        unsafe { ffi::PyDateTime_GET_YEAR(self.as_ptr()) as i32 }
    }

    fn get_month(&self) -> u8 {
        unsafe { ffi::PyDateTime_GET_MONTH(self.as_ptr()) as u8 }
    }

    fn get_day(&self) -> u8 {
        unsafe { ffi::PyDateTime_GET_DAY(self.as_ptr()) as u8 }
    }
}

impl PyTimeAccess for PyDateTime<'_> {
    fn get_hour(&self) -> u8 {
        unsafe { ffi::PyDateTime_DATE_GET_HOUR(self.as_ptr()) as u8 }
    }

    fn get_minute(&self) -> u8 {
        unsafe { ffi::PyDateTime_DATE_GET_MINUTE(self.as_ptr()) as u8 }
    }

    fn get_second(&self) -> u8 {
        unsafe { ffi::PyDateTime_DATE_GET_SECOND(self.as_ptr()) as u8 }
    }

    fn get_microsecond(&self) -> u32 {
        unsafe { ffi::PyDateTime_DATE_GET_MICROSECOND(self.as_ptr()) as u32 }
    }

    #[cfg(not(PyPy))]
    fn get_fold(&self) -> u8 {
        unsafe { ffi::PyDateTime_DATE_GET_FOLD(self.as_ptr()) as u8 }
    }
}

/// Bindings for `datetime.time`
#[repr(transparent)]
pub struct PyTime<'py>(Time, Python<'py>);

pyo3_native_object!(PyTime<'py>, Time, 'py);

impl<'py> PyTime<'py> {
    pub fn new(
        py: Python<'py>,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyAny>,
    ) -> PyResult<PyOwned<'py, Time>> {
        unsafe {
            let ptr = (ffi::PyDateTimeAPI.Time_FromTime)(
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                tzinfo.map_or(std::ptr::null_mut(), |any| any.as_ptr()),
                ffi::PyDateTimeAPI.TimeType,
            );
            PyOwned::from_raw_or_fetch_err(py, ptr)
        }
    }

    #[cfg(not(PyPy))]
    /// Alternate constructor that takes a `fold` argument
    pub fn new_with_fold(
        py: Python<'py>,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyAny>,
        fold: bool,
    ) -> PyResult<PyOwned<'py, Time>> {
        unsafe {
            let ptr = (ffi::PyDateTimeAPI.Time_FromTimeAndFold)(
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                tzinfo.map_or(std::ptr::null_mut(), |any| any.as_ptr()),
                fold as c_int,
                ffi::PyDateTimeAPI.TimeType,
            );
            PyOwned::from_raw_or_fetch_err(py, ptr)
        }
    }
}

impl PyTimeAccess for PyTime<'_> {
    fn get_hour(&self) -> u8 {
        unsafe { ffi::PyDateTime_TIME_GET_HOUR(self.as_ptr()) as u8 }
    }

    fn get_minute(&self) -> u8 {
        unsafe { ffi::PyDateTime_TIME_GET_MINUTE(self.as_ptr()) as u8 }
    }

    fn get_second(&self) -> u8 {
        unsafe { ffi::PyDateTime_TIME_GET_SECOND(self.as_ptr()) as u8 }
    }

    fn get_microsecond(&self) -> u32 {
        unsafe { ffi::PyDateTime_TIME_GET_MICROSECOND(self.as_ptr()) as u32 }
    }

    #[cfg(not(PyPy))]
    fn get_fold(&self) -> u8 {
        unsafe { ffi::PyDateTime_TIME_GET_FOLD(self.as_ptr()) as u8 }
    }
}

/// Bindings for `datetime.tzinfo`
///
/// This is an abstract base class and should not be constructed directly.
#[repr(transparent)]
pub struct PyTzInfo<'py>(TzInfo, Python<'py>);
pyo3_native_object!(PyTzInfo<'py>, TzInfo, 'py);

/// Bindings for `datetime.timedelta`
#[repr(transparent)]
pub struct PyTimeDelta<'py>(TimeDelta, Python<'py>);
pyo3_native_object!(PyTimeDelta<'py>, TimeDelta, 'py);

impl<'py> PyTimeDelta<'py> {
    pub fn new(
        py: Python<'py>,
        days: i32,
        seconds: i32,
        microseconds: i32,
        normalize: bool,
    ) -> PyResult<PyOwned<'py, TimeDelta>> {
        unsafe {
            let ptr = (ffi::PyDateTimeAPI.Delta_FromDelta)(
                days as c_int,
                seconds as c_int,
                microseconds as c_int,
                normalize as c_int,
                ffi::PyDateTimeAPI.DeltaType,
            );
            PyOwned::from_raw_or_fetch_err(py, ptr)
        }
    }
}

impl PyDeltaAccess for PyTimeDelta<'_> {
    fn get_days(&self) -> i32 {
        unsafe { ffi::PyDateTime_DELTA_GET_DAYS(self.as_ptr()) as i32 }
    }

    fn get_seconds(&self) -> i32 {
        unsafe { ffi::PyDateTime_DELTA_GET_SECONDS(self.as_ptr()) as i32 }
    }

    fn get_microseconds(&self) -> i32 {
        unsafe { ffi::PyDateTime_DELTA_GET_MICROSECONDS(self.as_ptr()) as i32 }
    }
}
