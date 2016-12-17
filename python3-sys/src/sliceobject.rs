use libc::c_int;
use pyport::Py_ssize_t;
use object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    static mut _Py_EllipsisObject: PyObject;
}

#[inline(always)]
pub unsafe fn Py_Ellipsis() -> *mut PyObject {
    &mut _Py_EllipsisObject
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PySlice_Type: PyTypeObject;
    pub static mut PyEllipsis_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PySlice_Check(op: *mut PyObject) -> c_int {
     (Py_TYPE(op) == &mut PySlice_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PySlice_New(start: *mut PyObject, stop: *mut PyObject,
                       step: *mut PyObject) -> *mut PyObject;
    pub fn PySlice_GetIndices(r: *mut PyObject, length: Py_ssize_t,
                              start: *mut Py_ssize_t, stop: *mut Py_ssize_t,
                              step: *mut Py_ssize_t) -> c_int;
    pub fn PySlice_GetIndicesEx(r: *mut PyObject, length: Py_ssize_t,
                                start: *mut Py_ssize_t, stop: *mut Py_ssize_t,
                                step: *mut Py_ssize_t,
                                slicelength: *mut Py_ssize_t)
     -> c_int;
}

