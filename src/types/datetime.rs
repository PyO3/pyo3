use conversion::ToPyObject;
use err::PyResult;
use ffi;
use ffi::PyDateTimeAPI;
use ffi::{PyDateTime_Check, PyDateTime_DateTimeType};
#[cfg(Py_3_6)]
use ffi::{PyDateTime_DATE_GET_FOLD, PyDateTime_TIME_GET_FOLD};
use ffi::{
    PyDateTime_DATE_GET_HOUR, PyDateTime_DATE_GET_MICROSECOND, PyDateTime_DATE_GET_MINUTE,
    PyDateTime_DATE_GET_SECOND,
};
use ffi::{
    PyDateTime_DELTA_GET_DAYS, PyDateTime_DELTA_GET_MICROSECONDS, PyDateTime_DELTA_GET_SECONDS,
};
use ffi::{PyDateTime_DateType, PyDate_Check};
use ffi::{PyDateTime_DeltaType, PyDelta_Check};
use ffi::{PyDateTime_GET_DAY, PyDateTime_GET_MONTH, PyDateTime_GET_YEAR};
use ffi::{
    PyDateTime_TIME_GET_HOUR, PyDateTime_TIME_GET_MICROSECOND, PyDateTime_TIME_GET_MINUTE,
    PyDateTime_TIME_GET_SECOND,
};
use ffi::{PyDateTime_TZInfoType, PyTZInfo_Check};
use ffi::{PyDateTime_TimeType, PyTime_Check};
use instance::Py;
use object::PyObject;
use python::{Python, ToPyPointer};
use std::os::raw::c_int;
use std::ptr;
use types::PyTuple;

// Traits
pub trait PyDateAccess {
    fn get_year(&self) -> i32;
    fn get_month(&self) -> u8;
    fn get_day(&self) -> u8;
}

pub trait PyDeltaAccess {
    fn get_days(&self) -> i32;
    fn get_seconds(&self) -> i32;
    fn get_microseconds(&self) -> i32;
}

pub trait PyTimeAccess {
    fn get_hour(&self) -> u8;
    fn get_minute(&self) -> u8;
    fn get_second(&self) -> u8;
    fn get_microsecond(&self) -> u32;
    #[cfg(Py_3_6)]
    fn get_fold(&self) -> u8;
}

// datetime.date bindings
pub struct PyDate(PyObject);
pyobject_native_type!(PyDate, PyDateTime_DateType, PyDate_Check);

impl PyDate {
    pub fn new(py: Python, year: i32, month: u8, day: u8) -> PyResult<Py<PyDate>> {
        unsafe {
            let ptr = (PyDateTimeAPI.Date_FromDate)(
                year,
                c_int::from(month),
                c_int::from(day),
                PyDateTimeAPI.DateType,
            );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    pub fn from_timestamp(py: Python, timestamp: i64) -> PyResult<Py<PyDate>> {
        let args = PyTuple::new(py, &[timestamp]);

        unsafe {
            let ptr = (PyDateTimeAPI.Date_FromTimestamp)(PyDateTimeAPI.DateType, args.as_ptr());
            Py::from_owned_ptr_or_err(py, ptr)
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

// datetime.datetime bindings
pub struct PyDateTime(PyObject);
pyobject_native_type!(PyDateTime, PyDateTime_DateTimeType, PyDateTime_Check);

impl PyDateTime {
    pub fn new(
        py: Python,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
    ) -> PyResult<Py<PyDateTime>> {
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
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    pub fn from_timestamp(
        py: Python,
        timestamp: f64,
        time_zone_info: Option<&PyTzInfo>,
    ) -> PyResult<Py<PyDateTime>> {
        let timestamp: PyObject = timestamp.to_object(py);

        let time_zone_info: PyObject = match time_zone_info {
            Some(time_zone_info) => time_zone_info.to_object(py),
            None => py.None(),
        };

        let args = PyTuple::new(py, &[timestamp, time_zone_info]);

        unsafe {
            let ptr = (PyDateTimeAPI.DateTime_FromTimestamp)(
                PyDateTimeAPI.DateTimeType,
                args.as_ptr(),
                ptr::null_mut(),
            );
            Py::from_owned_ptr_or_err(py, ptr)
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

    #[cfg(Py_3_6)]
    fn get_fold(&self) -> u8 {
        unsafe { PyDateTime_DATE_GET_FOLD(self.as_ptr()) as u8 }
    }
}

// datetime.time
pub struct PyTime(PyObject);
pyobject_native_type!(PyTime, PyDateTime_TimeType, PyTime_Check);

impl PyTime {
    pub fn new(
        py: Python,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
    ) -> PyResult<Py<PyTime>> {
        unsafe {
            let ptr = (PyDateTimeAPI.Time_FromTime)(
                c_int::from(hour),
                c_int::from(minute),
                c_int::from(second),
                microsecond as c_int,
                opt_to_pyobj(py, tzinfo),
                PyDateTimeAPI.TimeType,
            );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    #[cfg(Py_3_6)]
    pub fn new_with_fold(
        py: Python,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
        fold: bool,
    ) -> PyResult<Py<PyTime>> {
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
            Py::from_owned_ptr_or_err(py, ptr)
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

    #[cfg(Py_3_6)]
    fn get_fold(&self) -> u8 {
        unsafe { PyDateTime_TIME_GET_FOLD(self.as_ptr()) as u8 }
    }
}

// datetime.tzinfo bindings
pub struct PyTzInfo(PyObject);
pyobject_native_type!(PyTzInfo, PyDateTime_TZInfoType, PyTZInfo_Check);

// datetime.timedelta bindings
pub struct PyDelta(PyObject);
pyobject_native_type!(PyDelta, PyDateTime_DeltaType, PyDelta_Check);

impl PyDelta {
    pub fn new(
        py: Python,
        days: i32,
        seconds: i32,
        microseconds: i32,
        normalize: bool,
    ) -> PyResult<Py<PyDelta>> {
        unsafe {
            let ptr = (PyDateTimeAPI.Delta_FromDelta)(
                days as c_int,
                seconds as c_int,
                microseconds as c_int,
                normalize as c_int,
                PyDateTimeAPI.DeltaType,
            );
            Py::from_owned_ptr_or_err(py, ptr)
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
unsafe fn opt_to_pyobj(py: Python, opt: Option<&PyObject>) -> *mut ffi::PyObject {
    // Convenience function for unpacking Options to either an Object or None
    match opt {
        Some(tzi) => tzi.as_ptr(),
        None => py.None().as_ptr(),
    }
}
