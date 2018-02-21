use libc::size_t;
use std::os::raw::{c_void, c_int};
use ffi3::pyport::Py_ssize_t;
use ffi3::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyObject_Malloc(size: size_t) -> *mut c_void;
    pub fn PyObject_Calloc(nelem: size_t, elsize: size_t) -> *mut c_void;
    pub fn PyObject_Realloc(ptr: *mut c_void, new_size: size_t) -> *mut c_void;
    pub fn PyObject_Free(ptr: *mut c_void) -> ();

    #[cfg(not(Py_LIMITED_API))]
    pub fn _Py_GetAllocatedBlocks() -> Py_ssize_t;
    pub fn PyObject_Init(arg1: *mut PyObject, arg2: *mut PyTypeObject) -> *mut PyObject;
    pub fn PyObject_InitVar(arg1: *mut PyVarObject, arg2: *mut PyTypeObject,
                            arg3: Py_ssize_t) -> *mut PyVarObject;
    pub fn _PyObject_New(arg1: *mut PyTypeObject) -> *mut PyObject;
    pub fn _PyObject_NewVar(arg1: *mut PyTypeObject, arg2: Py_ssize_t) -> *mut PyVarObject;

    pub fn PyGC_Collect() -> Py_ssize_t;
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(not(Py_LIMITED_API))]
pub struct PyObjectArenaAllocator {
    pub ctx: *mut c_void,
    pub alloc: Option<extern "C" fn(ctx: *mut c_void, size: size_t) -> *mut c_void>,
    pub free: Option<extern "C" fn(ctx: *mut c_void, ptr: *mut c_void, size: size_t) -> ()>,
}

#[cfg(not(Py_LIMITED_API))]
impl Default for PyObjectArenaAllocator {
    #[inline] fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyObject_GetArenaAllocator(allocator: *mut PyObjectArenaAllocator) -> ();
    pub fn PyObject_SetArenaAllocator(allocator: *mut PyObjectArenaAllocator) -> ();
}

/// Test if a type has a GC head
#[inline(always)]
#[allow(unused_parens)]
pub unsafe fn PyType_IS_GC(t : *mut PyTypeObject) -> c_int {
    PyType_HasFeature(t, Py_TPFLAGS_HAVE_GC)
}

/// Test if an object has a GC head
#[inline(always)]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn PyObject_IS_GC(o : *mut PyObject) -> c_int {
    (PyType_IS_GC(Py_TYPE(o)) != 0 &&
    match (*Py_TYPE(o)).tp_is_gc {
        Some(tp_is_gc) => tp_is_gc(o) != 0,
        None => true
    }) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn _PyObject_GC_Resize(arg1: *mut PyVarObject, arg2: Py_ssize_t) -> *mut PyVarObject;

    #[cfg(not(Py_LIMITED_API))]
    pub fn _PyObject_GC_Malloc(size: size_t) -> *mut PyObject;
    #[cfg(not(Py_LIMITED_API))]
    pub fn _PyObject_GC_Calloc(size: size_t) -> *mut PyObject;
    pub fn _PyObject_GC_New(arg1: *mut PyTypeObject) -> *mut PyObject;
    pub fn _PyObject_GC_NewVar(arg1: *mut PyTypeObject, arg2: Py_ssize_t) -> *mut PyVarObject;
    pub fn PyObject_GC_Track(arg1: *mut c_void) -> ();
    pub fn PyObject_GC_UnTrack(arg1: *mut c_void) -> ();
    pub fn PyObject_GC_Del(arg1: *mut c_void) -> ();
}

/// Test if a type supports weak references
#[inline(always)]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn PyType_SUPPORTS_WEAKREFS(t : *mut PyTypeObject) -> c_int {
    ((*t).tp_weaklistoffset > 0) as c_int
}

#[inline(always)]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn PyObject_GET_WEAKREFS_LISTPTR(o : *mut PyObject) -> *mut *mut PyObject {
    let weaklistoffset = (*Py_TYPE(o)).tp_weaklistoffset as isize;
    (o as *mut u8).offset(weaklistoffset) as *mut *mut PyObject
}
