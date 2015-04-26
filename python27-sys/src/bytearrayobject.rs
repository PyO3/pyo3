use libc::{c_char, c_int};
use pyport::Py_ssize_t;
use object::*;

/*#[repr(C)]
#[deriving(Copy)]
struct PyByteArrayObject {
    #[cfg(feature="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(feature="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub ob_size: Py_ssize_t,
    pub ob_exports: c_int,
    pub ob_alloc: Py_ssize_t,
    pub ob_bytes: *mut c_char,
}*/

#[link(name = "python2.7")]
extern "C" {
    pub static mut PyByteArray_Type: PyTypeObject;
    pub static mut PyByteArrayIter_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyByteArray_Check(op : *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyByteArray_Type)
}

#[inline(always)]
pub unsafe fn PyByteArray_CheckExact(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyByteArray_Type;
    (Py_TYPE(op) == u) as c_int
}

#[link(name = "python2.7")]
extern "C" {
    pub fn PyByteArray_FromObject(o: *mut PyObject) -> *mut PyObject;
    pub fn PyByteArray_Concat(a: *mut PyObject, b: *mut PyObject)
     -> *mut PyObject;
    pub fn PyByteArray_FromStringAndSize(string: *const c_char,
                                         len: Py_ssize_t) -> *mut PyObject;
    pub fn PyByteArray_Size(bytearray: *mut PyObject) -> Py_ssize_t;
    pub fn PyByteArray_AsString(bytearray: *mut PyObject) -> *mut c_char;
    pub fn PyByteArray_Resize(bytearray: *mut PyObject, len: Py_ssize_t)
     -> c_int;
}

#[inline(always)]
pub unsafe fn PyByteArray_AS_STRING(o: *mut PyObject) -> *mut c_char {
    PyByteArray_AsString(o)
    // #define PyByteArray_AS_STRING(self) \
    //   (assert(PyByteArray_Check(self)), \
    //    Py_SIZE(self) ? ((PyByteArrayObject *)(self))->ob_bytes : _PyByteArray_empty_string)
}

#[inline(always)]
pub unsafe fn PyByteArray_GET_SIZE(o: *mut PyObject) -> Py_ssize_t {
    // #define PyByteArray_GET_SIZE(self)  (assert(PyByteArray_Check(self)),Py_SIZE(self))
    PyByteArray_Size(o)
}

