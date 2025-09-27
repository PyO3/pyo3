use crate::object::*;
use crate::pyport::Py_ssize_t;
use std::ffi::{c_char, c_int};
use std::ptr::addr_of_mut;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyDict_Type")]
    pub static mut PyDict_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyDict_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_DICT_SUBCLASS)
}

#[inline]
pub unsafe fn PyDict_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyDict_Type)) as c_int
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyDict_New")]
    pub fn PyDict_New() -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDict_GetItem")]
    pub fn PyDict_GetItem(mp: *mut PyObject, key: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDict_GetItemWithError")]
    pub fn PyDict_GetItemWithError(mp: *mut PyObject, key: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDict_SetItem")]
    pub fn PyDict_SetItem(mp: *mut PyObject, key: *mut PyObject, item: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyDict_DelItem")]
    pub fn PyDict_DelItem(mp: *mut PyObject, key: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyDict_Clear")]
    pub fn PyDict_Clear(mp: *mut PyObject);
    #[cfg_attr(PyPy, link_name = "PyPyDict_Next")]
    pub fn PyDict_Next(
        mp: *mut PyObject,
        pos: *mut Py_ssize_t,
        key: *mut *mut PyObject,
        value: *mut *mut PyObject,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyDict_Keys")]
    pub fn PyDict_Keys(mp: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDict_Values")]
    pub fn PyDict_Values(mp: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDict_Items")]
    pub fn PyDict_Items(mp: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDict_Size")]
    pub fn PyDict_Size(mp: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyDict_Copy")]
    pub fn PyDict_Copy(mp: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDict_Contains")]
    pub fn PyDict_Contains(mp: *mut PyObject, key: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyDict_Update")]
    pub fn PyDict_Update(mp: *mut PyObject, other: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyDict_Merge")]
    pub fn PyDict_Merge(mp: *mut PyObject, other: *mut PyObject, _override: c_int) -> c_int;
    pub fn PyDict_MergeFromSeq2(d: *mut PyObject, seq2: *mut PyObject, _override: c_int) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyDict_GetItemString")]
    pub fn PyDict_GetItemString(dp: *mut PyObject, key: *const c_char) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDict_SetItemString")]
    pub fn PyDict_SetItemString(
        dp: *mut PyObject,
        key: *const c_char,
        item: *mut PyObject,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyDict_DelItemString")]
    pub fn PyDict_DelItemString(dp: *mut PyObject, key: *const c_char) -> c_int;
    #[cfg(Py_3_13)]
    #[cfg_attr(PyPy, link_name = "PyPyDict_GetItemRef")]
    pub fn PyDict_GetItemRef(
        dp: *mut PyObject,
        key: *mut PyObject,
        result: *mut *mut PyObject,
    ) -> c_int;
    // skipped 3.10 / ex-non-limited PyObject_GenericGetDict
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyDictKeys_Type: PyTypeObject;
    pub static mut PyDictValues_Type: PyTypeObject;
    pub static mut PyDictItems_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyDictKeys_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyDictKeys_Type)) as c_int
}

#[inline]
pub unsafe fn PyDictValues_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyDictValues_Type)) as c_int
}

#[inline]
pub unsafe fn PyDictItems_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyDictItems_Type)) as c_int
}

#[inline]
pub unsafe fn PyDictViewSet_Check(op: *mut PyObject) -> c_int {
    (PyDictKeys_Check(op) != 0 || PyDictItems_Check(op) != 0) as c_int
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyDictIterKey_Type: PyTypeObject;
    pub static mut PyDictIterValue_Type: PyTypeObject;
    pub static mut PyDictIterItem_Type: PyTypeObject;
    #[cfg(Py_3_8)]
    pub static mut PyDictRevIterKey_Type: PyTypeObject;
    #[cfg(Py_3_8)]
    pub static mut PyDictRevIterValue_Type: PyTypeObject;
    #[cfg(Py_3_8)]
    pub static mut PyDictRevIterItem_Type: PyTypeObject;
}

#[cfg(any(PyPy, GraalPy, Py_LIMITED_API))]
// TODO: remove (see https://github.com/PyO3/pyo3/pull/1341#issuecomment-751515985)
opaque_struct!(pub PyDictObject);
