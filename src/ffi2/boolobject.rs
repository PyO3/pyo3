use ffi2::intobject::PyIntObject;
use ffi2::object::*;
use std::os::raw::{c_int, c_long};

pub type PyBoolObject = PyIntObject;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyBool_Type: PyTypeObject;
    static mut _Py_ZeroStruct: PyIntObject;
    static mut _Py_TrueStruct: PyIntObject;
    pub fn PyBool_FromLong(arg1: c_long) -> *mut PyObject;
}

#[inline]
pub unsafe fn PyBool_Check(op: *mut PyObject) -> c_int {
    let u: *mut PyTypeObject = &mut PyBool_Type;
    (Py_TYPE(op) == u) as c_int
}

#[inline]
pub unsafe fn Py_False() -> *mut PyObject {
    &mut _Py_ZeroStruct as *mut PyBoolObject as *mut PyObject
}

#[inline]
pub unsafe fn Py_True() -> *mut PyObject {
    &mut _Py_TrueStruct as *mut PyBoolObject as *mut PyObject
}
