use crate::object::*;
use crate::pyport::Py_ssize_t;
use libc::size_t;
use std::os::raw::{c_int, c_void};

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyObject_Malloc")]
    pub fn PyObject_Malloc(size: size_t) -> *mut c_void;
    pub fn PyObject_Calloc(nelem: size_t, elsize: size_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Realloc")]
    pub fn PyObject_Realloc(ptr: *mut c_void, new_size: size_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Free")]
    pub fn PyObject_Free(ptr: *mut c_void);

    #[cfg(not(Py_LIMITED_API))]
    pub fn _Py_GetAllocatedBlocks() -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Init")]
    pub fn PyObject_Init(arg1: *mut PyObject, arg2: *mut PyTypeObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_InitVar")]
    pub fn PyObject_InitVar(
        arg1: *mut PyVarObject,
        arg2: *mut PyTypeObject,
        arg3: Py_ssize_t,
    ) -> *mut PyVarObject;
    #[cfg_attr(PyPy, link_name = "_PyPyObject_New")]
    pub fn _PyObject_New(arg1: *mut PyTypeObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "_PyPyObject_NewVar")]
    pub fn _PyObject_NewVar(arg1: *mut PyTypeObject, arg2: Py_ssize_t) -> *mut PyVarObject;

    pub fn PyGC_Collect() -> Py_ssize_t;

    #[cfg(Py_3_10)]
    pub fn PyGC_Enable() -> c_int;

    #[cfg(Py_3_10)]
    pub fn PyGC_Disable() -> c_int;

    #[cfg(Py_3_10)]
    pub fn PyGC_IsEnabled() -> c_int;
}

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyObjectArenaAllocator {
    pub ctx: *mut c_void,
    pub alloc: Option<extern "C" fn(ctx: *mut c_void, size: size_t) -> *mut c_void>,
    pub free: Option<extern "C" fn(ctx: *mut c_void, ptr: *mut c_void, size: size_t)>,
}

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
impl Default for PyObjectArenaAllocator {
    #[inline]
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

extern "C" {
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    pub fn PyObject_GetArenaAllocator(allocator: *mut PyObjectArenaAllocator);
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    pub fn PyObject_SetArenaAllocator(allocator: *mut PyObjectArenaAllocator);
}

/// Test if a type has a GC head
#[inline]
pub unsafe fn PyType_IS_GC(t: *mut PyTypeObject) -> c_int {
    PyType_HasFeature(t, Py_TPFLAGS_HAVE_GC)
}

/// Test if an object has a GC head
#[inline]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn PyObject_IS_GC(o: *mut PyObject) -> c_int {
    (PyType_IS_GC(Py_TYPE(o)) != 0
        && match (*Py_TYPE(o)).tp_is_gc {
            Some(tp_is_gc) => tp_is_gc(o) != 0,
            None => true,
        }) as c_int
}

extern "C" {
    pub fn _PyObject_GC_Resize(arg1: *mut PyVarObject, arg2: Py_ssize_t) -> *mut PyVarObject;

    #[cfg(not(Py_LIMITED_API))]
    pub fn _PyObject_GC_Malloc(size: size_t) -> *mut PyObject;
    #[cfg(not(Py_LIMITED_API))]
    pub fn _PyObject_GC_Calloc(size: size_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "_PyPyObject_GC_New")]
    pub fn _PyObject_GC_New(arg1: *mut PyTypeObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "_PyPyObject_GC_NewVar")]
    pub fn _PyObject_GC_NewVar(arg1: *mut PyTypeObject, arg2: Py_ssize_t) -> *mut PyVarObject;
    pub fn PyObject_GC_Track(arg1: *mut c_void);
    pub fn PyObject_GC_UnTrack(arg1: *mut c_void);
    #[cfg_attr(PyPy, link_name = "PyPyObject_GC_Del")]
    pub fn PyObject_GC_Del(arg1: *mut c_void);
}

/// Test if a type supports weak references
#[inline]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn PyType_SUPPORTS_WEAKREFS(t: *mut PyTypeObject) -> c_int {
    ((*t).tp_weaklistoffset > 0) as c_int
}

#[inline]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn PyObject_GET_WEAKREFS_LISTPTR(o: *mut PyObject) -> *mut *mut PyObject {
    let weaklistoffset = (*Py_TYPE(o)).tp_weaklistoffset as isize;
    o.offset(weaklistoffset) as *mut *mut PyObject
}
