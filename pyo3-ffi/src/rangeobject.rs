use crate::object::*;
use std::ffi::c_int;

#[cfg(not(RustPython))]
extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyRange_Type")]
    pub static mut PyRange_Type: PyTypeObject;
    pub static mut PyRangeIter_Type: PyTypeObject;
    pub static mut PyLongRangeIter_Type: PyTypeObject;
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyRange_Check(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyRange_Type)
}

extern_libpython! {
    #[cfg(RustPython)]
    pub fn PyRange_Check(op: *mut PyObject) -> c_int;
}
