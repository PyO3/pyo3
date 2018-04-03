use object::PyObject;
use std::os::raw::c_int;
use ffi::{PyDateTime_CAPI, PyDateTime_IMPORT};
use python::{Python, ToPyPointer};
use instance::Py;

pub struct PyDate(PyObject);
pyobject_convert!(PyDate);
pyobject_nativetype!(PyDate, PyDateTime_Date, PyDate_Check);

lazy_static! {
    static ref PyDateTimeAPI: PyDateTime_CAPI = unsafe { PyDateTime_IMPORT() };
}

impl PyDate {
    pub fn new(py: Python, year: u32, month: u32, day: u32) -> Py<PyDate> {
        let y = year as c_int;
        let m = month as c_int;
        let d = day as c_int;

        unsafe {
            let ptr = PyDateTimeAPI.Date_FromDate.unwrap()(y, m, d, PyDateTimeAPI.DateType);
            Py::from_owned_ptr_or_panic(ptr)
        }
    }
}

