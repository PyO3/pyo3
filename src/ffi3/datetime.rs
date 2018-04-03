use std::os::raw::c_int;
use std::ffi::CString;
use std::option::Option;
use ffi3::object::*;
use ffi3::pycapsule::PyCapsule_Import;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyDateTime_Date: PyTypeObject;
    pub static mut PyDateTime_Time: PyTypeObject;
    pub static mut PyDateTime_DateTime: PyTypeObject;

    pub static mut PyDateTime_Delta: PyTypeObject;
    pub static mut PyDateTime_TZInfo: PyTypeObject;
}


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyDateTime_CAPI {
    pub DateType: *mut PyTypeObject,
    pub DateTimeType: *mut PyTypeObject,
    pub TimeType: *mut PyTypeObject,
    pub DeltaType: *mut PyTypeObject,
    pub TZInfoType: *mut PyTypeObject,
    /* pub TimeZone_UTC: *mut PyObject, */
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
    /* pub TimeZone_FromTimeZone: Option< */
    /*     unsafe extern "C" fn(offset: *mut PyObject, name: *mut PyObject) -> *mut PyObject, */
    /* >, */
    pub DateTime_FromTimestamp: Option<
        unsafe extern "C" fn(cls: *mut PyObject, args: *mut PyObject, kwargs: *mut PyObject)
            -> *mut PyObject,
    >,
    pub Date_FromTimestamp: Option<
        unsafe extern "C" fn(cls: *mut PyObject, args: *mut PyObject) -> *mut PyObject,
    >,
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

unsafe impl Sync for PyDateTime_CAPI {}

#[inline(always)]
pub unsafe fn PyDateTime_IMPORT() -> PyDateTime_CAPI {
    // PyDateTime_CAPSULE_NAME is a macro in C
    let PyDateTime_CAPSULE_NAME = CString::new("datetime.datetime_CAPI").unwrap();

    let capsule = PyCapsule_Import(PyDateTime_CAPSULE_NAME.as_ptr(), 0);
    *(capsule as *const PyDateTime_CAPI)
}


#[inline(always)]
pub unsafe fn PyDate_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyDateTime_Date) as c_int
}
