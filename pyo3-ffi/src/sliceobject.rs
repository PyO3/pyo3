use crate::object::*;
use crate::pyport::Py_ssize_t;
use std::ffi::c_int;
use std::ptr::addr_of_mut;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg(not(GraalPy))]
    #[cfg_attr(PyPy, link_name = "_PyPy_EllipsisObject")]
    static mut _Py_EllipsisObject: PyObject;

    #[cfg(GraalPy)]
    static mut _Py_EllipsisObjectReference: *mut PyObject;
}

#[inline]
pub unsafe fn Py_Ellipsis() -> *mut PyObject {
    #[cfg(not(GraalPy))]
    return addr_of_mut!(_Py_EllipsisObject);
    #[cfg(GraalPy)]
    return _Py_EllipsisObjectReference;
}

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
pub struct PySliceObject {
    pub ob_base: PyObject,
    #[cfg(not(GraalPy))]
    pub start: *mut PyObject,
    #[cfg(not(GraalPy))]
    pub stop: *mut PyObject,
    #[cfg(not(GraalPy))]
    pub step: *mut PyObject,
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySlice_Type")]
    pub static mut PySlice_Type: PyTypeObject;
    pub static mut PyEllipsis_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PySlice_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PySlice_Type)) as c_int
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySlice_New")]
    pub fn PySlice_New(
        start: *mut PyObject,
        stop: *mut PyObject,
        step: *mut PyObject,
    ) -> *mut PyObject;

    // skipped non-limited _PySlice_FromIndices
    // skipped non-limited _PySlice_GetLongIndices

    #[cfg_attr(PyPy, link_name = "PyPySlice_GetIndices")]
    pub fn PySlice_GetIndices(
        r: *mut PyObject,
        length: Py_ssize_t,
        start: *mut Py_ssize_t,
        stop: *mut Py_ssize_t,
        step: *mut Py_ssize_t,
    ) -> c_int;
}

#[inline]
pub unsafe fn PySlice_GetIndicesEx(
    slice: *mut PyObject,
    length: Py_ssize_t,
    start: *mut Py_ssize_t,
    stop: *mut Py_ssize_t,
    step: *mut Py_ssize_t,
    slicelength: *mut Py_ssize_t,
) -> c_int {
    if PySlice_Unpack(slice, start, stop, step) < 0 {
        *slicelength = 0;
        -1
    } else {
        *slicelength = PySlice_AdjustIndices(length, start, stop, *step);
        0
    }
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySlice_Unpack")]
    pub fn PySlice_Unpack(
        slice: *mut PyObject,
        start: *mut Py_ssize_t,
        stop: *mut Py_ssize_t,
        step: *mut Py_ssize_t,
    ) -> c_int;

    #[cfg_attr(all(PyPy, Py_3_10), link_name = "PyPySlice_AdjustIndices")]
    pub fn PySlice_AdjustIndices(
        length: Py_ssize_t,
        start: *mut Py_ssize_t,
        stop: *mut Py_ssize_t,
        step: Py_ssize_t,
    ) -> Py_ssize_t;
}
