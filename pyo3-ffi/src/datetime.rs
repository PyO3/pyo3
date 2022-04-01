//! FFI bindings to the functions and structs defined in `datetime.h`
//!
//! This is the unsafe thin  wrapper around the [CPython C API](https://docs.python.org/3/c-api/datetime.html),
//! and covers the various date and time related objects in the Python `datetime`
//! standard library module.
//!
//! A note regarding PyPy (cpyext) support:
//!
//! Support for `PyDateTime_CAPI` is limited as of PyPy 7.0.0.
//! `DateTime_FromTimestamp` and `Date_FromTimestamp` are currently not supported.

use crate::{PyObject, PyObject_TypeCheck, PyTypeObject, Py_TYPE};
use std::cell::UnsafeCell;
use std::os::raw::{c_char, c_int, c_uchar};
use std::ptr;
#[cfg(not(PyPy))]
use {
    crate::{PyCapsule_Import, Py_hash_t},
    std::ffi::CString,
};

// Type struct wrappers
const _PyDateTime_DATE_DATASIZE: usize = 4;
const _PyDateTime_TIME_DATASIZE: usize = 6;
const _PyDateTime_DATETIME_DATASIZE: usize = 10;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// Structure representing a `datetime.timedelta`.
pub struct PyDateTime_Delta {
    pub ob_base: PyObject,
    #[cfg(not(PyPy))]
    pub hashcode: Py_hash_t,
    pub days: c_int,
    pub seconds: c_int,
    pub microseconds: c_int,
}

// skipped non-limited PyDateTime_TZInfo
// skipped non-limited _PyDateTime_BaseTZInfo
// skipped non-limited _PyDateTime_BaseTime

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// Structure representing a `datetime.time`.
pub struct PyDateTime_Time {
    pub ob_base: PyObject,
    #[cfg(not(PyPy))]
    pub hashcode: Py_hash_t,
    pub hastzinfo: c_char,
    #[cfg(not(PyPy))]
    pub data: [c_uchar; _PyDateTime_TIME_DATASIZE],
    #[cfg(not(PyPy))]
    pub fold: c_uchar,
    pub tzinfo: *mut PyObject,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// Structure representing a `datetime.date`
pub struct PyDateTime_Date {
    pub ob_base: PyObject,
    #[cfg(not(PyPy))]
    pub hashcode: Py_hash_t,
    pub hastzinfo: c_char,
    pub data: [c_uchar; _PyDateTime_DATE_DATASIZE],
}

// skipped non-limited _PyDateTime_BaseDateTime

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// Structure representing a `datetime.datetime`
pub struct PyDateTime_DateTime {
    pub ob_base: PyObject,
    #[cfg(not(PyPy))]
    pub hashcode: Py_hash_t,
    pub hastzinfo: c_char,
    #[cfg(not(PyPy))]
    pub data: [c_uchar; _PyDateTime_DATETIME_DATASIZE],
    #[cfg(not(PyPy))]
    pub fold: c_uchar,
    pub tzinfo: *mut PyObject,
}

// skipped non-limited _PyDateTime_HAS_TZINFO

// Accessor functions for PyDateTime_Date and PyDateTime_DateTime
#[inline]
#[cfg(not(PyPy))]
/// Retrieve the year component of a `PyDateTime_Date` or `PyDateTime_DateTime`.
/// Returns a signed integer greater than 0.
pub unsafe fn PyDateTime_GET_YEAR(o: *mut PyObject) -> c_int {
    // This should work for Date or DateTime
    let d = *(o as *mut PyDateTime_Date);
    c_int::from(d.data[0]) << 8 | c_int::from(d.data[1])
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the month component of a `PyDateTime_Date` or `PyDateTime_DateTime`.
/// Returns a signed integer in the range `[1, 12]`.
pub unsafe fn PyDateTime_GET_MONTH(o: *mut PyObject) -> c_int {
    let d = *(o as *mut PyDateTime_Date);
    c_int::from(d.data[2])
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the day component of a `PyDateTime_Date` or `PyDateTime_DateTime`.
/// Returns a signed integer in the interval `[1, 31]`.
pub unsafe fn PyDateTime_GET_DAY(o: *mut PyObject) -> c_int {
    let d = *(o as *mut PyDateTime_Date);
    c_int::from(d.data[3])
}

// Accessor macros for times
#[cfg(not(PyPy))]
macro_rules! _PyDateTime_GET_HOUR {
    ($o: expr, $offset:expr) => {
        c_int::from((*$o).data[$offset + 0])
    };
}

#[cfg(not(PyPy))]
macro_rules! _PyDateTime_GET_MINUTE {
    ($o: expr, $offset:expr) => {
        c_int::from((*$o).data[$offset + 1])
    };
}

#[cfg(not(PyPy))]
macro_rules! _PyDateTime_GET_SECOND {
    ($o: expr, $offset:expr) => {
        c_int::from((*$o).data[$offset + 2])
    };
}

#[cfg(not(PyPy))]
macro_rules! _PyDateTime_GET_MICROSECOND {
    ($o: expr, $offset:expr) => {
        (c_int::from((*$o).data[$offset + 3]) << 16)
            | (c_int::from((*$o).data[$offset + 4]) << 8)
            | (c_int::from((*$o).data[$offset + 5]))
    };
}

#[cfg(not(PyPy))]
macro_rules! _PyDateTime_GET_FOLD {
    ($o: expr) => {
        (*$o).fold
    };
}

#[cfg(not(PyPy))]
macro_rules! _PyDateTime_GET_TZINFO {
    ($o: expr) => {
        (*$o).tzinfo
    };
}

// Accessor functions for DateTime
#[inline]
#[cfg(not(PyPy))]
/// Retrieve the hour component of a `PyDateTime_DateTime`.
/// Returns a signed integer in the interval `[0, 23]`
pub unsafe fn PyDateTime_DATE_GET_HOUR(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_HOUR!((o as *mut PyDateTime_DateTime), _PyDateTime_DATE_DATASIZE)
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the minute component of a `PyDateTime_DateTime`.
/// Returns a signed integer in the interval `[0, 59]`
pub unsafe fn PyDateTime_DATE_GET_MINUTE(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MINUTE!((o as *mut PyDateTime_DateTime), _PyDateTime_DATE_DATASIZE)
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the second component of a `PyDateTime_DateTime`.
/// Returns a signed integer in the interval `[0, 59]`
pub unsafe fn PyDateTime_DATE_GET_SECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_SECOND!((o as *mut PyDateTime_DateTime), _PyDateTime_DATE_DATASIZE)
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the microsecond component of a `PyDateTime_DateTime`.
/// Returns a signed integer in the interval `[0, 999999]`
pub unsafe fn PyDateTime_DATE_GET_MICROSECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MICROSECOND!((o as *mut PyDateTime_DateTime), _PyDateTime_DATE_DATASIZE)
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the fold component of a `PyDateTime_DateTime`.
/// Returns a signed integer in the interval `[0, 1]`
pub unsafe fn PyDateTime_DATE_GET_FOLD(o: *mut PyObject) -> c_uchar {
    _PyDateTime_GET_FOLD!(o as *mut PyDateTime_DateTime)
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the tzinfo component of a `PyDateTime_DateTime`.
/// Returns a pointer to a `PyObject` that should be either NULL or an instance
/// of a `datetime.tzinfo` subclass.
pub unsafe fn PyDateTime_DATE_GET_TZINFO(o: *mut PyObject) -> *mut PyObject {
    _PyDateTime_GET_TZINFO!(o as *mut PyDateTime_DateTime)
}

// Accessor functions for Time
#[inline]
#[cfg(not(PyPy))]
/// Retrieve the hour component of a `PyDateTime_Time`.
/// Returns a signed integer in the interval `[0, 23]`
pub unsafe fn PyDateTime_TIME_GET_HOUR(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_HOUR!((o as *mut PyDateTime_Time), 0)
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the minute component of a `PyDateTime_Time`.
/// Returns a signed integer in the interval `[0, 59]`
pub unsafe fn PyDateTime_TIME_GET_MINUTE(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MINUTE!((o as *mut PyDateTime_Time), 0)
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the second component of a `PyDateTime_DateTime`.
/// Returns a signed integer in the interval `[0, 59]`
pub unsafe fn PyDateTime_TIME_GET_SECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_SECOND!((o as *mut PyDateTime_Time), 0)
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the microsecond component of a `PyDateTime_DateTime`.
/// Returns a signed integer in the interval `[0, 999999]`
pub unsafe fn PyDateTime_TIME_GET_MICROSECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MICROSECOND!((o as *mut PyDateTime_Time), 0)
}

#[cfg(not(PyPy))]
#[inline]
/// Retrieve the fold component of a `PyDateTime_Time`.
/// Returns a signed integer in the interval `[0, 1]`
pub unsafe fn PyDateTime_TIME_GET_FOLD(o: *mut PyObject) -> c_uchar {
    _PyDateTime_GET_FOLD!(o as *mut PyDateTime_Time)
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the tzinfo component of a `PyDateTime_Time`.
/// Returns a pointer to a `PyObject` that should be either NULL or an instance
/// of a `datetime.tzinfo` subclass.
pub unsafe fn PyDateTime_TIME_GET_TZINFO(o: *mut PyObject) -> *mut PyObject {
    _PyDateTime_GET_TZINFO!(o as *mut PyDateTime_Time)
}

// Accessor functions
#[cfg(not(PyPy))]
macro_rules! _access_field {
    ($obj:expr, $type: ident, $field:ident) => {
        (*($obj as *mut $type)).$field
    };
}

// Accessor functions for PyDateTime_Delta
#[cfg(not(PyPy))]
macro_rules! _access_delta_field {
    ($obj:expr, $field:ident) => {
        _access_field!($obj, PyDateTime_Delta, $field)
    };
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the days component of a `PyDateTime_Delta`.
///
/// Returns a signed integer in the interval [-999999999, 999999999].
///
/// Note: This retrieves a component from the underlying structure, it is *not*
/// a representation of the total duration of the structure.
pub unsafe fn PyDateTime_DELTA_GET_DAYS(o: *mut PyObject) -> c_int {
    _access_delta_field!(o, days)
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the seconds component of a `PyDateTime_Delta`.
///
/// Returns a signed integer in the interval [0, 86399].
///
/// Note: This retrieves a component from the underlying structure, it is *not*
/// a representation of the total duration of the structure.
pub unsafe fn PyDateTime_DELTA_GET_SECONDS(o: *mut PyObject) -> c_int {
    _access_delta_field!(o, seconds)
}

#[inline]
#[cfg(not(PyPy))]
/// Retrieve the seconds component of a `PyDateTime_Delta`.
///
/// Returns a signed integer in the interval [0, 999999].
///
/// Note: This retrieves a component from the underlying structure, it is *not*
/// a representation of the total duration of the structure.
pub unsafe fn PyDateTime_DELTA_GET_MICROSECONDS(o: *mut PyObject) -> c_int {
    _access_delta_field!(o, microseconds)
}

#[cfg(PyPy)]
extern "C" {
    // skipped _PyDateTime_HAS_TZINFO (not in PyPy)
    #[link_name = "PyPyDateTime_GET_YEAR"]
    pub fn PyDateTime_GET_YEAR(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_GET_MONTH"]
    pub fn PyDateTime_GET_MONTH(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_GET_DAY"]
    pub fn PyDateTime_GET_DAY(o: *mut PyObject) -> c_int;

    #[link_name = "PyPyDateTime_DATE_GET_HOUR"]
    pub fn PyDateTime_DATE_GET_HOUR(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_DATE_GET_MINUTE"]
    pub fn PyDateTime_DATE_GET_MINUTE(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_DATE_GET_SECOND"]
    pub fn PyDateTime_DATE_GET_SECOND(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_DATE_GET_MICROSECOND"]
    pub fn PyDateTime_DATE_GET_MICROSECOND(o: *mut PyObject) -> c_int;
    // skipped PyDateTime_DATE_GET_FOLD (not in PyPy)
    // skipped PyDateTime_DATE_GET_TZINFO (not in PyPy)

    #[link_name = "PyPyDateTime_TIME_GET_HOUR"]
    pub fn PyDateTime_TIME_GET_HOUR(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_TIME_GET_MINUTE"]
    pub fn PyDateTime_TIME_GET_MINUTE(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_TIME_GET_SECOND"]
    pub fn PyDateTime_TIME_GET_SECOND(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_TIME_GET_MICROSECOND"]
    pub fn PyDateTime_TIME_GET_MICROSECOND(o: *mut PyObject) -> c_int;
    // skipped PyDateTime_TIME_GET_FOLD (not in PyPy)
    // skipped PyDateTime_TIME_GET_TZINFO (not in PyPy)

    #[link_name = "PyPyDateTime_DELTA_GET_DAYS"]
    pub fn PyDateTime_DELTA_GET_DAYS(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_DELTA_GET_SECONDS"]
    pub fn PyDateTime_DELTA_GET_SECONDS(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_DELTA_GET_MICROSECONDS"]
    pub fn PyDateTime_DELTA_GET_MICROSECONDS(o: *mut PyObject) -> c_int;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyDateTime_CAPI {
    pub DateType: *mut PyTypeObject,
    pub DateTimeType: *mut PyTypeObject,
    pub TimeType: *mut PyTypeObject,
    pub DeltaType: *mut PyTypeObject,
    pub TZInfoType: *mut PyTypeObject,
    pub TimeZone_UTC: *mut PyObject,
    pub Date_FromDate: unsafe extern "C" fn(
        year: c_int,
        month: c_int,
        day: c_int,
        cls: *mut PyTypeObject,
    ) -> *mut PyObject,
    pub DateTime_FromDateAndTime: unsafe extern "C" fn(
        year: c_int,
        month: c_int,
        day: c_int,
        hour: c_int,
        minute: c_int,
        second: c_int,
        microsecond: c_int,
        tzinfo: *mut PyObject,
        cls: *mut PyTypeObject,
    ) -> *mut PyObject,
    pub Time_FromTime: unsafe extern "C" fn(
        hour: c_int,
        minute: c_int,
        second: c_int,
        microsecond: c_int,
        tzinfo: *mut PyObject,
        cls: *mut PyTypeObject,
    ) -> *mut PyObject,
    pub Delta_FromDelta: unsafe extern "C" fn(
        days: c_int,
        seconds: c_int,
        microseconds: c_int,
        normalize: c_int,
        cls: *mut PyTypeObject,
    ) -> *mut PyObject,
    pub TimeZone_FromTimeZone:
        unsafe extern "C" fn(offset: *mut PyObject, name: *mut PyObject) -> *mut PyObject,

    pub DateTime_FromTimestamp: unsafe extern "C" fn(
        cls: *mut PyTypeObject,
        args: *mut PyObject,
        kwargs: *mut PyObject,
    ) -> *mut PyObject,
    pub Date_FromTimestamp:
        unsafe extern "C" fn(cls: *mut PyTypeObject, args: *mut PyObject) -> *mut PyObject,
    #[cfg(not(PyPy))]
    pub DateTime_FromDateAndTimeAndFold: unsafe extern "C" fn(
        year: c_int,
        month: c_int,
        day: c_int,
        hour: c_int,
        minute: c_int,
        second: c_int,
        microsecond: c_int,
        tzinfo: *mut PyObject,
        fold: c_int,
        cls: *mut PyTypeObject,
    ) -> *mut PyObject,
    #[cfg(not(PyPy))]
    pub Time_FromTimeAndFold: unsafe extern "C" fn(
        hour: c_int,
        minute: c_int,
        second: c_int,
        microsecond: c_int,
        tzinfo: *mut PyObject,
        fold: c_int,
        cls: *mut PyTypeObject,
    ) -> *mut PyObject,
}

// Python already shares this object between threads, so it's no more evil for us to do it too!
unsafe impl Sync for PyDateTime_CAPI {}

/// Returns a pointer to a `PyDateTime_CAPI` instance
///
/// # Note
/// This function will return a null pointer until
/// `PyDateTime_IMPORT` is called
#[inline]
pub unsafe fn PyDateTimeAPI() -> *mut PyDateTime_CAPI {
    *PyDateTimeAPI_impl.0.get()
}

#[inline]
pub unsafe fn PyDateTime_TimeZone_UTC() -> *mut PyObject {
    (*PyDateTimeAPI()).TimeZone_UTC
}

/// Populates the `PyDateTimeAPI` object
pub unsafe fn PyDateTime_IMPORT() {
    // PyPy expects the C-API to be initialized via PyDateTime_Import, so trying to use
    // `PyCapsule_Import` will behave unexpectedly in pypy.
    #[cfg(PyPy)]
    let py_datetime_c_api = PyDateTime_Import();

    #[cfg(not(PyPy))]
    let py_datetime_c_api = {
        // PyDateTime_CAPSULE_NAME is a macro in C
        let PyDateTime_CAPSULE_NAME = CString::new("datetime.datetime_CAPI").unwrap();

        PyCapsule_Import(PyDateTime_CAPSULE_NAME.as_ptr(), 1) as *mut PyDateTime_CAPI
    };

    *PyDateTimeAPI_impl.0.get() = py_datetime_c_api;
}

// skipped non-limited PyDateTime_TimeZone_UTC

/// Type Check macros
///
/// These are bindings around the C API typecheck macros, all of them return
/// `1` if True and `0` if False. In all type check macros, the argument (`op`)
/// must not be `NULL`.
#[inline]
/// Check if `op` is a `PyDateTimeAPI.DateType` or subtype.
pub unsafe fn PyDate_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, (*PyDateTimeAPI()).DateType) as c_int
}

#[inline]
/// Check if `op`'s type is exactly `PyDateTimeAPI.DateType`.
pub unsafe fn PyDate_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == (*PyDateTimeAPI()).DateType) as c_int
}

#[inline]
/// Check if `op` is a `PyDateTimeAPI.DateTimeType` or subtype.
pub unsafe fn PyDateTime_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, (*PyDateTimeAPI()).DateTimeType) as c_int
}

#[inline]
/// Check if `op`'s type is exactly `PyDateTimeAPI.DateTimeType`.
pub unsafe fn PyDateTime_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == (*PyDateTimeAPI()).DateTimeType) as c_int
}

#[inline]
/// Check if `op` is a `PyDateTimeAPI.TimeType` or subtype.
pub unsafe fn PyTime_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, (*PyDateTimeAPI()).TimeType) as c_int
}

#[inline]
/// Check if `op`'s type is exactly `PyDateTimeAPI.TimeType`.
pub unsafe fn PyTime_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == (*PyDateTimeAPI()).TimeType) as c_int
}

#[inline]
/// Check if `op` is a `PyDateTimeAPI.DetaType` or subtype.
pub unsafe fn PyDelta_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, (*PyDateTimeAPI()).DeltaType) as c_int
}

#[inline]
/// Check if `op`'s type is exactly `PyDateTimeAPI.DeltaType`.
pub unsafe fn PyDelta_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == (*PyDateTimeAPI()).DeltaType) as c_int
}

#[inline]
/// Check if `op` is a `PyDateTimeAPI.TZInfoType` or subtype.
pub unsafe fn PyTZInfo_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, (*PyDateTimeAPI()).TZInfoType) as c_int
}

#[inline]
/// Check if `op`'s type is exactly `PyDateTimeAPI.TZInfoType`.
pub unsafe fn PyTZInfo_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == (*PyDateTimeAPI()).TZInfoType) as c_int
}

// skipped non-limited PyDate_FromDate
// skipped non-limited PyDateTime_FromDateAndTime
// skipped non-limited PyDateTime_FromDateAndTimeAndFold
// skipped non-limited PyTime_FromTime
// skipped non-limited PyTime_FromTimeAndFold
// skipped non-limited PyDelta_FromDSU
// skipped non-limited PyTimeZone_FromOffset
// skipped non-limited PyTimeZone_FromOffsetAndName

#[cfg(not(PyPy))]
pub unsafe fn PyDateTime_FromTimestamp(args: *mut PyObject) -> *mut PyObject {
    let f = (*PyDateTimeAPI()).DateTime_FromTimestamp;
    f((*PyDateTimeAPI()).DateTimeType, args, std::ptr::null_mut())
}

#[cfg(not(PyPy))]
pub unsafe fn PyDate_FromTimestamp(args: *mut PyObject) -> *mut PyObject {
    let f = (*PyDateTimeAPI()).Date_FromTimestamp;
    f((*PyDateTimeAPI()).DateType, args)
}

#[cfg(PyPy)]
extern "C" {
    #[link_name = "PyPyDate_FromTimestamp"]
    pub fn PyDate_FromTimestamp(args: *mut PyObject) -> *mut PyObject;
    #[link_name = "PyPyDateTime_FromTimestamp"]
    pub fn PyDateTime_FromTimestamp(args: *mut PyObject) -> *mut PyObject;
}

#[cfg(PyPy)]
extern "C" {
    #[link_name = "_PyPyDateTime_Import"]
    pub fn PyDateTime_Import() -> *mut PyDateTime_CAPI;
}

// Rust specific implementation details

struct PyDateTimeAPISingleton(UnsafeCell<*mut PyDateTime_CAPI>);
unsafe impl Sync for PyDateTimeAPISingleton {}

static PyDateTimeAPI_impl: PyDateTimeAPISingleton =
    PyDateTimeAPISingleton(UnsafeCell::new(ptr::null_mut()));
