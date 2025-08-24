#[cfg(not(all(Py_3_11, any(PyPy, GraalPy))))]
use libc::size_t;
use std::ffi::c_int;

#[cfg(not(any(PyPy, GraalPy)))]
use std::ffi::c_void;

use crate::object::*;

// skipped _PyObject_SIZE
// skipped _PyObject_VAR_SIZE

#[cfg(not(Py_3_11))]
extern "C" {
    pub fn _Py_GetAllocatedBlocks() -> crate::Py_ssize_t;
}

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyObjectArenaAllocator {
    pub ctx: *mut c_void,
    pub alloc: Option<extern "C" fn(ctx: *mut c_void, size: size_t) -> *mut c_void>,
    pub free: Option<extern "C" fn(ctx: *mut c_void, ptr: *mut c_void, size: size_t)>,
}

#[cfg(not(any(PyPy, GraalPy)))]
impl Default for PyObjectArenaAllocator {
    #[inline]
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

extern "C" {
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyObject_GetArenaAllocator(allocator: *mut PyObjectArenaAllocator);
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyObject_SetArenaAllocator(allocator: *mut PyObjectArenaAllocator);

    #[cfg(Py_3_9)]
    pub fn PyObject_IS_GC(o: *mut PyObject) -> c_int;
}

#[inline]
#[cfg(not(Py_3_9))]
pub unsafe fn PyObject_IS_GC(o: *mut PyObject) -> c_int {
    (crate::PyType_IS_GC(Py_TYPE(o)) != 0
        && match (*Py_TYPE(o)).tp_is_gc {
            Some(tp_is_gc) => tp_is_gc(o) != 0,
            None => true,
        }) as c_int
}

#[cfg(not(Py_3_11))]
extern "C" {
    pub fn _PyObject_GC_Malloc(size: size_t) -> *mut PyObject;
    pub fn _PyObject_GC_Calloc(size: size_t) -> *mut PyObject;
}

#[inline]
pub unsafe fn PyType_SUPPORTS_WEAKREFS(t: *mut PyTypeObject) -> c_int {
    ((*t).tp_weaklistoffset > 0) as c_int
}

#[inline]
pub unsafe fn PyObject_GET_WEAKREFS_LISTPTR(o: *mut PyObject) -> *mut *mut PyObject {
    let weaklistoffset = (*Py_TYPE(o)).tp_weaklistoffset;
    o.offset(weaklistoffset) as *mut *mut PyObject
}

// skipped PyUnstable_Object_GC_NewWithExtraData
