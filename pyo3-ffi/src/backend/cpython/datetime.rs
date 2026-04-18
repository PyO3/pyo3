use crate::datetime::{
    PyDateTime_CAPI, PyDateTime_CAPSULE_NAME, PyDateTime_Date, PyDateTime_DateTime,
    PyDateTime_Delta, PyDateTime_Time,
};
use crate::{PyObject, Py_None};
use std::ffi::{c_int, c_uchar};

#[cfg(GraalPy)]
use crate::{PyLong_AsLong, PyLong_Check, PyObject_GetAttrString, Py_DECREF};

#[cfg(PyPy)]
use crate::datetime::PyDateTime_Import;

#[cfg(not(PyPy))]
use crate::PyCapsule_Import;

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_GET_YEAR(o: *mut PyObject) -> c_int {
    let data = (*(o as *mut PyDateTime_Date)).data;
    (c_int::from(data[0]) << 8) | c_int::from(data[1])
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_GET_MONTH(o: *mut PyObject) -> c_int {
    let data = (*(o as *mut PyDateTime_Date)).data;
    c_int::from(data[2])
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_GET_DAY(o: *mut PyObject) -> c_int {
    let data = (*(o as *mut PyDateTime_Date)).data;
    c_int::from(data[3])
}

#[cfg(not(any(PyPy, GraalPy)))]
macro_rules! _PyDateTime_GET_HOUR {
    ($o: expr, $offset:expr) => {
        c_int::from((*$o).data[$offset + 0])
    };
}

#[cfg(not(any(PyPy, GraalPy)))]
macro_rules! _PyDateTime_GET_MINUTE {
    ($o: expr, $offset:expr) => {
        c_int::from((*$o).data[$offset + 1])
    };
}

#[cfg(not(any(PyPy, GraalPy)))]
macro_rules! _PyDateTime_GET_SECOND {
    ($o: expr, $offset:expr) => {
        c_int::from((*$o).data[$offset + 2])
    };
}

#[cfg(not(any(PyPy, GraalPy)))]
macro_rules! _PyDateTime_GET_MICROSECOND {
    ($o: expr, $offset:expr) => {
        (c_int::from((*$o).data[$offset + 3]) << 16)
            | (c_int::from((*$o).data[$offset + 4]) << 8)
            | c_int::from((*$o).data[$offset + 5])
    };
}

#[cfg(not(any(PyPy, GraalPy)))]
macro_rules! _PyDateTime_GET_FOLD {
    ($o: expr) => {
        (*$o).fold
    };
}

#[cfg(not(any(PyPy, GraalPy)))]
macro_rules! _PyDateTime_GET_TZINFO {
    ($o: expr) => {
        if (*$o).hastzinfo != 0 {
            (*$o).tzinfo
        } else {
            Py_None()
        }
    };
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_HOUR(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_HOUR!(o as *mut PyDateTime_DateTime, 4)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_MINUTE(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MINUTE!(o as *mut PyDateTime_DateTime, 4)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_SECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_SECOND!(o as *mut PyDateTime_DateTime, 4)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_MICROSECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MICROSECOND!(o as *mut PyDateTime_DateTime, 4)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_FOLD(o: *mut PyObject) -> c_uchar {
    _PyDateTime_GET_FOLD!(o as *mut PyDateTime_DateTime)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_TZINFO(o: *mut PyObject) -> *mut PyObject {
    _PyDateTime_GET_TZINFO!(o as *mut PyDateTime_DateTime)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_HOUR(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_HOUR!(o as *mut PyDateTime_Time, 0)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_MINUTE(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MINUTE!(o as *mut PyDateTime_Time, 0)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_SECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_SECOND!(o as *mut PyDateTime_Time, 0)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_MICROSECOND(o: *mut PyObject) -> c_int {
    _PyDateTime_GET_MICROSECOND!(o as *mut PyDateTime_Time, 0)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_FOLD(o: *mut PyObject) -> c_uchar {
    _PyDateTime_GET_FOLD!(o as *mut PyDateTime_Time)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_TZINFO(o: *mut PyObject) -> *mut PyObject {
    _PyDateTime_GET_TZINFO!(o as *mut PyDateTime_Time)
}

#[cfg(not(any(PyPy, GraalPy)))]
macro_rules! _access_field {
    ($obj:expr, $type: ident, $field:ident) => {
        (*($obj as *mut $type)).$field
    };
}

#[cfg(not(any(PyPy, GraalPy)))]
macro_rules! _access_delta_field {
    ($obj:expr, $field:ident) => {
        _access_field!($obj, PyDateTime_Delta, $field)
    };
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_DELTA_GET_DAYS(o: *mut PyObject) -> c_int {
    _access_delta_field!(o, days)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_DELTA_GET_SECONDS(o: *mut PyObject) -> c_int {
    _access_delta_field!(o, seconds)
}

#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyDateTime_DELTA_GET_MICROSECONDS(o: *mut PyObject) -> c_int {
    _access_delta_field!(o, microseconds)
}

#[cfg(GraalPy)]
#[inline]
unsafe fn _get_attr(obj: *mut PyObject, field: &std::ffi::CStr) -> c_int {
    let result = PyObject_GetAttrString(obj, field.as_ptr());
    Py_DECREF(result);
    if PyLong_Check(result) == 1 {
        PyLong_AsLong(result) as c_int
    } else {
        0
    }
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_GET_YEAR(o: *mut PyObject) -> c_int {
    _get_attr(o, c"year")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_GET_MONTH(o: *mut PyObject) -> c_int {
    _get_attr(o, c"month")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_GET_DAY(o: *mut PyObject) -> c_int {
    _get_attr(o, c"day")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_HOUR(o: *mut PyObject) -> c_int {
    _get_attr(o, c"hour")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_MINUTE(o: *mut PyObject) -> c_int {
    _get_attr(o, c"minute")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_SECOND(o: *mut PyObject) -> c_int {
    _get_attr(o, c"second")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_MICROSECOND(o: *mut PyObject) -> c_int {
    _get_attr(o, c"microsecond")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_FOLD(o: *mut PyObject) -> c_int {
    _get_attr(o, c"fold")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_DATE_GET_TZINFO(o: *mut PyObject) -> *mut PyObject {
    let res = PyObject_GetAttrString(o, c"tzinfo".as_ptr().cast());
    Py_DECREF(res);
    res
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_HOUR(o: *mut PyObject) -> c_int {
    _get_attr(o, c"hour")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_MINUTE(o: *mut PyObject) -> c_int {
    _get_attr(o, c"minute")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_SECOND(o: *mut PyObject) -> c_int {
    _get_attr(o, c"second")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_MICROSECOND(o: *mut PyObject) -> c_int {
    _get_attr(o, c"microsecond")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_FOLD(o: *mut PyObject) -> c_int {
    _get_attr(o, c"fold")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_TIME_GET_TZINFO(o: *mut PyObject) -> *mut PyObject {
    let res = PyObject_GetAttrString(o, c"tzinfo".as_ptr().cast());
    Py_DECREF(res);
    res
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_DELTA_GET_DAYS(o: *mut PyObject) -> c_int {
    _get_attr(o, c"days")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_DELTA_GET_SECONDS(o: *mut PyObject) -> c_int {
    _get_attr(o, c"seconds")
}

#[cfg(GraalPy)]
#[inline]
pub unsafe fn PyDateTime_DELTA_GET_MICROSECONDS(o: *mut PyObject) -> c_int {
    _get_attr(o, c"microseconds")
}

#[cfg(PyPy)]
extern_libpython! {
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
    #[link_name = "PyPyDateTime_GET_FOLD"]
    pub fn PyDateTime_DATE_GET_FOLD(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_DATE_GET_TZINFO"]
    #[cfg(Py_3_10)]
    pub fn PyDateTime_DATE_GET_TZINFO(o: *mut PyObject) -> *mut PyObject;

    #[link_name = "PyPyDateTime_TIME_GET_HOUR"]
    pub fn PyDateTime_TIME_GET_HOUR(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_TIME_GET_MINUTE"]
    pub fn PyDateTime_TIME_GET_MINUTE(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_TIME_GET_SECOND"]
    pub fn PyDateTime_TIME_GET_SECOND(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_TIME_GET_MICROSECOND"]
    pub fn PyDateTime_TIME_GET_MICROSECOND(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_TIME_GET_FOLD"]
    pub fn PyDateTime_TIME_GET_FOLD(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_TIME_GET_TZINFO"]
    #[cfg(Py_3_10)]
    pub fn PyDateTime_TIME_GET_TZINFO(o: *mut PyObject) -> *mut PyObject;

    #[link_name = "PyPyDateTime_DELTA_GET_DAYS"]
    pub fn PyDateTime_DELTA_GET_DAYS(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_DELTA_GET_SECONDS"]
    pub fn PyDateTime_DELTA_GET_SECONDS(o: *mut PyObject) -> c_int;
    #[link_name = "PyPyDateTime_DELTA_GET_MICROSECONDS"]
    pub fn PyDateTime_DELTA_GET_MICROSECONDS(o: *mut PyObject) -> c_int;
}

pub unsafe fn import_datetime_api() -> *mut PyDateTime_CAPI {
    #[cfg(PyPy)]
    {
        PyDateTime_Import()
    }

    #[cfg(not(PyPy))]
    {
        PyCapsule_Import(PyDateTime_CAPSULE_NAME.as_ptr(), 1) as *mut PyDateTime_CAPI
    }
}
