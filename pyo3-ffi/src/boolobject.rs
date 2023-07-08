use crate::longobject::PyLongObject;
use crate::object::*;
use std::os::raw::{c_int, c_long};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyBool_Type")]
    pub static mut PyBool_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyBool_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut_shim!(PyBool_Type)) as c_int
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "_PyPy_FalseStruct")]
    static mut _Py_FalseStruct: PyLongObject;
    #[cfg_attr(PyPy, link_name = "_PyPy_TrueStruct")]
    static mut _Py_TrueStruct: PyLongObject;
}

#[inline]
pub unsafe fn Py_False() -> *mut PyObject {
    addr_of_mut_shim!(_Py_FalseStruct) as *mut PyObject
}

#[inline]
pub unsafe fn Py_True() -> *mut PyObject {
    addr_of_mut_shim!(_Py_TrueStruct) as *mut PyObject
}

#[inline]
pub unsafe fn Py_IsTrue(x: *mut PyObject) -> c_int {
    Py_Is(x, Py_True())
}

#[inline]
pub unsafe fn Py_IsFalse(x: *mut PyObject) -> c_int {
    Py_Is(x, Py_False())
}

// skipped Py_RETURN_TRUE
// skipped Py_RETURN_FALSE

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyBool_FromLong")]
    pub fn PyBool_FromLong(arg1: c_long) -> *mut PyObject;
}
