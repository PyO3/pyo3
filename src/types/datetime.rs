//! Safe Rust wrappers for types defined in the Python `datetime` library
//!
//! For more details about these types, see the [Python
//! documentation](https://docs.python.org/3/library/datetime.html)

use crate::err::PyResult;
use crate::ffi::{
    self, PyDateTime_CAPI, PyDateTime_FromTimestamp, PyDateTime_IMPORT, PyDate_FromTimestamp,
};
use crate::ffi::{
    PyDateTime_DATE_GET_FOLD, PyDateTime_DATE_GET_HOUR, PyDateTime_DATE_GET_MICROSECOND,
    PyDateTime_DATE_GET_MINUTE, PyDateTime_DATE_GET_SECOND,
};
#[cfg(GraalPy)]
use crate::ffi::{PyDateTime_DATE_GET_TZINFO, PyDateTime_TIME_GET_TZINFO, Py_IsNone};
use crate::ffi::{
    PyDateTime_DELTA_GET_DAYS, PyDateTime_DELTA_GET_MICROSECONDS, PyDateTime_DELTA_GET_SECONDS,
};
use crate::ffi::{PyDateTime_GET_DAY, PyDateTime_GET_MONTH, PyDateTime_GET_YEAR};
use crate::ffi::{
    PyDateTime_TIME_GET_FOLD, PyDateTime_TIME_GET_HOUR, PyDateTime_TIME_GET_MICROSECOND,
    PyDateTime_TIME_GET_MINUTE, PyDateTime_TIME_GET_SECOND,
};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::types::any::PyAnyMethods;
use crate::types::PyTuple;
use crate::{Bound, IntoPyObject, PyAny, PyErr, Python};
use std::os::raw::c_int;
#[cfg(any(feature = "chrono", feature = "jiff-02"))]
use std::ptr;

fn ensure_datetime_api(py: Python<'_>) -> PyResult<&'static PyDateTime_CAPI> {
    if let Some(api) = unsafe { pyo3_ffi::PyDateTimeAPI().as_ref() } {
        Ok(api)
    } else {
        unsafe {
            PyDateTime_IMPORT();
            pyo3_ffi::PyDateTimeAPI().as_ref()
        }
        .ok_or_else(|| PyErr::fetch(py))
    }
}

fn expect_datetime_api(py: Python<'_>) -> &'static PyDateTime_CAPI {
    ensure_datetime_api(py).expect("failed to import `datetime` C API")
}

// Type Check macros
//
// These are bindings around the C API typecheck macros, all of them return
// `1` if True and `0` if False. In all type check macros, the argument (`op`)
// must not be `NULL`. The implementations here all call ensure_datetime_api
// to ensure that the PyDateTimeAPI is initialized before use
//
//
// # Safety
//
// These functions must only be called when the GIL is held!

macro_rules! ffi_fun_with_autoinit {
    ($(#[$outer:meta] unsafe fn $name: ident($arg: ident: *mut PyObject) -> $ret: ty;)*) => {
        $(
            #[$outer]
            #[allow(non_snake_case)]
            /// # Safety
            ///
            /// Must only be called while the GIL is held
            unsafe fn $name($arg: *mut crate::ffi::PyObject) -> $ret {

                let _ = ensure_datetime_api(unsafe { Python::assume_gil_acquired() });
                unsafe { crate::ffi::$name($arg) }
            }
        )*


    };
}

ffi_fun_with_autoinit! {
    /// Check if `op` is a `PyDateTimeAPI.DateType` or subtype.
    unsafe fn PyDate_Check(op: *mut PyObject) -> c_int;

    /// Check if `op` is a `PyDateTimeAPI.DateTimeType` or subtype.
    unsafe fn PyDateTime_Check(op: *mut PyObject) -> c_int;

    /// Check if `op` is a `PyDateTimeAPI.TimeType` or subtype.
    unsafe fn PyTime_Check(op: *mut PyObject) -> c_int;

    /// Check if `op` is a `PyDateTimeAPI.DetaType` or subtype.
    unsafe fn PyDelta_Check(op: *mut PyObject) -> c_int;

    /// Check if `op` is a `PyDateTimeAPI.TZInfoType` or subtype.
    unsafe fn PyTZInfo_Check(op: *mut PyObject) -> c_int;
}

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
    /// This typically occurs at the end of daylight savings time. Only valid if the
    /// represented time is ambiguous.
    /// See [PEP 495](https://www.python.org/dev/peps/pep-0495/) for more detail.
    fn get_fold(&self) -> bool;
}

/// Trait for accessing the components of a struct containing a tzinfo.
pub trait PyTzInfoAccess<'py> {
    /// Returns the tzinfo (which may be None).
    ///
    /// Implementations should conform to the upstream documentation:
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_DATE_GET_TZINFO>
    /// <https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_TIME_GET_TZINFO>
    fn get_tzinfo(&self) -> Option<Bound<'py, PyTzInfo>>;

    /// Deprecated name for [`PyTzInfoAccess::get_tzinfo`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyTzInfoAccess::get_tzinfo`")]
    #[inline]
    fn get_tzinfo_bound(&self) -> Option<Bound<'py, PyTzInfo>> {
        self.get_tzinfo()
    }
}

/// Bindings around `datetime.date`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyDate>`][crate::Py] or [`Bound<'py, PyDate>`][Bound].
#[repr(transparent)]
pub struct PyDate(PyAny);
pyobject_native_type!(
    PyDate,
    crate::ffi::PyDateTime_Date,
    |py| expect_datetime_api(py).DateType,
    #module=Some("datetime"),
    #checkfunction=PyDate_Check
);
pyobject_subclassable_native_type!(PyDate, crate::ffi::PyDateTime_Date);

impl PyDate {
    /// Creates a new `datetime.date`.
    pub fn new(py: Python<'_>, year: i32, month: u8, day: u8) -> PyResult<Bound<'_, PyDate>> {
        let api = ensure_datetime_api(py)?;
        unsafe {
            (api.Date_FromDate)(year, c_int::from(month), c_int::from(day), api.DateType)
                .assume_owned_or_err(py)
                .downcast_into_unchecked()
        }
    }

    /// Deprecated name for [`PyDate::new`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyDate::new`")]
    #[inline]
    pub fn new_bound(py: Python<'_>, year: i32, month: u8, day: u8) -> PyResult<Bound<'_, PyDate>> {
        Self::new(py, year, month, day)
    }

    /// Construct a `datetime.date` from a POSIX timestamp
    ///
    /// This is equivalent to `datetime.date.fromtimestamp`
    pub fn from_timestamp(py: Python<'_>, timestamp: i64) -> PyResult<Bound<'_, PyDate>> {
        let time_tuple = PyTuple::new(py, [timestamp])?;

        // safety ensure that the API is loaded
        let _api = ensure_datetime_api(py)?;

        unsafe {
            PyDate_FromTimestamp(time_tuple.as_ptr())
                .assume_owned_or_err(py)
                .downcast_into_unchecked()
        }
    }

    /// Deprecated name for [`PyDate::from_timestamp`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyDate::from_timestamp`")]
    #[inline]
    pub fn from_timestamp_bound(py: Python<'_>, timestamp: i64) -> PyResult<Bound<'_, PyDate>> {
        Self::from_timestamp(py, timestamp)
    }
}

impl PyDateAccess for Bound<'_, PyDate> {
    fn get_year(&self) -> i32 {
        unsafe { PyDateTime_GET_YEAR(self.as_ptr()) }
    }

    fn get_month(&self) -> u8 {
        unsafe { PyDateTime_GET_MONTH(self.as_ptr()) as u8 }
    }

    fn get_day(&self) -> u8 {
        unsafe { PyDateTime_GET_DAY(self.as_ptr()) as u8 }
    }
}

/// Bindings for `datetime.datetime`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyDateTime>`][crate::Py] or [`Bound<'py, PyDateTime>`][Bound].
#[repr(transparent)]
pub struct PyDateTime(PyAny);
pyobject_native_type!(
    PyDateTime,
    crate::ffi::PyDateTime_DateTime,
    |py| expect_datetime_api(py).DateTimeType,
    #module=Some("datetime"),
    #checkfunction=PyDateTime_Check
);
pyobject_subclassable_native_type!(PyDateTime, crate::ffi::PyDateTime_DateTime);

impl PyDateTime {
    /// Creates a new `datetime.datetime` object.
    #[allow(clippy::too_many_arguments)]
    pub fn new<'py>(
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
        let api = ensure_datetime_api(py)?;
        unsafe {
            (api.DateTime_FromDateAndTime)(
                year,
                c_int::from(month),
                c_int::from(day),
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                opt_to_pyobj(tzinfo),
                api.DateTimeType,
            )
            .assume_owned_or_err(py)
            .downcast_into_unchecked()
        }
    }

    /// Deprecated name for [`PyDateTime::new`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyDateTime::new`")]
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn new_bound<'py>(
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
        Self::new(
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

    /// Alternate constructor that takes a `fold` parameter. A `true` value for this parameter
    /// signifies this this datetime is the later of two moments with the same representation,
    /// during a repeated interval.
    ///
    /// This typically occurs at the end of daylight savings time. Only valid if the
    /// represented time is ambiguous.
    /// See [PEP 495](https://www.python.org/dev/peps/pep-0495/) for more detail.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_fold<'py>(
        py: Python<'py>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&Bound<'py, PyTzInfo>>,
        fold: bool,
    ) -> PyResult<Bound<'py, PyDateTime>> {
        let api = ensure_datetime_api(py)?;
        unsafe {
            (api.DateTime_FromDateAndTimeAndFold)(
                year,
                c_int::from(month),
                c_int::from(day),
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                opt_to_pyobj(tzinfo),
                c_int::from(fold),
                api.DateTimeType,
            )
            .assume_owned_or_err(py)
            .downcast_into_unchecked()
        }
    }

    /// Deprecated name for [`PyDateTime::new_with_fold`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyDateTime::new_with_fold`")]
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn new_bound_with_fold<'py>(
        py: Python<'py>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&Bound<'py, PyTzInfo>>,
        fold: bool,
    ) -> PyResult<Bound<'py, PyDateTime>> {
        Self::new_with_fold(
            py,
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond,
            tzinfo,
            fold,
        )
    }

    /// Construct a `datetime` object from a POSIX timestamp
    ///
    /// This is equivalent to `datetime.datetime.fromtimestamp`
    pub fn from_timestamp<'py>(
        py: Python<'py>,
        timestamp: f64,
        tzinfo: Option<&Bound<'py, PyTzInfo>>,
    ) -> PyResult<Bound<'py, PyDateTime>> {
        let args = (timestamp, tzinfo).into_pyobject(py)?;

        // safety ensure API is loaded
        let _api = ensure_datetime_api(py)?;

        unsafe {
            PyDateTime_FromTimestamp(args.as_ptr())
                .assume_owned_or_err(py)
                .downcast_into_unchecked()
        }
    }

    /// Deprecated name for [`PyDateTime::from_timestamp`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyDateTime::from_timestamp`")]
    #[inline]
    pub fn from_timestamp_bound<'py>(
        py: Python<'py>,
        timestamp: f64,
        tzinfo: Option<&Bound<'py, PyTzInfo>>,
    ) -> PyResult<Bound<'py, PyDateTime>> {
        Self::from_timestamp(py, timestamp, tzinfo)
    }
}

impl PyDateAccess for Bound<'_, PyDateTime> {
    fn get_year(&self) -> i32 {
        unsafe { PyDateTime_GET_YEAR(self.as_ptr()) }
    }

    fn get_month(&self) -> u8 {
        unsafe { PyDateTime_GET_MONTH(self.as_ptr()) as u8 }
    }

    fn get_day(&self) -> u8 {
        unsafe { PyDateTime_GET_DAY(self.as_ptr()) as u8 }
    }
}

impl PyTimeAccess for Bound<'_, PyDateTime> {
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

    fn get_fold(&self) -> bool {
        unsafe { PyDateTime_DATE_GET_FOLD(self.as_ptr()) > 0 }
    }
}

impl<'py> PyTzInfoAccess<'py> for Bound<'py, PyDateTime> {
    fn get_tzinfo(&self) -> Option<Bound<'py, PyTzInfo>> {
        let ptr = self.as_ptr() as *mut ffi::PyDateTime_DateTime;
        #[cfg(not(GraalPy))]
        unsafe {
            if (*ptr).hastzinfo != 0 {
                Some(
                    (*ptr)
                        .tzinfo
                        .assume_borrowed(self.py())
                        .to_owned()
                        .downcast_into_unchecked(),
                )
            } else {
                None
            }
        }

        #[cfg(GraalPy)]
        unsafe {
            let res = PyDateTime_DATE_GET_TZINFO(ptr as *mut ffi::PyObject);
            if Py_IsNone(res) == 1 {
                None
            } else {
                Some(
                    res.assume_borrowed(self.py())
                        .to_owned()
                        .downcast_into_unchecked(),
                )
            }
        }
    }
}

/// Bindings for `datetime.time`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyTime>`][crate::Py] or [`Bound<'py, PyTime>`][Bound].
#[repr(transparent)]
pub struct PyTime(PyAny);
pyobject_native_type!(
    PyTime,
    crate::ffi::PyDateTime_Time,
    |py| expect_datetime_api(py).TimeType,
    #module=Some("datetime"),
    #checkfunction=PyTime_Check
);
pyobject_subclassable_native_type!(PyTime, crate::ffi::PyDateTime_Time);

impl PyTime {
    /// Creates a new `datetime.time` object.
    pub fn new<'py>(
        py: Python<'py>,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&Bound<'py, PyTzInfo>>,
    ) -> PyResult<Bound<'py, PyTime>> {
        let api = ensure_datetime_api(py)?;
        unsafe {
            (api.Time_FromTime)(
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                opt_to_pyobj(tzinfo),
                api.TimeType,
            )
            .assume_owned_or_err(py)
            .downcast_into_unchecked()
        }
    }

    /// Deprecated name for [`PyTime::new`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyTime::new`")]
    #[inline]
    pub fn new_bound<'py>(
        py: Python<'py>,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&Bound<'py, PyTzInfo>>,
    ) -> PyResult<Bound<'py, PyTime>> {
        Self::new(py, hour, minute, second, microsecond, tzinfo)
    }

    /// Alternate constructor that takes a `fold` argument. See [`PyDateTime::new_bound_with_fold`].
    pub fn new_with_fold<'py>(
        py: Python<'py>,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&Bound<'py, PyTzInfo>>,
        fold: bool,
    ) -> PyResult<Bound<'py, PyTime>> {
        let api = ensure_datetime_api(py)?;
        unsafe {
            (api.Time_FromTimeAndFold)(
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                opt_to_pyobj(tzinfo),
                fold as c_int,
                api.TimeType,
            )
            .assume_owned_or_err(py)
            .downcast_into_unchecked()
        }
    }

    /// Deprecated name for [`PyTime::new_with_fold`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyTime::new_with_fold`")]
    #[inline]
    pub fn new_bound_with_fold<'py>(
        py: Python<'py>,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&Bound<'py, PyTzInfo>>,
        fold: bool,
    ) -> PyResult<Bound<'py, PyTime>> {
        Self::new_with_fold(py, hour, minute, second, microsecond, tzinfo, fold)
    }
}

impl PyTimeAccess for Bound<'_, PyTime> {
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

    fn get_fold(&self) -> bool {
        unsafe { PyDateTime_TIME_GET_FOLD(self.as_ptr()) != 0 }
    }
}

impl<'py> PyTzInfoAccess<'py> for Bound<'py, PyTime> {
    fn get_tzinfo(&self) -> Option<Bound<'py, PyTzInfo>> {
        let ptr = self.as_ptr() as *mut ffi::PyDateTime_Time;
        #[cfg(not(GraalPy))]
        unsafe {
            if (*ptr).hastzinfo != 0 {
                Some(
                    (*ptr)
                        .tzinfo
                        .assume_borrowed(self.py())
                        .to_owned()
                        .downcast_into_unchecked(),
                )
            } else {
                None
            }
        }

        #[cfg(GraalPy)]
        unsafe {
            let res = PyDateTime_TIME_GET_TZINFO(ptr as *mut ffi::PyObject);
            if Py_IsNone(res) == 1 {
                None
            } else {
                Some(
                    res.assume_borrowed(self.py())
                        .to_owned()
                        .downcast_into_unchecked(),
                )
            }
        }
    }
}

/// Bindings for `datetime.tzinfo`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyTzInfo>`][crate::Py] or [`Bound<'py, PyTzInfo>`][Bound].
///
/// This is an abstract base class and cannot be constructed directly.
/// For concrete time zone implementations, see [`timezone_utc_bound`] and
/// the [`zoneinfo` module](https://docs.python.org/3/library/zoneinfo.html).
#[repr(transparent)]
pub struct PyTzInfo(PyAny);
pyobject_native_type!(
    PyTzInfo,
    crate::ffi::PyObject,
    |py| expect_datetime_api(py).TZInfoType,
    #module=Some("datetime"),
    #checkfunction=PyTZInfo_Check
);
pyobject_subclassable_native_type!(PyTzInfo, crate::ffi::PyObject);

/// Equivalent to `datetime.timezone.utc`
pub fn timezone_utc(py: Python<'_>) -> Bound<'_, PyTzInfo> {
    // TODO: this _could_ have a borrowed form `timezone_utc_borrowed`, but that seems
    // like an edge case optimization and we'd prefer in PyO3 0.21 to use `Bound` as
    // much as possible
    unsafe {
        expect_datetime_api(py)
            .TimeZone_UTC
            .assume_borrowed(py)
            .to_owned()
            .downcast_into_unchecked()
    }
}

/// Deprecated name for [`timezone_utc`].
#[deprecated(since = "0.23.0", note = "renamed to `timezone_utc`")]
#[inline]
pub fn timezone_utc_bound(py: Python<'_>) -> Bound<'_, PyTzInfo> {
    timezone_utc(py)
}

/// Equivalent to `datetime.timezone` constructor
///
/// Only used internally
#[cfg(any(feature = "chrono", feature = "jiff-02"))]
pub(crate) fn timezone_from_offset<'py>(
    offset: &Bound<'py, PyDelta>,
) -> PyResult<Bound<'py, PyTzInfo>> {
    let py = offset.py();
    let api = ensure_datetime_api(py)?;
    unsafe {
        (api.TimeZone_FromTimeZone)(offset.as_ptr(), ptr::null_mut())
            .assume_owned_or_err(py)
            .downcast_into_unchecked()
    }
}

/// Bindings for `datetime.timedelta`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyDelta>`][crate::Py] or [`Bound<'py, PyDelta>`][Bound].
#[repr(transparent)]
pub struct PyDelta(PyAny);
pyobject_native_type!(
    PyDelta,
    crate::ffi::PyDateTime_Delta,
    |py| expect_datetime_api(py).DeltaType,
    #module=Some("datetime"),
    #checkfunction=PyDelta_Check
);
pyobject_subclassable_native_type!(PyDelta, crate::ffi::PyDateTime_Delta);

impl PyDelta {
    /// Creates a new `timedelta`.
    pub fn new(
        py: Python<'_>,
        days: i32,
        seconds: i32,
        microseconds: i32,
        normalize: bool,
    ) -> PyResult<Bound<'_, PyDelta>> {
        let api = ensure_datetime_api(py)?;
        unsafe {
            (api.Delta_FromDelta)(
                days as c_int,
                seconds as c_int,
                microseconds as c_int,
                normalize as c_int,
                api.DeltaType,
            )
            .assume_owned_or_err(py)
            .downcast_into_unchecked()
        }
    }

    /// Deprecated name for [`PyDelta::new`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyDelta::new`")]
    #[inline]
    pub fn new_bound(
        py: Python<'_>,
        days: i32,
        seconds: i32,
        microseconds: i32,
        normalize: bool,
    ) -> PyResult<Bound<'_, PyDelta>> {
        Self::new(py, days, seconds, microseconds, normalize)
    }
}

impl PyDeltaAccess for Bound<'_, PyDelta> {
    fn get_days(&self) -> i32 {
        unsafe { PyDateTime_DELTA_GET_DAYS(self.as_ptr()) }
    }

    fn get_seconds(&self) -> i32 {
        unsafe { PyDateTime_DELTA_GET_SECONDS(self.as_ptr()) }
    }

    fn get_microseconds(&self) -> i32 {
        unsafe { PyDateTime_DELTA_GET_MICROSECONDS(self.as_ptr()) }
    }
}

// Utility function which returns a borrowed reference to either
// the underlying tzinfo or None.
fn opt_to_pyobj(opt: Option<&Bound<'_, PyTzInfo>>) -> *mut ffi::PyObject {
    match opt {
        Some(tzi) => tzi.as_ptr(),
        None => unsafe { ffi::Py_None() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "macros")]
    use crate::py_run;

    #[test]
    #[cfg(feature = "macros")]
    #[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
    fn test_datetime_fromtimestamp() {
        Python::with_gil(|py| {
            let dt = PyDateTime::from_timestamp(py, 100.0, None).unwrap();
            py_run!(
                py,
                dt,
                "import datetime; assert dt == datetime.datetime.fromtimestamp(100)"
            );

            let dt = PyDateTime::from_timestamp(py, 100.0, Some(&timezone_utc(py))).unwrap();
            py_run!(
                py,
                dt,
                "import datetime; assert dt == datetime.datetime.fromtimestamp(100, datetime.timezone.utc)"
            );
        })
    }

    #[test]
    #[cfg(feature = "macros")]
    #[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
    fn test_date_fromtimestamp() {
        Python::with_gil(|py| {
            let dt = PyDate::from_timestamp(py, 100).unwrap();
            py_run!(
                py,
                dt,
                "import datetime; assert dt == datetime.date.fromtimestamp(100)"
            );
        })
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
    fn test_new_with_fold() {
        Python::with_gil(|py| {
            let a = PyDateTime::new_with_fold(py, 2021, 1, 23, 20, 32, 40, 341516, None, false);
            let b = PyDateTime::new_with_fold(py, 2021, 1, 23, 20, 32, 40, 341516, None, true);

            assert!(!a.unwrap().get_fold());
            assert!(b.unwrap().get_fold());
        });
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
    fn test_get_tzinfo() {
        crate::Python::with_gil(|py| {
            let utc = timezone_utc(py);

            let dt = PyDateTime::new(py, 2018, 1, 1, 0, 0, 0, 0, Some(&utc)).unwrap();

            assert!(dt.get_tzinfo().unwrap().eq(&utc).unwrap());

            let dt = PyDateTime::new(py, 2018, 1, 1, 0, 0, 0, 0, None).unwrap();

            assert!(dt.get_tzinfo().is_none());

            let t = PyTime::new(py, 0, 0, 0, 0, Some(&utc)).unwrap();

            assert!(t.get_tzinfo().unwrap().eq(utc).unwrap());

            let t = PyTime::new(py, 0, 0, 0, 0, None).unwrap();

            assert!(t.get_tzinfo().is_none());
        });
    }

    #[test]
    #[cfg(all(feature = "macros", feature = "chrono"))]
    #[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
    fn test_timezone_from_offset() {
        use crate::types::PyNone;

        Python::with_gil(|py| {
            assert!(
                timezone_from_offset(&PyDelta::new(py, 0, -3600, 0, true).unwrap())
                    .unwrap()
                    .call_method1("utcoffset", (PyNone::get(py),))
                    .unwrap()
                    .downcast_into::<PyDelta>()
                    .unwrap()
                    .eq(PyDelta::new(py, 0, -3600, 0, true).unwrap())
                    .unwrap()
            );

            assert!(
                timezone_from_offset(&PyDelta::new(py, 0, 3600, 0, true).unwrap())
                    .unwrap()
                    .call_method1("utcoffset", (PyNone::get(py),))
                    .unwrap()
                    .downcast_into::<PyDelta>()
                    .unwrap()
                    .eq(PyDelta::new(py, 0, 3600, 0, true).unwrap())
                    .unwrap()
            );

            timezone_from_offset(&PyDelta::new(py, 1, 0, 0, true).unwrap()).unwrap_err();
        })
    }
}
