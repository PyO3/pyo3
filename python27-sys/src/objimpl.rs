use libc::{c_void, c_char, c_int, size_t};
use pyport::Py_ssize_t;
use object::*;

#[link(name = "python2.7")]
extern "C" {
    pub fn PyObject_Malloc(arg1: size_t) -> *mut c_void;
    pub fn PyObject_Realloc(arg1: *mut c_void, arg2: size_t)
     -> *mut c_void;
    pub fn PyObject_Free(arg1: *mut c_void);

    pub fn PyObject_Init(arg1: *mut PyObject, arg2: *mut PyTypeObject)
     -> *mut PyObject;
    pub fn PyObject_InitVar(arg1: *mut PyVarObject, arg2: *mut PyTypeObject,
                            arg3: Py_ssize_t) -> *mut PyVarObject;
    pub fn _PyObject_New(arg1: *mut PyTypeObject) -> *mut PyObject;
    pub fn _PyObject_NewVar(arg1: *mut PyTypeObject, arg2: Py_ssize_t)
     -> *mut PyVarObject;
     
    // GC Support
    pub fn PyGC_Collect() -> Py_ssize_t;
    pub fn _PyObject_GC_Resize(arg1: *mut PyVarObject, arg2: Py_ssize_t)
     -> *mut PyVarObject;
    pub fn _PyObject_GC_Malloc(arg1: size_t) -> *mut PyObject;
    pub fn _PyObject_GC_New(arg1: *mut PyTypeObject) -> *mut PyObject;
    pub fn _PyObject_GC_NewVar(arg1: *mut PyTypeObject, arg2: Py_ssize_t)
     -> *mut PyVarObject;
    pub fn PyObject_GC_Track(arg1: *mut c_void);
    pub fn PyObject_GC_UnTrack(arg1: *mut c_void);
    pub fn PyObject_GC_Del(arg1: *mut c_void);
}

/// Test if a type has a GC head
#[inline(always)]
pub unsafe fn PyType_IS_GC(t : *mut PyTypeObject) -> c_int {
    PyType_HasFeature((t), Py_TPFLAGS_HAVE_GC)
}

/// Test if an object has a GC head
#[inline(always)]
pub unsafe fn PyObject_IS_GC(o : *mut PyObject) -> c_int {
    (PyType_IS_GC(Py_TYPE(o)) != 0 &&
    match (*Py_TYPE(o)).tp_is_gc {
        Some(tp_is_gc) => tp_is_gc(o) != 0,
        None => true
    }) as c_int
}

/* Test if a type supports weak references */
#[inline(always)]
pub unsafe fn PyType_SUPPORTS_WEAKREFS(t : *mut PyTypeObject) -> c_int {
    (PyType_HasFeature((t), Py_TPFLAGS_HAVE_WEAKREFS) != 0
     && ((*t).tp_weaklistoffset > 0)) as c_int
}

#[inline(always)]
pub unsafe fn PyObject_GET_WEAKREFS_LISTPTR(o : *mut PyObject) -> *mut *mut PyObject {
    let weaklistoffset = (*Py_TYPE(o)).tp_weaklistoffset as isize;
    (o as *mut c_char).offset(weaklistoffset) as *mut *mut PyObject
}

