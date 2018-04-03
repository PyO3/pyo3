use std::os::raw::{c_int, c_long};
use ffi3::object::*;
use ffi3::longobject::PyLongObject;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyBool_Type")]
    pub static mut PyBool_Type: PyTypeObject;
    static mut _Py_FalseStruct: PyLongObject;
    #[cfg_attr(PyPy, link_name="\u{1}__PyPy_TrueStruct")]
    static mut _Py_TrueStruct: PyLongObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyBool_FromLong")]
    pub fn PyBool_FromLong(arg1: c_long) -> *mut PyObject;
}

#[inline(always)]
#[cfg_attr(PyPy, link_name="\u{1}_PyPyBool_Check")]
pub unsafe fn PyBool_Check(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyBool_Type) as c_int
}

#[inline(always)]
pub unsafe fn Py_False() -> *mut PyObject {
    &mut _Py_FalseStruct as *mut PyLongObject as *mut PyObject
}

#[inline(always)]
pub unsafe fn Py_True() -> *mut PyObject {
    &mut _Py_TrueStruct as *mut PyLongObject as *mut PyObject
}