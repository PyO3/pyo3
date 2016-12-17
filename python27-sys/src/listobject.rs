use libc::c_int;
use pyport::Py_ssize_t;
use object::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyListObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub ob_size: Py_ssize_t,
    pub ob_item: *mut *mut PyObject,
    pub allocated: Py_ssize_t,
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyList_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyList_Check(op : *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_LIST_SUBCLASS)
}

#[inline(always)]
pub unsafe fn PyList_CheckExact(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyList_Type;
    (Py_TYPE(op) == u) as c_int
}


// Macro, trading safety for speed
#[inline(always)]
pub unsafe fn PyList_GET_ITEM(op: *mut PyObject, i: Py_ssize_t) -> *mut PyObject {
   *(*(op as *mut PyListObject)).ob_item.offset(i as isize)
}

#[inline(always)]
pub unsafe fn PyList_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    Py_SIZE(op)
}

/// Macro, *only* to be used to fill in brand new lists
#[inline(always)]
pub unsafe fn PyList_SET_ITEM(op: *mut PyObject, i: Py_ssize_t, v: *mut PyObject) {
   *(*(op as *mut PyListObject)).ob_item.offset(i as isize) = v;
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyList_New(size: Py_ssize_t) -> *mut PyObject;
    pub fn PyList_Size(list: *mut PyObject) -> Py_ssize_t;
    pub fn PyList_GetItem(list: *mut PyObject, index: Py_ssize_t)
     -> *mut PyObject;
    pub fn PyList_SetItem(list: *mut PyObject, index: Py_ssize_t,
                          item: *mut PyObject) -> c_int;
    pub fn PyList_Insert(list: *mut PyObject, index: Py_ssize_t,
                         item: *mut PyObject) -> c_int;
    pub fn PyList_Append(list: *mut PyObject, item: *mut PyObject)
     -> c_int;
    pub fn PyList_GetSlice(list: *mut PyObject, low: Py_ssize_t,
                           high: Py_ssize_t) -> *mut PyObject;
    pub fn PyList_SetSlice(list: *mut PyObject, low: Py_ssize_t,
                           high: Py_ssize_t, itemlist: *mut PyObject)
     -> c_int;
    pub fn PyList_Sort(list: *mut PyObject) -> c_int;
    pub fn PyList_Reverse(list: *mut PyObject) -> c_int;
    pub fn PyList_AsTuple(list: *mut PyObject) -> *mut PyObject;
    //fn _PyList_Extend(arg1: *mut PyListObject, arg2: *mut PyObject)
    //-> *mut PyObject;
}

