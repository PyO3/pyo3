use libc::c_int;
use pyport::Py_ssize_t;
use object::*;

extern "C" {
    pub static mut PyTuple_Type: PyTypeObject;
    pub static mut PyTupleIter_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyTuple_Check(op : *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_TUPLE_SUBCLASS)
}

#[inline(always)]
pub unsafe fn PyTuple_CheckExact(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyTuple_Type) as c_int
}

extern "C" {
    pub fn PyTuple_New(size: Py_ssize_t) -> *mut PyObject;
    pub fn PyTuple_Size(arg1: *mut PyObject) -> Py_ssize_t;
    pub fn PyTuple_GetItem(arg1: *mut PyObject, arg2: Py_ssize_t)
     -> *mut PyObject;
    pub fn PyTuple_SetItem(arg1: *mut PyObject, arg2: Py_ssize_t,
                           arg3: *mut PyObject) -> c_int;
    pub fn PyTuple_GetSlice(arg1: *mut PyObject, arg2: Py_ssize_t,
                            arg3: Py_ssize_t) -> *mut PyObject;
    pub fn PyTuple_Pack(arg1: Py_ssize_t, ...) -> *mut PyObject;
    pub fn PyTuple_ClearFreeList() -> c_int;
}

