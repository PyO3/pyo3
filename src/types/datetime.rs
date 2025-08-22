//! Safe Rust wrappers for types defined in the Python `datetime` library
//!
//! For more details about these types, see the [Python
//! documentation](https://docs.python.org/3/library/datetime.html)

use crate::err::PyResult;
#[cfg(not(Py_3_9))]
use crate::exceptions::PyImportError;
#[cfg(not(Py_LIMITED_API))]
use crate::ffi::{
    self, PyDateTime_CAPI, PyDateTime_DATE_GET_FOLD, PyDateTime_DATE_GET_HOUR,
    PyDateTime_DATE_GET_MICROSECOND, PyDateTime_DATE_GET_MINUTE, PyDateTime_DATE_GET_SECOND,
    PyDateTime_DELTA_GET_DAYS, PyDateTime_DELTA_GET_MICROSECONDS, PyDateTime_DELTA_GET_SECONDS,
    PyDateTime_FromTimestamp, PyDateTime_GET_DAY, PyDateTime_GET_MONTH, PyDateTime_GET_YEAR,
    PyDateTime_IMPORT, PyDateTime_TIME_GET_FOLD, PyDateTime_TIME_GET_HOUR,
    PyDateTime_TIME_GET_MICROSECOND, PyDateTime_TIME_GET_MINUTE, PyDateTime_TIME_GET_SECOND,
    PyDate_FromTimestamp,
};
#[cfg(all(Py_3_10, not(Py_LIMITED_API)))]
use crate::ffi::{PyDateTime_DATE_GET_TZINFO, PyDateTime_TIME_GET_TZINFO, Py_IsNone};
use crate::types::{any::PyAnyMethods, PyString, PyType};
#[cfg(not(Py_LIMITED_API))]
use crate::{ffi_ptr_ext::FfiPtrExt, py_result_ext::PyResultExt, types::PyTuple, BoundObject};
use crate::{sync::PyOnceLock, Py};
#[cfg(Py_LIMITED_API)]
use crate::{types::IntoPyDict, PyTypeCheck};
use crate::{Borrowed, Bound, IntoPyObject, PyAny, PyErr, Python};
#[cfg(not(Py_LIMITED_API))]
use std::ffi::c_int;

#[cfg(not(Py_LIMITED_API))]
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

#[cfg(not(Py_LIMITED_API))]
fn expect_datetime_api(py: Python<'_>) -> &'static PyDateTime_CAPI {
    ensure_datetime_api(py).expect("failed to import `datetime` C API")
}

#[cfg(Py_LIMITED_API)]
struct DatetimeTypes {
    date: Py<PyType>,
    datetime: Py<PyType>,
    time: Py<PyType>,
    timedelta: Py<PyType>,
    timezone: Py<PyType>,
    tzinfo: Py<PyType>,
}

#[cfg(Py_LIMITED_API)]
impl DatetimeTypes {
    fn get(py: Python<'_>) -> &Self {
        Self::try_get(py).expect("failed to load datetime module")
    }

    fn try_get(py: Python<'_>) -> PyResult<&Self> {
        static TYPES: PyOnceLock<DatetimeTypes> = PyOnceLock::new();
        TYPES.get_or_try_init(py, || {
            let datetime = py.import("datetime")?;
            Ok::<_, PyErr>(Self {
                date: datetime.getattr("date")?.cast_into()?.into(),
                datetime: datetime.getattr("datetime")?.cast_into()?.into(),
                time: datetime.getattr("time")?.cast_into()?.into(),
                timedelta: datetime.getattr("timedelta")?.cast_into()?.into(),
                timezone: datetime.getattr("timezone")?.cast_into()?.into(),
                tzinfo: datetime.getattr("tzinfo")?.cast_into()?.into(),
            })
        })
    }
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
#[cfg(not(Py_LIMITED_API))]
macro_rules! ffi_fun_with_autoinit {
    ($(#[$outer:meta] unsafe fn $name: ident($arg: ident: *mut PyObject) -> $ret: ty;)*) => {
        $(
            #[$outer]
            #[allow(non_snake_case)]
            /// # Safety
            ///
            /// Must only be called while the GIL is held
            unsafe fn $name($arg: *mut crate::ffi::PyObject) -> $ret {

                let _ = ensure_datetime_api(unsafe { Python::assume_attached() });
                unsafe { crate::ffi::$name($arg) }
            }
        )*


    };
}

#[cfg(not(Py_LIMITED_API))]
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
#[cfg(not(Py_LIMITED_API))]
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
#[cfg(not(Py_LIMITED_API))]
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
#[cfg(not(Py_LIMITED_API))]
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
}

/// Bindings around `datetime.date`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyDate>`][crate::Py] or [`Bound<'py, PyDate>`][Bound].
#[repr(transparent)]
pub struct PyDate(PyAny);

#[cfg(not(Py_LIMITED_API))]
pyobject_native_type!(
    PyDate,
    crate::ffi::PyDateTime_Date,
    |py| expect_datetime_api(py).DateType,
    #module=Some("datetime"),
    #checkfunction=PyDate_Check
);
#[cfg(not(Py_LIMITED_API))]
pyobject_subclassable_native_type!(PyDate, crate::ffi::PyDateTime_Date);

#[cfg(Py_LIMITED_API)]
pyobject_native_type_named!(PyDate);

#[cfg(Py_LIMITED_API)]
impl PyTypeCheck for PyDate {
    const NAME: &'static str = "PyDate";
    #[cfg(feature = "experimental-inspect")]
    const PYTHON_TYPE: &'static str = "datetime.date";

    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        let py = object.py();
        DatetimeTypes::try_get(py)
            .and_then(|module| object.is_instance(module.date.bind(py)))
            .unwrap_or_default()
    }
}

impl PyDate {
    /// Creates a new `datetime.date`.
    pub fn new(py: Python<'_>, year: i32, month: u8, day: u8) -> PyResult<Bound<'_, PyDate>> {
        #[cfg(not(Py_LIMITED_API))]
        {
            let api = ensure_datetime_api(py)?;
            unsafe {
                (api.Date_FromDate)(year, c_int::from(month), c_int::from(day), api.DateType)
                    .assume_owned_or_err(py)
                    .cast_into_unchecked()
            }
        }
        #[cfg(Py_LIMITED_API)]
        unsafe {
            Ok(DatetimeTypes::try_get(py)?
                .date
                .bind(py)
                .call((year, month, day), None)?
                .cast_into_unchecked())
        }
    }

    /// Construct a `datetime.date` from a POSIX timestamp
    ///
    /// This is equivalent to `datetime.date.fromtimestamp`
    pub fn from_timestamp(py: Python<'_>, timestamp: i64) -> PyResult<Bound<'_, PyDate>> {
        #[cfg(not(Py_LIMITED_API))]
        {
            let time_tuple = PyTuple::new(py, [timestamp])?;

            // safety ensure that the API is loaded
            let _api = ensure_datetime_api(py)?;

            unsafe {
                PyDate_FromTimestamp(time_tuple.as_ptr())
                    .assume_owned_or_err(py)
                    .cast_into_unchecked()
            }
        }

        #[cfg(Py_LIMITED_API)]
        unsafe {
            Ok(DatetimeTypes::try_get(py)?
                .date
                .bind(py)
                .call_method1("fromtimestamp", (timestamp,))?
                .cast_into_unchecked())
        }
    }
}

#[cfg(not(Py_LIMITED_API))]
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

#[cfg(not(Py_LIMITED_API))]
pyobject_native_type!(
    PyDateTime,
    crate::ffi::PyDateTime_DateTime,
    |py| expect_datetime_api(py).DateTimeType,
    #module=Some("datetime"),
    #checkfunction=PyDateTime_Check
);
#[cfg(not(Py_LIMITED_API))]
pyobject_subclassable_native_type!(PyDateTime, crate::ffi::PyDateTime_DateTime);

#[cfg(Py_LIMITED_API)]
pyobject_native_type_named!(PyDateTime);

#[cfg(Py_LIMITED_API)]
impl PyTypeCheck for PyDateTime {
    const NAME: &'static str = "PyDateTime";
    #[cfg(feature = "experimental-inspect")]
    const PYTHON_TYPE: &'static str = "datetime.datetime";

    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        let py = object.py();
        DatetimeTypes::try_get(py)
            .and_then(|module| object.is_instance(module.datetime.bind(py)))
            .unwrap_or_default()
    }
}

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
        #[cfg(not(Py_LIMITED_API))]
        {
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
                .cast_into_unchecked()
            }
        }

        #[cfg(Py_LIMITED_API)]
        unsafe {
            Ok(DatetimeTypes::try_get(py)?
                .datetime
                .bind(py)
                .call(
                    (year, month, day, hour, minute, second, microsecond, tzinfo),
                    None,
                )?
                .cast_into_unchecked())
        }
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
        #[cfg(not(Py_LIMITED_API))]
        {
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
                .cast_into_unchecked()
            }
        }

        #[cfg(Py_LIMITED_API)]
        unsafe {
            Ok(DatetimeTypes::try_get(py)?
                .datetime
                .bind(py)
                .call(
                    (year, month, day, hour, minute, second, microsecond, tzinfo),
                    Some(&[("fold", fold)].into_py_dict(py)?),
                )?
                .cast_into_unchecked())
        }
    }

    /// Construct a `datetime` object from a POSIX timestamp
    ///
    /// This is equivalent to `datetime.datetime.fromtimestamp`
    pub fn from_timestamp<'py>(
        py: Python<'py>,
        timestamp: f64,
        tzinfo: Option<&Bound<'py, PyTzInfo>>,
    ) -> PyResult<Bound<'py, PyDateTime>> {
        #[cfg(not(Py_LIMITED_API))]
        {
            let args = (timestamp, tzinfo).into_pyobject(py)?;

            // safety ensure API is loaded
            let _api = ensure_datetime_api(py)?;

            unsafe {
                PyDateTime_FromTimestamp(args.as_ptr())
                    .assume_owned_or_err(py)
                    .cast_into_unchecked()
            }
        }

        #[cfg(Py_LIMITED_API)]
        unsafe {
            Ok(DatetimeTypes::try_get(py)?
                .datetime
                .bind(py)
                .call_method1("fromtimestamp", (timestamp, tzinfo))?
                .cast_into_unchecked())
        }
    }
}

#[cfg(not(Py_LIMITED_API))]
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

#[cfg(not(Py_LIMITED_API))]
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
        #[cfg(all(not(Py_3_10), not(Py_LIMITED_API)))]
        unsafe {
            let ptr = self.as_ptr() as *mut ffi::PyDateTime_DateTime;
            if (*ptr).hastzinfo != 0 {
                Some(
                    (*ptr)
                        .tzinfo
                        .assume_borrowed(self.py())
                        .to_owned()
                        .cast_into_unchecked(),
                )
            } else {
                None
            }
        }

        #[cfg(all(Py_3_10, not(Py_LIMITED_API)))]
        unsafe {
            let res = PyDateTime_DATE_GET_TZINFO(self.as_ptr());
            if Py_IsNone(res) == 1 {
                None
            } else {
                Some(
                    res.assume_borrowed(self.py())
                        .to_owned()
                        .cast_into_unchecked(),
                )
            }
        }

        #[cfg(Py_LIMITED_API)]
        unsafe {
            let tzinfo = self.getattr(intern!(self.py(), "tzinfo")).ok()?;
            if tzinfo.is_none() {
                None
            } else {
                Some(tzinfo.cast_into_unchecked())
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

#[cfg(not(Py_LIMITED_API))]
pyobject_native_type!(
    PyTime,
    crate::ffi::PyDateTime_Time,
    |py| expect_datetime_api(py).TimeType,
    #module=Some("datetime"),
    #checkfunction=PyTime_Check
);
#[cfg(not(Py_LIMITED_API))]
pyobject_subclassable_native_type!(PyTime, crate::ffi::PyDateTime_Time);

#[cfg(Py_LIMITED_API)]
pyobject_native_type_named!(PyTime);

#[cfg(Py_LIMITED_API)]
impl PyTypeCheck for PyTime {
    const NAME: &'static str = "PyTime";
    #[cfg(feature = "experimental-inspect")]
    const PYTHON_TYPE: &'static str = "datetime.time";

    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        let py = object.py();
        DatetimeTypes::try_get(py)
            .and_then(|module| object.is_instance(module.time.bind(py)))
            .unwrap_or_default()
    }
}

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
        #[cfg(not(Py_LIMITED_API))]
        {
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
                .cast_into_unchecked()
            }
        }

        #[cfg(Py_LIMITED_API)]
        unsafe {
            Ok(DatetimeTypes::try_get(py)?
                .time
                .bind(py)
                .call((hour, minute, second, microsecond, tzinfo), None)?
                .cast_into_unchecked())
        }
    }

    /// Alternate constructor that takes a `fold` argument. See [`PyDateTime::new_with_fold`].
    pub fn new_with_fold<'py>(
        py: Python<'py>,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&Bound<'py, PyTzInfo>>,
        fold: bool,
    ) -> PyResult<Bound<'py, PyTime>> {
        #[cfg(not(Py_LIMITED_API))]
        {
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
                .cast_into_unchecked()
            }
        }

        #[cfg(Py_LIMITED_API)]
        #[cfg(Py_LIMITED_API)]
        unsafe {
            Ok(DatetimeTypes::try_get(py)?
                .time
                .bind(py)
                .call(
                    (hour, minute, second, microsecond, tzinfo),
                    Some(&[("fold", fold)].into_py_dict(py)?),
                )?
                .cast_into_unchecked())
        }
    }
}

#[cfg(not(Py_LIMITED_API))]
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
        #[cfg(all(not(Py_3_10), not(Py_LIMITED_API)))]
        unsafe {
            let ptr = self.as_ptr() as *mut ffi::PyDateTime_Time;
            if (*ptr).hastzinfo != 0 {
                Some(
                    (*ptr)
                        .tzinfo
                        .assume_borrowed(self.py())
                        .to_owned()
                        .cast_into_unchecked(),
                )
            } else {
                None
            }
        }

        #[cfg(all(Py_3_10, not(Py_LIMITED_API)))]
        unsafe {
            let res = PyDateTime_TIME_GET_TZINFO(self.as_ptr());
            if Py_IsNone(res) == 1 {
                None
            } else {
                Some(
                    res.assume_borrowed(self.py())
                        .to_owned()
                        .cast_into_unchecked(),
                )
            }
        }

        #[cfg(Py_LIMITED_API)]
        unsafe {
            let tzinfo = self.getattr(intern!(self.py(), "tzinfo")).ok()?;
            if tzinfo.is_none() {
                None
            } else {
                Some(tzinfo.cast_into_unchecked())
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
/// For concrete time zone implementations, see [`timezone_utc`] and
/// the [`zoneinfo` module](https://docs.python.org/3/library/zoneinfo.html).
#[repr(transparent)]
pub struct PyTzInfo(PyAny);

#[cfg(not(Py_LIMITED_API))]
pyobject_native_type!(
    PyTzInfo,
    crate::ffi::PyObject,
    |py| expect_datetime_api(py).TZInfoType,
    #module=Some("datetime"),
    #checkfunction=PyTZInfo_Check
);
#[cfg(not(Py_LIMITED_API))]
pyobject_subclassable_native_type!(PyTzInfo, crate::ffi::PyObject);

#[cfg(Py_LIMITED_API)]
pyobject_native_type_named!(PyTzInfo);

#[cfg(Py_LIMITED_API)]
impl PyTypeCheck for PyTzInfo {
    const NAME: &'static str = "PyTzInfo";
    #[cfg(feature = "experimental-inspect")]
    const PYTHON_TYPE: &'static str = "datetime.tzinfo";

    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        let py = object.py();
        DatetimeTypes::try_get(py)
            .and_then(|module| object.is_instance(module.tzinfo.bind(py)))
            .unwrap_or_default()
    }
}

impl PyTzInfo {
    /// Equivalent to `datetime.timezone.utc`
    pub fn utc(py: Python<'_>) -> PyResult<Borrowed<'static, '_, PyTzInfo>> {
        #[cfg(not(Py_LIMITED_API))]
        unsafe {
            Ok(ensure_datetime_api(py)?
                .TimeZone_UTC
                .assume_borrowed(py)
                .cast_unchecked())
        }

        #[cfg(Py_LIMITED_API)]
        {
            static UTC: PyOnceLock<Py<PyTzInfo>> = PyOnceLock::new();
            UTC.get_or_try_init(py, || {
                Ok(DatetimeTypes::get(py)
                    .timezone
                    .bind(py)
                    .getattr("utc")?
                    .cast_into()?
                    .unbind())
            })
            .map(|utc| utc.bind_borrowed(py))
        }
    }

    /// Equivalent to `zoneinfo.ZoneInfo` constructor
    pub fn timezone<'py, T>(py: Python<'py>, iana_name: T) -> PyResult<Bound<'py, PyTzInfo>>
    where
        T: IntoPyObject<'py, Target = PyString>,
    {
        static ZONE_INFO: PyOnceLock<Py<PyType>> = PyOnceLock::new();

        let zoneinfo = ZONE_INFO.import(py, "zoneinfo", "ZoneInfo");

        #[cfg(not(Py_3_9))]
        let zoneinfo = zoneinfo
            .or_else(|_| ZONE_INFO.import(py, "backports.zoneinfo", "ZoneInfo"))
            .map_err(|_| PyImportError::new_err("Could not import \"backports.zoneinfo.ZoneInfo\". ZoneInfo is required when converting timezone-aware DateTime's. Please install \"backports.zoneinfo\" on python < 3.9"));

        zoneinfo?
            .call1((iana_name,))?
            .cast_into()
            .map_err(Into::into)
    }

    /// Equivalent to `datetime.timezone` constructor
    pub fn fixed_offset<'py, T>(py: Python<'py>, offset: T) -> PyResult<Bound<'py, PyTzInfo>>
    where
        T: IntoPyObject<'py, Target = PyDelta>,
    {
        #[cfg(not(Py_LIMITED_API))]
        {
            let api = ensure_datetime_api(py)?;
            let delta = offset.into_pyobject(py).map_err(Into::into)?;
            unsafe {
                (api.TimeZone_FromTimeZone)(delta.as_ptr(), std::ptr::null_mut())
                    .assume_owned_or_err(py)
                    .cast_into_unchecked()
            }
        }

        #[cfg(Py_LIMITED_API)]
        unsafe {
            Ok(DatetimeTypes::try_get(py)?
                .timezone
                .bind(py)
                .call1((offset,))?
                .cast_into_unchecked())
        }
    }
}

/// Equivalent to `datetime.timezone.utc`
#[deprecated(since = "0.25.0", note = "use `PyTzInfo::utc` instead")]
pub fn timezone_utc(py: Python<'_>) -> Bound<'_, PyTzInfo> {
    PyTzInfo::utc(py)
        .expect("failed to import datetime.timezone.utc")
        .to_owned()
}

/// Bindings for `datetime.timedelta`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyDelta>`][crate::Py] or [`Bound<'py, PyDelta>`][Bound].
#[repr(transparent)]
pub struct PyDelta(PyAny);

#[cfg(not(Py_LIMITED_API))]
pyobject_native_type!(
    PyDelta,
    crate::ffi::PyDateTime_Delta,
    |py| expect_datetime_api(py).DeltaType,
    #module=Some("datetime"),
    #checkfunction=PyDelta_Check
);
#[cfg(not(Py_LIMITED_API))]
pyobject_subclassable_native_type!(PyDelta, crate::ffi::PyDateTime_Delta);

#[cfg(Py_LIMITED_API)]
pyobject_native_type_named!(PyDelta);

#[cfg(Py_LIMITED_API)]
impl PyTypeCheck for PyDelta {
    const NAME: &'static str = "PyDelta";
    #[cfg(feature = "experimental-inspect")]
    const PYTHON_TYPE: &'static str = "datetime.timedelta";

    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        let py = object.py();
        DatetimeTypes::try_get(py)
            .and_then(|module| object.is_instance(module.timedelta.bind(py)))
            .unwrap_or_default()
    }
}

impl PyDelta {
    /// Creates a new `timedelta`.
    pub fn new(
        py: Python<'_>,
        days: i32,
        seconds: i32,
        microseconds: i32,
        normalize: bool,
    ) -> PyResult<Bound<'_, PyDelta>> {
        #[cfg(not(Py_LIMITED_API))]
        {
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
                .cast_into_unchecked()
            }
        }

        #[cfg(Py_LIMITED_API)]
        unsafe {
            let _ = normalize;
            Ok(DatetimeTypes::try_get(py)?
                .timedelta
                .bind(py)
                .call1((days, seconds, microseconds))?
                .cast_into_unchecked())
        }
    }
}

#[cfg(not(Py_LIMITED_API))]
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
#[cfg(not(Py_LIMITED_API))]
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
        Python::attach(|py| {
            let dt = PyDateTime::from_timestamp(py, 100.0, None).unwrap();
            py_run!(
                py,
                dt,
                "import datetime; assert dt == datetime.datetime.fromtimestamp(100)"
            );

            let utc = PyTzInfo::utc(py).unwrap();
            let dt = PyDateTime::from_timestamp(py, 100.0, Some(&utc)).unwrap();
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
        Python::attach(|py| {
            let dt = PyDate::from_timestamp(py, 100).unwrap();
            py_run!(
                py,
                dt,
                "import datetime; assert dt == datetime.date.fromtimestamp(100)"
            );
        })
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    #[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
    fn test_new_with_fold() {
        Python::attach(|py| {
            let a = PyDateTime::new_with_fold(py, 2021, 1, 23, 20, 32, 40, 341516, None, false);
            let b = PyDateTime::new_with_fold(py, 2021, 1, 23, 20, 32, 40, 341516, None, true);

            assert!(!a.unwrap().get_fold());
            assert!(b.unwrap().get_fold());
        });
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", ignore)] // DateTime import fails on wasm for mysterious reasons
    fn test_get_tzinfo() {
        crate::Python::attach(|py| {
            let utc = PyTzInfo::utc(py).unwrap();

            let dt = PyDateTime::new(py, 2018, 1, 1, 0, 0, 0, 0, Some(&utc)).unwrap();

            assert!(dt.get_tzinfo().unwrap().eq(utc).unwrap());

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

        Python::attach(|py| {
            assert!(
                PyTzInfo::fixed_offset(py, PyDelta::new(py, 0, -3600, 0, true).unwrap())
                    .unwrap()
                    .call_method1("utcoffset", (PyNone::get(py),))
                    .unwrap()
                    .cast_into::<PyDelta>()
                    .unwrap()
                    .eq(PyDelta::new(py, 0, -3600, 0, true).unwrap())
                    .unwrap()
            );

            assert!(
                PyTzInfo::fixed_offset(py, PyDelta::new(py, 0, 3600, 0, true).unwrap())
                    .unwrap()
                    .call_method1("utcoffset", (PyNone::get(py),))
                    .unwrap()
                    .cast_into::<PyDelta>()
                    .unwrap()
                    .eq(PyDelta::new(py, 0, 3600, 0, true).unwrap())
                    .unwrap()
            );

            PyTzInfo::fixed_offset(py, PyDelta::new(py, 1, 0, 0, true).unwrap()).unwrap_err();
        })
    }
}
