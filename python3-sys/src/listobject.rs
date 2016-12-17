use libc::c_int;
use pyport::Py_ssize_t;
use object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyList_Type: PyTypeObject;
    pub static mut PyListIter_Type: PyTypeObject;
    pub static mut PyListRevIter_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyList_Check(op : *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_LIST_SUBCLASS)
}

#[inline(always)]
pub unsafe fn PyList_CheckExact(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyList_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyList_New(size: Py_ssize_t) -> *mut PyObject;
    pub fn PyList_Size(arg1: *mut PyObject) -> Py_ssize_t;
    pub fn PyList_GetItem(arg1: *mut PyObject, arg2: Py_ssize_t)
     -> *mut PyObject;
    pub fn PyList_SetItem(arg1: *mut PyObject, arg2: Py_ssize_t,
                          arg3: *mut PyObject) -> c_int;
    pub fn PyList_Insert(arg1: *mut PyObject, arg2: Py_ssize_t,
                         arg3: *mut PyObject) -> c_int;
    pub fn PyList_Append(arg1: *mut PyObject, arg2: *mut PyObject)
     -> c_int;
    pub fn PyList_GetSlice(arg1: *mut PyObject, arg2: Py_ssize_t,
                           arg3: Py_ssize_t) -> *mut PyObject;
    pub fn PyList_SetSlice(arg1: *mut PyObject, arg2: Py_ssize_t,
                           arg3: Py_ssize_t, arg4: *mut PyObject)
     -> c_int;
    pub fn PyList_Sort(arg1: *mut PyObject) -> c_int;
    pub fn PyList_Reverse(arg1: *mut PyObject) -> c_int;
    pub fn PyList_AsTuple(arg1: *mut PyObject) -> *mut PyObject;
}

