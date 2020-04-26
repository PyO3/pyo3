use crate::ffi::object::*;
use crate::ffi::pyport::{Py_hash_t, Py_ssize_t};
use std::os::raw::c_int;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySet_Type")]
    pub static mut PySet_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyFrozenSet_Type")]
    pub static mut PyFrozenSet_Type: PyTypeObject;
    pub static mut PySetIter_Type: PyTypeObject;
}

pub const PySet_MINSIZE: usize = 8;

#[repr(C)]
#[derive(Debug)]
pub struct setentry {
    pub key: *mut PyObject,
    pub hash: Py_hash_t,
}

#[repr(C)]
#[derive(Debug)]
pub struct PySetObject {
    pub ob_base: PyObject,
    pub fill: Py_ssize_t,
    pub used: Py_ssize_t,
    pub mask: Py_ssize_t,
    pub table: *mut setentry,
    pub hash: Py_hash_t,
    pub finger: Py_ssize_t,
    pub smalltable: [setentry; PySet_MINSIZE],
    pub weakreflist: *mut PyObject,
}

#[inline]
#[cfg_attr(PyPy, link_name = "PyPyFrozenSet_CheckExact")]
pub unsafe fn PyFrozenSet_CheckExact(ob: *mut PyObject) -> c_int {
    (Py_TYPE(ob) == &mut PyFrozenSet_Type) as c_int
}

#[inline]
#[cfg_attr(PyPy, link_name = "PyPyAnySet_CheckExact")]
pub unsafe fn PyAnySet_CheckExact(ob: *mut PyObject) -> c_int {
    (Py_TYPE(ob) == &mut PySet_Type || Py_TYPE(ob) == &mut PyFrozenSet_Type) as c_int
}

#[inline]
pub unsafe fn PyAnySet_Check(ob: *mut PyObject) -> c_int {
    (PyAnySet_CheckExact(ob) != 0
        || PyType_IsSubtype(Py_TYPE(ob), &mut PySet_Type) != 0
        || PyType_IsSubtype(Py_TYPE(ob), &mut PyFrozenSet_Type) != 0) as c_int
}

#[inline]
#[cfg_attr(PyPy, link_name = "PyPySet_Check")]
pub unsafe fn PySet_Check(ob: *mut PyObject) -> c_int {
    (Py_TYPE(ob) == &mut PySet_Type || PyType_IsSubtype(Py_TYPE(ob), &mut PySet_Type) != 0) as c_int
}

#[inline]
pub unsafe fn PyFrozenSet_Check(ob: *mut PyObject) -> c_int {
    (Py_TYPE(ob) == &mut PyFrozenSet_Type
        || PyType_IsSubtype(Py_TYPE(ob), &mut PyFrozenSet_Type) != 0) as c_int
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySet_New")]
    pub fn PySet_New(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyFrozenSet_New")]
    pub fn PyFrozenSet_New(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPySet_Size")]
    pub fn PySet_Size(anyset: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPySet_Clear")]
    pub fn PySet_Clear(set: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySet_Contains")]
    pub fn PySet_Contains(anyset: *mut PyObject, key: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySet_Discard")]
    pub fn PySet_Discard(set: *mut PyObject, key: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySet_Add")]
    pub fn PySet_Add(set: *mut PyObject, key: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySet_Pop")]
    pub fn PySet_Pop(set: *mut PyObject) -> *mut PyObject;

    #[cfg(not(Py_LIMITED_API))]
    #[cfg_attr(PyPy, link_name = "_PySet_NextEntry")]
    pub fn _PySet_NextEntry(
        set: *mut PyObject,
        pos: *mut Py_ssize_t,
        key: *mut *mut PyObject,
        hash: *mut super::Py_hash_t,
    ) -> c_int;
}
