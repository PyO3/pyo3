use std::os::raw::c_int;
use ffi3::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyRange_Type: PyTypeObject;
    pub static mut PyRangeIter_Type: PyTypeObject;
    pub static mut PyLongRangeIter_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyRange_Check(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyRange_Type) as c_int
}

