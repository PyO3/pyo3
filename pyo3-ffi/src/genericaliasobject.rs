use crate::object::{PyObject, PyTypeObject, Py_TYPE};
use crate::PyObject_TypeCheck;
use std::os::raw::c_int;
use std::ptr;

#[inline]
pub unsafe fn PyGenericAlias_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == ptr::addr_of_mut!(Py_GenericAliasType)) as c_int
}

#[inline]
pub unsafe fn PyGenericAlias_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, ptr::addr_of_mut!(Py_GenericAliasType))
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub fn Py_GenericAlias(origin: *mut PyObject, args: *mut PyObject) -> *mut PyObject;

    pub static mut Py_GenericAliasType: PyTypeObject;
}
