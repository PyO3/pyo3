use std::os::raw::c_int;
use ffi2::pyport::Py_ssize_t;
use ffi2::object::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyTupleObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub ob_size: Py_ssize_t,
    pub ob_item: [*mut PyObject; 1],
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyTuple_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyTuple_Check(op : *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_TUPLE_SUBCLASS)
}

#[inline(always)]
pub unsafe fn PyTuple_CheckExact(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyTuple_Type;
    (Py_TYPE(op) == u) as c_int
}


// Macro, trading safety for speed
#[inline(always)]
pub unsafe fn PyTuple_GET_ITEM(op: *mut PyObject, i: Py_ssize_t) -> *mut PyObject {
   *(*(op as *mut PyTupleObject)).ob_item.as_ptr().offset(i as isize)
}

#[inline(always)]
pub unsafe fn PyTuple_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    Py_SIZE(op)
}

/// Macro, *only* to be used to fill in brand new tuples
#[inline(always)]
pub unsafe fn PyTuple_SET_ITEM(op: *mut PyObject, i: Py_ssize_t, v: *mut PyObject) {
   *(*(op as *mut PyTupleObject)).ob_item.as_mut_ptr().offset(i as isize) = v;
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyTuple_New(size: Py_ssize_t) -> *mut PyObject;
    pub fn PyTuple_Size(p: *mut PyObject) -> Py_ssize_t;
    pub fn PyTuple_GetItem(p: *mut PyObject, pos: Py_ssize_t) -> *mut PyObject;
    pub fn PyTuple_SetItem(p: *mut PyObject, pos: Py_ssize_t,
                           o: *mut PyObject) -> c_int;
    pub fn PyTuple_GetSlice(p: *mut PyObject, low: Py_ssize_t,
                            high: Py_ssize_t) -> *mut PyObject;
    pub fn _PyTuple_Resize(p: *mut *mut PyObject, newsize: Py_ssize_t) -> c_int;
    pub fn PyTuple_Pack(n: Py_ssize_t, ...) -> *mut PyObject;
    //pub fn _PyTuple_MaybeUntrack(arg1: *mut PyObject);
    pub fn PyTuple_ClearFreeList() -> c_int;
}
