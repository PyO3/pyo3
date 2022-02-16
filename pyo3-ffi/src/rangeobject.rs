use crate::object::*;
use std::os::raw::c_int;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyRange_Type")]
    pub static mut PyRange_Type: PyTypeObject;
    pub static mut PyRangeIter_Type: PyTypeObject;
    pub static mut PyLongRangeIter_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyRange_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut_shim!(PyRange_Type)) as c_int
}
