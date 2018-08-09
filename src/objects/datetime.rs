use err::PyResult;
use object::PyObject;
use std::os::raw::c_int;
use ffi::{PyDateTimeAPI};
use ffi::{PyDateTime_DateType, PyDate_Check};
use ffi::{PyDateTime_Date_GET_YEAR, PyDateTime_Date_GET_MONTH, PyDateTime_Date_GET_DAY};
use ffi::{PyDateTime_DateTimeType, PyDateTime_Check};
use ffi::{PyDateTime_DateTime_GET_YEAR, PyDateTime_DateTime_GET_MONTH, PyDateTime_DateTime_GET_DAY};
use ffi::{PyDateTime_TIME_GET_HOUR, PyDateTime_TIME_GET_MINUTE,
          PyDateTime_TIME_GET_SECOND, PyDateTime_TIME_GET_MICROSECOND};
use ffi::{PyDateTime_DATE_GET_HOUR, PyDateTime_DATE_GET_MINUTE,
          PyDateTime_DATE_GET_SECOND, PyDateTime_DATE_GET_MICROSECOND};
use ffi::{PyDateTime_DeltaType, PyDelta_Check};
use ffi::{PyDateTime_DELTA_GET_DAYS, PyDateTime_DELTA_GET_SECONDS, PyDateTime_DELTA_GET_MICROSECONDS};
use ffi::{PyDateTime_TimeType, PyTime_Check};
use ffi::{PyDateTime_TZInfoType, PyTZInfo_Check};

#[cfg(Py_3_6)]
use ffi::{PyDateTime_DATE_GET_FOLD, PyDateTime_TIME_GET_FOLD};

use python::{Python, ToPyPointer};
use instance::Py;

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
        let y = year as c_int;
        let m = month as c_int;
        let d = day as c_int;

        unsafe {
            let ptr = PyDateTimeAPI.Date_FromDate.unwrap()(y, m, d, PyDateTimeAPI.DateType);
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    pub fn from_timestamp(py: Python, args: &PyObject) -> PyResult<Py<PyDate>> {
        unsafe {
            let ptr = PyDateTimeAPI.Date_FromTimestamp.unwrap()
                (PyDateTimeAPI.DateType, args.as_ptr());
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }


}

impl PyDateComponentAccess for PyDate {
    fn get_year(&self) -> u32 {
        unsafe {
            PyDateTime_Date_GET_YEAR(self.as_ptr()) as u32
        }
    }

    fn get_month(&self) -> u32 {
        unsafe {
            PyDateTime_Date_GET_MONTH(self.as_ptr()) as u32
        }
    }

    fn get_day(&self) -> u32 {
        unsafe {
            PyDateTime_Date_GET_DAY(self.as_ptr()) as u32
        }
    }
}


// datetime.datetime bindings
pub struct PyDateTime(PyObject);
pyobject_native_type!(PyDateTime, PyDateTime_DateTimeType, PyDateTime_Check);


impl PyDateTime {
    pub fn new(py: Python, year: u32, month: u32, day: u32,
               hour: u32, minute: u32, second: u32, microsecond: u32,
               tzinfo: &PyObject) -> PyResult<Py<PyDateTime>> {
        let y = year as c_int;
        let mo = month as c_int;
        let d = day as c_int;
        let h = hour as c_int;
        let mi = minute as c_int;
        let s = second as c_int;
        let u = microsecond as c_int;

        unsafe {
            let ptr = PyDateTimeAPI.DateTime_FromDateAndTime.unwrap()(
                y, mo, d, h, mi, s, u, tzinfo.as_ptr(),
                PyDateTimeAPI.DateTimeType
            );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    pub fn from_timestamp(py: Python, args: &PyObject, kwargs: &PyObject) ->
            PyResult<Py<PyDateTime>> {

        unsafe {
            let ptr = PyDateTimeAPI.DateTime_FromTimestamp.unwrap()
                (PyDateTimeAPI.DateTimeType, args.as_ptr(), kwargs.as_ptr());
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

}

impl PyDateComponentAccess for PyDateTime {
    fn get_year(&self) -> u32 {
        unsafe {
            PyDateTime_DateTime_GET_YEAR(self.as_ptr()) as u32
        }
    }

    fn get_month(&self) -> u32 {
        unsafe {
            PyDateTime_DateTime_GET_MONTH(self.as_ptr()) as u32
        }
    }

    fn get_day(&self) -> u32 {
        unsafe {
            PyDateTime_DateTime_GET_DAY(self.as_ptr()) as u32
        }
    }
}

impl PyTimeComponentAccess for PyDateTime {
    fn get_hour(&self) -> u32 {
        unsafe {
            PyDateTime_DATE_GET_HOUR(self.as_ptr()) as u32
        }
    }

    fn get_minute(&self) -> u32 {
        unsafe {
            PyDateTime_DATE_GET_MINUTE(self.as_ptr()) as u32
        }
    }

    fn get_second(&self) -> u32 {
        unsafe {
            PyDateTime_DATE_GET_SECOND(self.as_ptr()) as u32
        }
    }

    fn get_microsecond(&self) -> u32 {
        unsafe {
            PyDateTime_DATE_GET_MICROSECOND(self.as_ptr()) as u32
        }
    }

    #[cfg(Py_3_6)]
    fn get_fold(&self) -> u8 {
        unsafe {
            PyDateTime_DATE_GET_FOLD(self.as_ptr()) as u8
        }
    }
}


// datetime.time
pub struct PyTime(PyObject);
pyobject_native_type!(PyTime, PyDateTime_TimeType, PyTime_Check);

impl PyTime {
    pub fn new(py: Python, hour: u32, minute: u32, second: u32,
               microsecond: u32, tzinfo: &PyObject) -> PyResult<Py<PyTime>> {
        let h = hour as c_int;
        let m = minute as c_int;
        let s = second as c_int;
        let u = microsecond as c_int;
        let tzi = tzinfo.as_ptr();

        unsafe {
            let ptr = PyDateTimeAPI.Time_FromTime.unwrap()(
                h, m, s, u, tzi, PyDateTimeAPI.TimeType
                );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }

    #[cfg(Py_3_6)]
    pub fn new_with_fold(py: Python, hour: u32, minute: u32, second: u32,
                         microsecond: u32, tzinfo: &PyObject,
                         fold: bool) -> PyResult<Py<PyTime>> {
        let h = hour as c_int;
        let m = minute as c_int;
        let s = second as c_int;
        let u = microsecond as c_int;

        let f = fold as c_int;
        unsafe {
            let ptr = PyDateTimeAPI.Time_FromTimeAndFold.unwrap()
                (h, m, s, u, tzinfo.as_ptr(), f, PyDateTimeAPI.TimeType);
            Py::from_owned_ptr_or_err(py, ptr)
        }

    }

}

impl PyTimeComponentAccess for PyTime {
    fn get_hour(&self) -> u32 {
        unsafe {
            PyDateTime_TIME_GET_HOUR(self.as_ptr()) as u32
        }
    }

    fn get_minute(&self) -> u32 {
        unsafe {
            PyDateTime_TIME_GET_MINUTE(self.as_ptr()) as u32
        }
    }

    fn get_second(&self) -> u32 {
        unsafe {
            PyDateTime_TIME_GET_SECOND(self.as_ptr()) as u32
        }
    }

    fn get_microsecond(&self) -> u32 {
        unsafe {
            PyDateTime_TIME_GET_MICROSECOND(self.as_ptr()) as u32
        }
    }

    #[cfg(Py_3_6)]
    fn get_fold(&self) -> u8 {
        unsafe {
            PyDateTime_TIME_GET_FOLD(self.as_ptr()) as u8
        }
    }
}


// datetime.tzinfo bindings
pub struct PyTzInfo(PyObject);
pyobject_native_type!(PyTzInfo, PyDateTime_TZInfoType, PyTZInfo_Check);


// datetime.timedelta bindings
pub struct PyDelta(PyObject);
pyobject_native_type!(PyDelta, PyDateTime_DeltaType, PyDelta_Check);

impl PyDelta {
    pub fn new(py: Python, days: i32, seconds: i32, microseconds: i32,
               normalize: bool) -> PyResult<Py<PyDelta>> {
        let d = days as c_int;
        let s = seconds as c_int;
        let u = microseconds as c_int;
        let n = normalize as c_int;

        unsafe {
            let ptr = PyDateTimeAPI.Delta_FromDelta.unwrap()(
                d, s, u, n, PyDateTimeAPI.DeltaType
                );
            Py::from_owned_ptr_or_err(py, ptr)
        }
    }
}

impl PyDeltaComponentAccess for PyDelta {
    fn get_days(&self) -> i32 {
        unsafe {
            PyDateTime_DELTA_GET_DAYS(self.as_ptr()) as i32
        }
    }

    fn get_seconds(&self) -> i32 {
        unsafe {
            PyDateTime_DELTA_GET_SECONDS(self.as_ptr()) as i32
        }
    }

    fn get_microseconds(&self) -> i32 {
        unsafe {
            PyDateTime_DELTA_GET_MICROSECONDS(self.as_ptr()) as i32
        }
    }
}
