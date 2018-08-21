use ffi::{PyObject, PyTypeObject};
use ffi::{Py_TYPE, PyObject_TypeCheck};
use ffi::PyCapsule_Import;
use ffi::Py_hash_t;
use std::ffi::CString;
use std::ops::Deref;
use std::os::raw::{c_char, c_int, c_uchar};
use std::ptr;
use std::sync::Once;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyDateTime_DateType: PyTypeObject;
    pub static mut PyDateTime_TimeType: PyTypeObject;
    pub static mut PyDateTime_DateTimeType: PyTypeObject;

    pub static mut PyDateTime_DeltaType: PyTypeObject;
    pub static mut PyDateTime_TZInfoType: PyTypeObject;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyDateTime_CAPI {
    pub DateType: *mut PyTypeObject,
    pub DateTimeType: *mut PyTypeObject,
    pub TimeType: *mut PyTypeObject,
    pub DeltaType: *mut PyTypeObject,
    pub TZInfoType: *mut PyTypeObject,
    #[cfg(Py_3_7)]
    pub TimeZone_UTC: *mut PyObject,

    pub Date_FromDate:
        unsafe extern "C" fn(year: c_int, month: c_int, day: c_int, cls: *mut PyTypeObject)
            -> *mut PyObject,
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
    #[cfg(Py_3_7)]
    pub TimeZone_FromTimeZone:
        unsafe extern "C" fn(offset: *mut PyObject, name: *mut PyObject) -> *mut PyObject,
    pub DateTime_FromTimestamp:
        unsafe extern "C" fn(cls: *mut PyTypeObject, args: *mut PyObject, kwargs: *mut PyObject)
            -> *mut PyObject,
    pub Date_FromTimestamp:
        unsafe extern "C" fn(cls: *mut PyTypeObject, args: *mut PyObject) -> *mut PyObject,
    #[cfg(Py_3_6)]
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
    #[cfg(Py_3_6)]
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

// Type struct wrappers

const _PyDateTime_DATE_DATASIZE: usize = 4;
const _PyDateTime_TIME_DATASIZE: usize = 6;
const _PyDateTime_DATETIME_DATASIZE: usize = 10;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyDateTime_Date {
    pub ob_base: PyObject,
    pub hashcode: Py_hash_t,
    pub hastzinfo: c_char,
    pub data: [c_uchar; _PyDateTime_DATE_DATASIZE],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyDateTime_Time {
    pub ob_base: PyObject,
    pub hashcode: Py_hash_t,
    pub hastzinfo: c_char,
    pub data: [c_uchar; _PyDateTime_TIME_DATASIZE],
    #[cfg(Py_3_6)]
    pub fold: c_uchar,
    pub tzinfo: *mut PyObject,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyDateTime_DateTime {
    pub ob_base: PyObject,
    pub hashcode: Py_hash_t,
    pub hastzinfo: c_char,
    pub data: [c_uchar; _PyDateTime_DATETIME_DATASIZE],
    #[cfg(Py_3_6)]
    pub fold: c_uchar,
    pub tzinfo: *mut PyObject,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyDateTime_Delta {
    pub ob_base: PyObject,
    pub hashcode: Py_hash_t,
    pub days: c_int,
    pub seconds: c_int,
    pub microseconds: c_int,
}

// C API Capsule
// Note: This is "roll-your-own" lazy-static implementation is necessary because
// of the interaction between the call_once locks and the GIL. It turns out that
// calling PyCapsule_Import releases and re-acquires the GIL during the import,
// so if you have two threads attempting to use the PyDateTimeAPI singleton
// under the GIL, it causes a deadlock; what happens is:
//
// Thread 1 acquires GIL
// Thread 1 acquires the call_once lock
// Thread 1 calls PyCapsule_Import, thus releasing the GIL
// Thread 2 acquires the GIL
// Thread 2 blocks waiting for the call_once lock
// Thread 1 blocks waiting for the GIL
//
// However, Python's import mechanism acquires a module-specific lock around
// each import call, so all call importing datetime will return the same
// object, making the call_once lock superfluous. As such, we can weaken
// the guarantees of the cache, such that PyDateTime_IMPORT can be called
// until __PY_DATETIME_API_UNSAFE_CACHE is populated, which will happen exactly
// one time. So long as PyDateTime_IMPORT has no side effects (it should not),
// this will be at most a slight waste of resources.
static __PY_DATETIME_API_ONCE: Once = Once::new();
static mut __PY_DATETIME_API_UNSAFE_CACHE: *const PyDateTime_CAPI = ptr::null();

pub struct PyDateTimeAPI {
    __private_field: (),
}
pub static PyDateTimeAPI: PyDateTimeAPI = PyDateTimeAPI {
    __private_field: (),
};

impl Deref for PyDateTimeAPI {
    type Target = PyDateTime_CAPI;

    fn deref(&self) -> &'static PyDateTime_CAPI {
        unsafe {
            let cache_val = if !__PY_DATETIME_API_UNSAFE_CACHE.is_null() {
                return &(*__PY_DATETIME_API_UNSAFE_CACHE);
            } else {
                PyDateTime_IMPORT()
            };

            __PY_DATETIME_API_ONCE.call_once(move || {
                __PY_DATETIME_API_UNSAFE_CACHE = cache_val;
            });

            &(*__PY_DATETIME_API_UNSAFE_CACHE)
        }
    }
}

#[inline(always)]
pub unsafe fn PyDateTime_IMPORT() -> *const PyDateTime_CAPI {
    // PyDateTime_CAPSULE_NAME is a macro in C
    let PyDateTime_CAPSULE_NAME = CString::new("datetime.datetime_CAPI").unwrap();

    let capsule = PyCapsule_Import(PyDateTime_CAPSULE_NAME.as_ptr(), 1);
    capsule as *const PyDateTime_CAPI
}

//
// Type Check macros
//
#[inline(always)]
pub unsafe fn PyDate_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, PyDateTimeAPI.DateType) as c_int
}

#[inline(always)]
pub unsafe fn PyDate_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == PyDateTimeAPI.DateType) as c_int
}

#[inline(always)]
pub unsafe fn PyDateTime_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, PyDateTimeAPI.DateTimeType) as c_int
}

#[inline(always)]
pub unsafe fn PyDateTime_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == PyDateTimeAPI.DateTimeType) as c_int
}

#[inline(always)]
pub unsafe fn PyTime_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, PyDateTimeAPI.TimeType) as c_int
}

#[inline(always)]
pub unsafe fn PyTime_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == PyDateTimeAPI.TimeType) as c_int
}

#[inline(always)]
pub unsafe fn PyDelta_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, PyDateTimeAPI.DeltaType) as c_int
}

#[inline(always)]
pub unsafe fn PyDelta_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == PyDateTimeAPI.DeltaType) as c_int
}

#[inline(always)]
pub unsafe fn PyTZInfo_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, PyDateTimeAPI.TZInfoType) as c_int
}

#[inline(always)]
pub unsafe fn PyTZInfo_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == PyDateTimeAPI.TZInfoType) as c_int
}

//
// Accessor functions
//
macro_rules! _access_field {
    ($obj:expr, $type: ident, $field:tt) => {
        (*($obj as *mut $type)).$field
    };
}

// Accessor functions for PyDateTime_Date and PyDateTime_DateTime
#[inline]
pub unsafe fn PyDateTime_GET_YEAR(o: *mut PyObject) -> c_int {
    // This should work for Date or DateTime
    let d = *(o as *mut PyDateTime_Date);
    (c_int::from(d.data[0]) << 8 | c_int::from(d.data[1]))
}

#[inline]
pub unsafe fn PyDateTime_GET_MONTH(o: *mut PyObject) -> c_int {
    let d = *(o as *mut PyDateTime_Date);
    c_int::from(d.data[2])
}

#[inline]
pub unsafe fn PyDateTime_GET_DAY(o: *mut PyObject) -> c_int {
    let d = *(o as *mut PyDateTime_Date);
    c_int::from(d.data[3])
}

// Accessor macros for times
macro_rules! _PyDateTime_GET_HOUR {
    ($o: expr, $offset:expr) => {
        c_int::from((*$o).data[$offset + 0])
    };
}

macro_rules! _PyDateTime_GET_MINUTE {
    ($o: expr, $offset:expr) => {
        c_int::from((*$o).data[$offset + 1])
    };
}

macro_rules! _PyDateTime_GET_SECOND {
    ($o: expr, $offset:expr) => {
        c_int::from((*$o).data[$offset + 2])
    };
}

macro_rules! _PyDateTime_GET_MICROSECOND {
    ($o: expr, $offset:expr) => {
        (c_int::from((*$o).data[$offset + 3]) << 16)
            | (c_int::from((*$o).data[$offset + 4]) << 8)
            | (c_int::from((*$o).data[$offset + 5]))
    };
}

#[cfg(Py_3_6)]
macro_rules! _PyDateTime_GET_FOLD {
    ($o: expr) => {
        (*$o).fold
    };
}

macro_rules! _PyDateTime_GET_TZINFO {
    ($o: expr) => {
        (*$o).tzinfo
    };
}

// Accessor functions for DateTime
#[inline(always)]
pub unsafe fn PyDateTime_DATE_GET_HOUR(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_HOUR!((o as *mut PyDateTime_DateTime), _PyDateTime_DATE_DATASIZE)
}

#[inline(always)]
pub unsafe fn PyDateTime_DATE_GET_MINUTE(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MINUTE!((o as *mut PyDateTime_DateTime), _PyDateTime_DATE_DATASIZE)
}

#[inline(always)]
pub unsafe fn PyDateTime_DATE_GET_SECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_SECOND!((o as *mut PyDateTime_DateTime), _PyDateTime_DATE_DATASIZE)
}

#[inline(always)]
pub unsafe fn PyDateTime_DATE_GET_MICROSECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MICROSECOND!((o as *mut PyDateTime_DateTime), _PyDateTime_DATE_DATASIZE)
}

#[cfg(Py_3_6)]
#[inline(always)]
pub unsafe fn PyDateTime_DATE_GET_FOLD(o: *mut PyObject) -> c_uchar {
    _PyDateTime_GET_FOLD!(o as *mut PyDateTime_DateTime)
}

#[inline(always)]
pub unsafe fn PyDateTime_DATE_GET_TZINFO(o: *mut PyObject) -> *mut PyObject {
    _PyDateTime_GET_TZINFO!(o as *mut PyDateTime_DateTime)
}

// Accessor functions for Time
#[inline(always)]
pub unsafe fn PyDateTime_TIME_GET_HOUR(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_HOUR!((o as *mut PyDateTime_Time), 0)
}

#[inline(always)]
pub unsafe fn PyDateTime_TIME_GET_MINUTE(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MINUTE!((o as *mut PyDateTime_Time), 0)
}

#[inline(always)]
pub unsafe fn PyDateTime_TIME_GET_SECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_SECOND!((o as *mut PyDateTime_Time), 0)
}

#[inline(always)]
pub unsafe fn PyDateTime_TIME_GET_MICROSECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MICROSECOND!((o as *mut PyDateTime_Time), 0)
}

#[cfg(Py_3_6)]
#[inline(always)]
pub unsafe fn PyDateTime_TIME_GET_FOLD(o: *mut PyObject) -> c_uchar {
    _PyDateTime_GET_FOLD!(o as *mut PyDateTime_Time)
}

#[inline(always)]
pub unsafe fn PyDateTime_TIME_GET_TZINFO(o: *mut PyObject) -> *mut PyObject {
    _PyDateTime_GET_TZINFO!(o as *mut PyDateTime_Time)
}

// Accessor functions for PyDateTime_Delta
macro_rules! _access_delta_field {
    ($obj:expr, $field:tt) => {
        _access_field!($obj, PyDateTime_Delta, $field)
    };
}

#[inline(always)]
pub unsafe fn PyDateTime_DELTA_GET_DAYS(o: *mut PyObject) -> c_int {
    _access_delta_field!(o, days)
}

#[inline(always)]
pub unsafe fn PyDateTime_DELTA_GET_SECONDS(o: *mut PyObject) -> c_int {
    _access_delta_field!(o, seconds)
}

#[inline(always)]
pub unsafe fn PyDateTime_DELTA_GET_MICROSECONDS(o: *mut PyObject) -> c_int {
    _access_delta_field!(o, microseconds)
}
