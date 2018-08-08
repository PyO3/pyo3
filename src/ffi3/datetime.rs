use std::os::raw::c_int;
use std::ffi::CString;
use std::option::Option;
use ffi3::pyport::Py_hash_t;
use ffi3::object::*;
use ffi3::pycapsule::PyCapsule_Import;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
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

    pub Date_FromDate: Option<
        unsafe extern "C" fn(
            year: c_int,
            month: c_int,
            day: c_int,
            cls: *mut PyTypeObject,
        ) -> *mut PyObject,
    >,
    pub DateTime_FromDateAndTime: Option<
        unsafe extern "C" fn(
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
    >,
    pub Time_FromTime: Option<
        unsafe extern "C" fn(
            hour: c_int,
            minute: c_int,
            second: c_int,
            microsecond: c_int,
            tzinfo: *mut PyObject,
            cls: *mut PyTypeObject,
        ) -> *mut PyObject,
    >,
    pub Delta_FromDelta: Option<
        unsafe extern "C" fn(
            days: c_int,
            seconds: c_int,
            microseconds: c_int,
            normalize: c_int,
            cls: *mut PyTypeObject,
        ) -> *mut PyObject,
    >,
    #[cfg(Py_3_7)]
    pub TimeZone_FromTimeZone: Option<
        unsafe extern "C" fn(offset: *mut PyObject, name: *mut PyObject) -> *mut PyObject,
    >,
    pub DateTime_FromTimestamp: Option<
        unsafe extern "C" fn(cls: *mut PyTypeObject, args: *mut PyObject, kwargs: *mut PyObject)
            -> *mut PyObject,
    >,
    pub Date_FromTimestamp: Option<
        unsafe extern "C" fn(cls: *mut PyTypeObject, args: *mut PyObject) -> *mut PyObject,
    >,
    #[cfg(Py_3_6)]
    pub DateTime_FromDateAndTimeAndFold: Option<
        unsafe extern "C" fn(
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
    >,
    #[cfg(Py_3_6)]
    pub Time_FromTimeAndFold: Option< unsafe extern "C" fn(
            hour: c_int,
            minute: c_int,
            second: c_int,
            microsecond: c_int,
            tzinfo: *mut PyObject,
            fold: c_int,
            cls: *mut PyTypeObject,
        ) -> *mut PyObject,
    >,
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

unsafe impl Sync for PyDateTime_CAPI {}

lazy_static! {
    pub static ref PyDateTimeAPI: PyDateTime_CAPI = unsafe { PyDateTime_IMPORT() };
}


#[inline(always)]
pub unsafe fn PyDateTime_IMPORT() -> PyDateTime_CAPI {
    // PyDateTime_CAPSULE_NAME is a macro in C
    let PyDateTime_CAPSULE_NAME = CString::new("datetime.datetime_CAPI").unwrap();

    let capsule = PyCapsule_Import(PyDateTime_CAPSULE_NAME.as_ptr(), 0);
    *(capsule as *const PyDateTime_CAPI)
}


#[inline(always)]
pub unsafe fn PyDate_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, PyDateTimeAPI.DateType) as c_int
}

#[inline(always)]
pub unsafe fn PyDateTime_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, PyDateTimeAPI.DateTimeType) as c_int
}

#[inline(always)]
pub unsafe fn PyTZInfo_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, PyDateTimeAPI.TZInfoType) as c_int
}

#[inline(always)]
pub unsafe fn PyTime_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, PyDateTimeAPI.TimeType) as c_int
}

#[inline(always)]
pub unsafe fn PyDelta_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, PyDateTimeAPI.DeltaType) as c_int
}

//
// Accessor functions
//
macro_rules! _access_field {
    ($obj:expr, $type: ident, $field:tt) => {
        (*($obj as *mut $type)).$field
    }
}

// Accessor functions for PyDateTime_Delta
macro_rules! _access_delta_field {
    ($obj:expr, $field:tt) => {
        _access_field!($obj, PyDateTime_Delta, $field)
    }
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
