use std::os::raw::c_int;
use ffi2::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyRange_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyRange_Check(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyRange_Type;
    (Py_TYPE(op) == u) as c_int
}

