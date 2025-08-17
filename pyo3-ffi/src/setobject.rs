use crate::object::*;
#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
use crate::pyport::Py_hash_t;
use crate::pyport::Py_ssize_t;
use std::ffi::c_int;
use std::ptr::addr_of_mut;

pub const PySet_MINSIZE: usize = 8;

#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
#[repr(C)]
#[derive(Debug)]
pub struct setentry {
    pub key: *mut PyObject,
    pub hash: Py_hash_t,
}

#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
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

// skipped
#[inline]
#[cfg(all(not(any(PyPy, GraalPy)), not(Py_LIMITED_API)))]
pub unsafe fn PySet_GET_SIZE(so: *mut PyObject) -> Py_ssize_t {
    debug_assert_eq!(PyAnySet_Check(so), 1);
    let so = so.cast::<PySetObject>();
    (*so).used
}

// skipped _PySet_Dummy

extern "C" {
    #[cfg(not(Py_LIMITED_API))]
    #[cfg_attr(PyPy, link_name = "_PyPySet_NextEntry")]
    pub fn _PySet_NextEntry(
        set: *mut PyObject,
        pos: *mut Py_ssize_t,
        key: *mut *mut PyObject,
        hash: *mut super::Py_hash_t,
    ) -> c_int;

    // skipped non-limited _PySet_Update
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySet_Type")]
    pub static mut PySet_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyFrozenSet_Type")]
    pub static mut PyFrozenSet_Type: PyTypeObject;
    pub static mut PySetIter_Type: PyTypeObject;
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySet_New")]
    pub fn PySet_New(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyFrozenSet_New")]
    pub fn PyFrozenSet_New(arg1: *mut PyObject) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPySet_Add")]
    pub fn PySet_Add(set: *mut PyObject, key: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySet_Clear")]
    pub fn PySet_Clear(set: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySet_Contains")]
    pub fn PySet_Contains(anyset: *mut PyObject, key: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySet_Discard")]
    pub fn PySet_Discard(set: *mut PyObject, key: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPySet_Pop")]
    pub fn PySet_Pop(set: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPySet_Size")]
    pub fn PySet_Size(anyset: *mut PyObject) -> Py_ssize_t;

    #[cfg(PyPy)]
    #[link_name = "PyPyFrozenSet_CheckExact"]
    pub fn PyFrozenSet_CheckExact(ob: *mut PyObject) -> c_int;
}

#[inline]
#[cfg(not(any(PyPy, GraalPy)))]
pub unsafe fn PyFrozenSet_CheckExact(ob: *mut PyObject) -> c_int {
    (Py_TYPE(ob) == addr_of_mut!(PyFrozenSet_Type)) as c_int
}

extern "C" {
    #[cfg(PyPy)]
    #[link_name = "PyPyFrozenSet_Check"]
    pub fn PyFrozenSet_Check(ob: *mut PyObject) -> c_int;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyFrozenSet_Check(ob: *mut PyObject) -> c_int {
    (Py_TYPE(ob) == addr_of_mut!(PyFrozenSet_Type)
        || PyType_IsSubtype(Py_TYPE(ob), addr_of_mut!(PyFrozenSet_Type)) != 0) as c_int
}

extern "C" {
    #[cfg(PyPy)]
    #[link_name = "PyPyAnySet_CheckExact"]
    pub fn PyAnySet_CheckExact(ob: *mut PyObject) -> c_int;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyAnySet_CheckExact(ob: *mut PyObject) -> c_int {
    (Py_TYPE(ob) == addr_of_mut!(PySet_Type) || Py_TYPE(ob) == addr_of_mut!(PyFrozenSet_Type))
        as c_int
}

#[inline]
pub unsafe fn PyAnySet_Check(ob: *mut PyObject) -> c_int {
    (PyAnySet_CheckExact(ob) != 0
        || PyType_IsSubtype(Py_TYPE(ob), addr_of_mut!(PySet_Type)) != 0
        || PyType_IsSubtype(Py_TYPE(ob), addr_of_mut!(PyFrozenSet_Type)) != 0) as c_int
}

#[inline]
#[cfg(Py_3_10)]
pub unsafe fn PySet_CheckExact(op: *mut PyObject) -> c_int {
    crate::Py_IS_TYPE(op, addr_of_mut!(PySet_Type))
}

extern "C" {
    #[cfg(PyPy)]
    #[link_name = "PyPySet_Check"]
    pub fn PySet_Check(ob: *mut PyObject) -> c_int;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PySet_Check(ob: *mut PyObject) -> c_int {
    (Py_TYPE(ob) == addr_of_mut!(PySet_Type)
        || PyType_IsSubtype(Py_TYPE(ob), addr_of_mut!(PySet_Type)) != 0) as c_int
}
