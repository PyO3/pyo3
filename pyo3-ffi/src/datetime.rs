//! FFI bindings to the functions and structs defined in `datetime.h`
//!
//! This is the unsafe thin  wrapper around the [CPython C API](https://docs.python.org/3/c-api/datetime.html),
//! and covers the various date and time related objects in the Python `datetime`
//! standard library module.

use crate::{PyObject, PyObject_TypeCheck, PyTypeObject, Py_None, Py_TYPE};
use std::ffi::c_char;
use std::ffi::c_int;
use std::ptr;
use std::sync::Once;
use std::{cell::UnsafeCell, ffi::CStr};
#[cfg(not(PyPy))]
use {crate::Py_hash_t, std::ffi::c_uchar};
// Type struct wrappers
const _PyDateTime_DATE_DATASIZE: usize = 4;
const _PyDateTime_TIME_DATASIZE: usize = 6;
const _PyDateTime_DATETIME_DATASIZE: usize = 10;

#[repr(C)]
#[derive(Debug)]
/// Structure representing a `datetime.timedelta`.
pub struct PyDateTime_Delta {
    pub ob_base: PyObject,
    #[cfg(not(PyPy))]
    pub hashcode: Py_hash_t,
    pub days: c_int,
    pub seconds: c_int,
    pub microseconds: c_int,
}

#[repr(C)]
#[derive(Debug)]
pub struct PyDateTime_TZInfo {
    pub ob_base: PyObject,
}

// skipped non-limited _PyDateTime_BaseTZInfo

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Debug)]
/// Structure representing a `datetime.time` without a `tzinfo` member.
pub struct _PyDateTime_BaseTime {
    pub ob_base: PyObject,
    pub hashcode: Py_hash_t,
    pub hastzinfo: c_char,
    pub data: [c_uchar; _PyDateTime_TIME_DATASIZE],
}

#[repr(C)]
#[derive(Debug)]
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
    /// # Safety
    ///
    /// Care should be taken when reading this field. If the time does not have a
    /// tzinfo then CPython may allocate as a `_PyDateTime_BaseTime` without this field.
    pub tzinfo: *mut PyObject,
}

#[repr(C)]
#[derive(Debug)]
/// Structure representing a `datetime.date`
pub struct PyDateTime_Date {
    pub ob_base: PyObject,
    #[cfg(not(PyPy))]
    pub hashcode: Py_hash_t,
    #[cfg(not(PyPy))]
    pub hastzinfo: c_char,
    #[cfg(not(PyPy))]
    pub data: [c_uchar; _PyDateTime_DATE_DATASIZE],
}

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Debug)]
/// Structure representing a `datetime.datetime` without a `tzinfo` member.
pub struct _PyDateTime_BaseDateTime {
    pub ob_base: PyObject,
    pub hashcode: Py_hash_t,
    pub hastzinfo: c_char,
    pub data: [c_uchar; _PyDateTime_DATETIME_DATASIZE],
}

#[repr(C)]
#[derive(Debug)]
/// Structure representing a `datetime.datetime`.
pub struct PyDateTime_DateTime {
    pub ob_base: PyObject,
    #[cfg(not(PyPy))]
    pub hashcode: Py_hash_t,
    pub hastzinfo: c_char,
    #[cfg(not(PyPy))]
    pub data: [c_uchar; _PyDateTime_DATETIME_DATASIZE],
    #[cfg(not(PyPy))]
    pub fold: c_uchar,
    /// # Safety
    ///
    /// Care should be taken when reading this field. If the time does not have a
    /// tzinfo then CPython may allocate as a `_PyDateTime_BaseDateTime` without this field.
    pub tzinfo: *mut PyObject,
}

// skipped non-limited _PyDateTime_HAS_TZINFO

pub use crate::backend::current::datetime::{
    PyDateTime_DATE_GET_FOLD, PyDateTime_DATE_GET_HOUR, PyDateTime_DATE_GET_MICROSECOND,
    PyDateTime_DATE_GET_MINUTE, PyDateTime_DATE_GET_SECOND, PyDateTime_DATE_GET_TZINFO,
    PyDateTime_DELTA_GET_DAYS, PyDateTime_DELTA_GET_MICROSECONDS, PyDateTime_DELTA_GET_SECONDS,
    PyDateTime_GET_DAY, PyDateTime_GET_MONTH, PyDateTime_GET_YEAR, PyDateTime_TIME_GET_FOLD,
    PyDateTime_TIME_GET_HOUR, PyDateTime_TIME_GET_MICROSECOND, PyDateTime_TIME_GET_MINUTE,
    PyDateTime_TIME_GET_SECOND, PyDateTime_TIME_GET_TZINFO,
};

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

pub const PyDateTime_CAPSULE_NAME: &CStr = c"datetime.datetime_CAPI";

/// Returns a pointer to a `PyDateTime_CAPI` instance
///
/// # Note
/// This function will return a null pointer until
/// `PyDateTime_IMPORT` is called
#[inline]
pub unsafe fn PyDateTimeAPI() -> *mut PyDateTime_CAPI {
    *PyDateTimeAPI_impl.ptr.get()
}

/// Populates the `PyDateTimeAPI` object
pub unsafe fn PyDateTime_IMPORT() {
    if !PyDateTimeAPI_impl.once.is_completed() {
        let py_datetime_c_api = crate::backend::current::datetime::import_datetime_api();

        if py_datetime_c_api.is_null() {
            return;
        }

        // Protect against race conditions when the datetime API is concurrently
        // initialized in multiple threads. UnsafeCell.get() cannot panic so this
        // won't panic either.
        PyDateTimeAPI_impl.once.call_once(|| {
            *PyDateTimeAPI_impl.ptr.get() = py_datetime_c_api;
        });
    }
}

#[inline]
pub unsafe fn PyDateTime_TimeZone_UTC() -> *mut PyObject {
    (*PyDateTimeAPI()).TimeZone_UTC
}

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

pub unsafe fn PyDate_FromDate(year: c_int, month: c_int, day: c_int) -> *mut PyObject {
    ((*PyDateTimeAPI()).Date_FromDate)(year, month, day, (*PyDateTimeAPI()).DateType)
}

#[allow(clippy::too_many_arguments)]
/// See <https://github.com/python/cpython/blob/3.10/Include/datetime.h#L226-L228>
pub unsafe fn PyDateTime_FromDateAndTime(
    year: c_int,
    month: c_int,
    day: c_int,
    hour: c_int,
    minute: c_int,
    second: c_int,
    microsecond: c_int,
) -> *mut PyObject {
    ((*PyDateTimeAPI()).DateTime_FromDateAndTime)(
        year,
        month,
        day,
        hour,
        minute,
        second,
        microsecond,
        Py_None(),
        (*PyDateTimeAPI()).DateTimeType,
    )
}

#[allow(clippy::too_many_arguments)]
/// See <https://github.com/python/cpython/blob/3.10/Include/datetime.h#L230-L232>
pub unsafe fn PyDateTime_FromDateAndTimeAndFold(
    year: c_int,
    month: c_int,
    day: c_int,
    hour: c_int,
    minute: c_int,
    second: c_int,
    microsecond: c_int,
    fold: c_int,
) -> *mut PyObject {
    ((*PyDateTimeAPI()).DateTime_FromDateAndTimeAndFold)(
        year,
        month,
        day,
        hour,
        minute,
        second,
        microsecond,
        Py_None(),
        fold,
        (*PyDateTimeAPI()).DateTimeType,
    )
}

pub unsafe fn PyTime_FromTime(
    hour: c_int,
    minute: c_int,
    second: c_int,
    microsecond: c_int,
) -> *mut PyObject {
    ((*PyDateTimeAPI()).Time_FromTime)(
        hour,
        minute,
        second,
        microsecond,
        Py_None(),
        (*PyDateTimeAPI()).TimeType,
    )
}

pub unsafe fn PyTime_FromTimeAndFold(
    hour: c_int,
    minute: c_int,
    second: c_int,
    microsecond: c_int,
    fold: c_int,
) -> *mut PyObject {
    ((*PyDateTimeAPI()).Time_FromTimeAndFold)(
        hour,
        minute,
        second,
        microsecond,
        Py_None(),
        fold,
        (*PyDateTimeAPI()).TimeType,
    )
}

pub unsafe fn PyDelta_FromDSU(days: c_int, seconds: c_int, microseconds: c_int) -> *mut PyObject {
    ((*PyDateTimeAPI()).Delta_FromDelta)(
        days,
        seconds,
        microseconds,
        1,
        (*PyDateTimeAPI()).DeltaType,
    )
}

pub unsafe fn PyTimeZone_FromOffset(offset: *mut PyObject) -> *mut PyObject {
    ((*PyDateTimeAPI()).TimeZone_FromTimeZone)(offset, std::ptr::null_mut())
}

pub unsafe fn PyTimeZone_FromOffsetAndName(
    offset: *mut PyObject,
    name: *mut PyObject,
) -> *mut PyObject {
    ((*PyDateTimeAPI()).TimeZone_FromTimeZone)(offset, name)
}

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
extern_libpython! {
    #[link_name = "PyPyDate_FromTimestamp"]
    pub fn PyDate_FromTimestamp(args: *mut PyObject) -> *mut PyObject;
    #[link_name = "PyPyDateTime_FromTimestamp"]
    pub fn PyDateTime_FromTimestamp(args: *mut PyObject) -> *mut PyObject;
}

#[cfg(PyPy)]
extern_libpython! {
    #[link_name = "_PyPyDateTime_Import"]
    pub fn PyDateTime_Import() -> *mut PyDateTime_CAPI;
}

// Rust specific implementation details

struct PyDateTimeAPISingleton {
    once: Once,
    ptr: UnsafeCell<*mut PyDateTime_CAPI>,
}
unsafe impl Sync for PyDateTimeAPISingleton {}

static PyDateTimeAPI_impl: PyDateTimeAPISingleton = PyDateTimeAPISingleton {
    once: Once::new(),
    ptr: UnsafeCell::new(ptr::null_mut()),
};
