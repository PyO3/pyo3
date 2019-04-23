use crate::ffi3::object::*;
use crate::ffi3::pyport::Py_ssize_t;
use std::os::raw::c_int;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "_PyPy_EllipsisObject")]
    static mut _Py_EllipsisObject: PyObject;
}

#[inline]
pub unsafe fn Py_Ellipsis() -> *mut PyObject {
    &mut _Py_EllipsisObject
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySlice_Type")]
    pub static mut PySlice_Type: PyTypeObject;
    pub static mut PyEllipsis_Type: PyTypeObject;
}

#[inline]
#[cfg_attr(PyPy, link_name = "PyPySlice_Check")]
pub unsafe fn PySlice_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PySlice_Type) as c_int
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySlice_New")]
    pub fn PySlice_New(
        start: *mut PyObject,
        stop: *mut PyObject,
        step: *mut PyObject,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPySlice_GetIndices")]
    pub fn PySlice_GetIndices(
        r: *mut PyObject,
        length: Py_ssize_t,
        start: *mut Py_ssize_t,
        stop: *mut Py_ssize_t,
        step: *mut Py_ssize_t,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySlice_GetIndicesEx")]
    pub fn PySlice_GetIndicesEx(
        r: *mut PyObject,
        length: Py_ssize_t,
        start: *mut Py_ssize_t,
        stop: *mut Py_ssize_t,
        step: *mut Py_ssize_t,
        slicelength: *mut Py_ssize_t,
    ) -> c_int;
}
