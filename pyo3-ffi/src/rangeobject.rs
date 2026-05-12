use crate::object::*;
use core::ffi::c_int;

extern_libpython! {
    #[cfg(not(RustPython))]
    #[cfg_attr(PyPy, link_name = "PyPyRange_Type")]
    pub static mut PyRange_Type: PyTypeObject;
    #[cfg(not(RustPython))]
    pub static mut PyRangeIter_Type: PyTypeObject;
    #[cfg(not(RustPython))]
    pub static mut PyLongRangeIter_Type: PyTypeObject;

    #[cfg(RustPython)]
    pub fn PyRange_Check(op: *mut PyObject) -> c_int;
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyRange_Check(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyRange_Type)
}
