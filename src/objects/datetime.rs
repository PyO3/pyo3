use err::PyResult;
use ffi::PyDateTimeAPI;
use ffi::{PyDateTime_Check, PyDateTime_DateTimeType};
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
use object::PyObject;
use std::os::raw::c_int;

#[cfg(Py_3_6)]
use ffi::{PyDateTime_DATE_GET_FOLD, PyDateTime_TIME_GET_FOLD};

use instance::Py;
use python::{Python, ToPyPointer};

// Traits
pub trait PyDateComponentAccess {
    fn get_year(&self) -> u32;
    fn get_month(&self) -> u32;
    fn get_day(&self) -> u32;
}

pub trait PyDeltaComponentAccess {
    fn get_days(&self) -> i32;
    fn get_seconds(&self) -> i32;
    fn get_microseconds(&self) -> i32;
}

pub trait PyTimeComponentAccess {
    fn get_hour(&self) -> u32;
    fn get_minute(&self) -> u32;
    fn get_second(&self) -> u32;
    fn get_microsecond(&self) -> u32;
    #[cfg(Py_3_6)]
    fn get_fold(&self) -> u8;
}

// datetime.date bindings
pub struct PyDate(PyObject);
pyobject_native_type!(PyDate, PyDateTime_DateType, PyDate_Check);

impl PyDate {
    pub fn new(py: Python, year: u32, month: u32, day: u32) -> PyResult<Py<PyDate>> {
        unsafe {
            let ptr = (PyDateTimeAPI.Date_FromDate)(
                year as c_int,
                month as c_int,
                day as c_int,
                PyDateTimeAPI.DateType,
            );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    pub fn from_timestamp(py: Python, args: &PyObject) -> PyResult<Py<PyDate>> {
        unsafe {
            let ptr = (PyDateTimeAPI.Date_FromTimestamp)(PyDateTimeAPI.DateType, args.as_ptr());
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }
}

impl PyDateComponentAccess for PyDate {
    fn get_year(&self) -> u32 {
        unsafe { PyDateTime_GET_YEAR(self.as_ptr()) as u32 }
    }

    fn get_month(&self) -> u32 {
        unsafe { PyDateTime_GET_MONTH(self.as_ptr()) as u32 }
    }

    fn get_day(&self) -> u32 {
        unsafe { PyDateTime_GET_DAY(self.as_ptr()) as u32 }
    }
}

// datetime.datetime bindings
pub struct PyDateTime(PyObject);
pyobject_native_type!(PyDateTime, PyDateTime_DateTimeType, PyDateTime_Check);

impl PyDateTime {
    pub fn new(
        py: Python,
        year: u32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
    ) -> PyResult<Py<PyDateTime>> {
        unsafe {
            let ptr = (PyDateTimeAPI.DateTime_FromDateAndTime)(
                year as c_int,
                month as c_int,
                day as c_int,
                hour as c_int,
                minute as c_int,
                second as c_int,
                microsecond as c_int,
                match tzinfo {
                    Some(o) => o.as_ptr(),
                    None => py.None().as_ptr(),
                },
                PyDateTimeAPI.DateTimeType,
            );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    pub fn from_timestamp(
        py: Python,
        args: &PyObject,
        kwargs: &PyObject,
    ) -> PyResult<Py<PyDateTime>> {
        unsafe {
            let ptr = (PyDateTimeAPI.DateTime_FromTimestamp)(
                PyDateTimeAPI.DateTimeType,
                args.as_ptr(),
                kwargs.as_ptr(),
            );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }
}

impl PyDateComponentAccess for PyDateTime {
    fn get_year(&self) -> u32 {
        unsafe { PyDateTime_GET_YEAR(self.as_ptr()) as u32 }
    }

    fn get_month(&self) -> u32 {
        unsafe { PyDateTime_GET_MONTH(self.as_ptr()) as u32 }
    }

    fn get_day(&self) -> u32 {
        unsafe { PyDateTime_GET_DAY(self.as_ptr()) as u32 }
    }
}

impl PyTimeComponentAccess for PyDateTime {
    fn get_hour(&self) -> u32 {
        unsafe { PyDateTime_DATE_GET_HOUR(self.as_ptr()) as u32 }
    }

    fn get_minute(&self) -> u32 {
        unsafe { PyDateTime_DATE_GET_MINUTE(self.as_ptr()) as u32 }
    }

    fn get_second(&self) -> u32 {
        unsafe { PyDateTime_DATE_GET_SECOND(self.as_ptr()) as u32 }
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
        hour: u32,
        minute: u32,
        second: u32,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
    ) -> PyResult<Py<PyTime>> {
        unsafe {
            let ptr = (PyDateTimeAPI.Time_FromTime)(
                hour as c_int,
                minute as c_int,
                second as c_int,
                microsecond as c_int,
                match tzinfo {
                    Some(o) => o.as_ptr(),
                    None => py.None().as_ptr(),
                },
                PyDateTimeAPI.TimeType,
            );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    #[cfg(Py_3_6)]
    pub fn new_with_fold(
        py: Python,
        hour: u32,
        minute: u32,
        second: u32,
        microsecond: u32,
        tzinfo: Option<&PyObject>,
        fold: bool,
    ) -> PyResult<Py<PyTime>> {
        unsafe {
            let ptr = (PyDateTimeAPI.Time_FromTimeAndFold)(
                hour as c_int,
                minute as c_int,
                second as c_int,
                microsecond as c_int,
                match tzinfo {
                    Some(o) => o.as_ptr(),
                    None => py.None().as_ptr(),
                },
                fold as c_int,
                PyDateTimeAPI.TimeType,
            );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }
}

impl PyTimeComponentAccess for PyTime {
    fn get_hour(&self) -> u32 {
        unsafe { PyDateTime_TIME_GET_HOUR(self.as_ptr()) as u32 }
    }

    fn get_minute(&self) -> u32 {
        unsafe { PyDateTime_TIME_GET_MINUTE(self.as_ptr()) as u32 }
    }

    fn get_second(&self) -> u32 {
        unsafe { PyDateTime_TIME_GET_SECOND(self.as_ptr()) as u32 }
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

impl PyDeltaComponentAccess for PyDelta {
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
