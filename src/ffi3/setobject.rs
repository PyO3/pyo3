use std::os::raw::c_int;
use ffi3::pyport::Py_ssize_t;
use ffi3::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PySet_Type: PyTypeObject;
    pub static mut PyFrozenSet_Type: PyTypeObject;
    pub static mut PySetIter_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyFrozenSet_CheckExact(ob : *mut PyObject) -> c_int {
    (Py_TYPE(ob) == &mut PyFrozenSet_Type) as c_int
}

#[inline]
pub unsafe fn PyAnySet_CheckExact(ob : *mut PyObject) -> c_int {
    (Py_TYPE(ob) == &mut PySet_Type || Py_TYPE(ob) == &mut PyFrozenSet_Type) as c_int
}

#[inline]
pub unsafe fn PyAnySet_Check(ob : *mut PyObject) -> c_int {
    (PyAnySet_CheckExact(ob) != 0 ||
      PyType_IsSubtype(Py_TYPE(ob), &mut PySet_Type) != 0 ||
      PyType_IsSubtype(Py_TYPE(ob), &mut PyFrozenSet_Type) != 0) as c_int
}

#[inline]
pub unsafe fn PySet_Check(ob : *mut PyObject) -> c_int {
    (Py_TYPE(ob) == &mut PySet_Type || PyType_IsSubtype(Py_TYPE(ob), &mut PySet_Type) != 0) as c_int
}

#[inline]
pub unsafe fn PyFrozenSet_Check(ob : *mut PyObject) -> c_int {
    (Py_TYPE(ob) == &mut PyFrozenSet_Type || PyType_IsSubtype(Py_TYPE(ob), &mut PyFrozenSet_Type) != 0) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PySet_New(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyFrozenSet_New(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PySet_Size(anyset: *mut PyObject) -> Py_ssize_t;
    pub fn PySet_Clear(set: *mut PyObject) -> c_int;
    pub fn PySet_Contains(anyset: *mut PyObject, key: *mut PyObject)
     -> c_int;
    pub fn PySet_Discard(set: *mut PyObject, key: *mut PyObject)
     -> c_int;
    pub fn PySet_Add(set: *mut PyObject, key: *mut PyObject) -> c_int;
    pub fn PySet_Pop(set: *mut PyObject) -> *mut PyObject;
}

