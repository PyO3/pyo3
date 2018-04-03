use std::os::raw::{c_int, c_long};
use ffi2::object::*;
use ffi2::intobject::PyIntObject;

pub type PyBoolObject = PyIntObject;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyBool_Type")]
    pub static mut PyBool_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name="\u{1}__PyPy_ZeroStruct")]
    static mut _Py_ZeroStruct: PyIntObject;
    #[cfg_attr(PyPy, link_name="\u{1}__PyPy_TrueStruct")]
    static mut _Py_TrueStruct: PyIntObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyBool_FromLong")]
    pub fn PyBool_FromLong(arg1: c_long) -> *mut PyObject;
}

#[inline(always)]
#[cfg_attr(PyPy, link_name="\u{1}_PyPyBool_Check")]
pub unsafe fn PyBool_Check(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyBool_Type;
    (Py_TYPE(op) == u) as c_int
}

#[inline(always)]
pub unsafe fn Py_False() -> *mut PyObject {
    &mut _Py_ZeroStruct as *mut PyBoolObject as *mut PyObject
}

#[inline(always)]
pub unsafe fn Py_True() -> *mut PyObject {
    &mut _Py_TrueStruct as *mut PyBoolObject as *mut PyObject
}