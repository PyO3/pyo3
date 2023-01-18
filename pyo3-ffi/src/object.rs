// FFI note: this file changed a lot between 3.6 and 3.10.
// Some missing definitions may not be marked "skipped".
use crate::pyport::{Py_hash_t, Py_ssize_t};
use std::mem;
use std::os::raw::{c_char, c_int, c_uint, c_ulong, c_void};
use std::ptr;

#[cfg(Py_LIMITED_API)]
opaque_struct!(PyTypeObject);

#[cfg(not(Py_LIMITED_API))]
pub use crate::cpython::object::PyTypeObject;

// _PyObject_HEAD_EXTRA: conditionally defined in PyObject_HEAD_INIT
// _PyObject_EXTRA_INIT: conditionally defined in PyObject_HEAD_INIT

pub const PyObject_HEAD_INIT: PyObject = PyObject {
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    _ob_next: std::ptr::null_mut(),
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    _ob_prev: std::ptr::null_mut(),
    #[cfg(Py_NOGIL)]
    ob_tid: 0,
    #[cfg(Py_NOGIL)]
    ob_reflocal: 1,
    #[cfg(Py_NOGIL)]
    ob_ref_shared: 0,
    #[cfg(not(Py_NOGIL))]
    ob_refcnt: 1,
    #[cfg(PyPy)]
    ob_pypy_link: 0,
    ob_type: std::ptr::null_mut(),
};

// skipped PyObject_VAR_HEAD
// skipped Py_INVALID_SIZE

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PyObject {
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    #[cfg(Py_NOGIL)]
    pub ob_tid: usize,
    #[cfg(Py_NOGIL)]
    pub ob_reflocal: u32,
    #[cfg(Py_NOGIL)]
    pub ob_ref_shared: u32,
    #[cfg(not(Py_NOGIL))]
    pub ob_refcnt: Py_ssize_t,
    #[cfg(PyPy)]
    pub ob_pypy_link: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
}

// skipped _PyObject_CAST
// skipped _PyObject_CAST_CONST

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyVarObject {
    pub ob_base: PyObject,
    pub ob_size: Py_ssize_t,
}

// skipped _PyVarObject_CAST
// skipped _PyVarObject_CAST_CONST

#[inline]
pub unsafe fn Py_Is(x: *mut PyObject, y: *mut PyObject) -> c_int {
    (x == y).into()
}

// skipped _Py_REFCNT: defined in Py_REFCNT

#[inline]
#[cfg(not(Py_NOGIL))]
pub unsafe fn Py_REFCNT(ob: *mut PyObject) -> Py_ssize_t {
    assert!(!ob.is_null());
    (*ob).ob_refcnt
}

#[inline]
#[cfg(Py_NOGIL)]
pub unsafe fn Py_REFCNT(ob: *mut PyObject) -> Py_ssize_t {
    assert!(!ob.is_null());
    Py_RefCnt(ob)
}

#[inline]
pub unsafe fn Py_TYPE(ob: *mut PyObject) -> *mut PyTypeObject {
    (*ob).ob_type
}

#[inline]
pub unsafe fn Py_SIZE(ob: *mut PyObject) -> Py_ssize_t {
    (*(ob as *mut PyVarObject)).ob_size
}

#[inline]
pub unsafe fn Py_IS_TYPE(ob: *mut PyObject, tp: *mut PyTypeObject) -> c_int {
    (Py_TYPE(ob) == tp) as c_int
}

// skipped _Py_SET_REFCNT
// skipped Py_SET_REFCNT
// skipped _Py_SET_TYPE
// skipped Py_SET_TYPE
// skipped _Py_SET_SIZE
// skipped Py_SET_SIZE

pub type unaryfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> *mut PyObject;

pub type binaryfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;

pub type ternaryfunc = unsafe extern "C" fn(
    arg1: *mut PyObject,
    arg2: *mut PyObject,
    arg3: *mut PyObject,
) -> *mut PyObject;

pub type inquiry = unsafe extern "C" fn(arg1: *mut PyObject) -> c_int;

pub type lenfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> Py_ssize_t;

pub type ssizeargfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject;

pub type ssizessizeargfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: Py_ssize_t, arg3: Py_ssize_t) -> *mut PyObject;

pub type ssizeobjargproc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: Py_ssize_t, arg3: *mut PyObject) -> c_int;

pub type ssizessizeobjargproc = unsafe extern "C" fn(
    arg1: *mut PyObject,
    arg2: Py_ssize_t,
    arg3: Py_ssize_t,
    arg4: *mut PyObject,
) -> c_int;

pub type objobjargproc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject) -> c_int;

pub type objobjproc = unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int;
pub type visitproc = unsafe extern "C" fn(object: *mut PyObject, arg: *mut c_void) -> c_int;
pub type traverseproc =
    unsafe extern "C" fn(slf: *mut PyObject, visit: visitproc, arg: *mut c_void) -> c_int;

pub type freefunc = unsafe extern "C" fn(arg1: *mut c_void);
pub type destructor = unsafe extern "C" fn(arg1: *mut PyObject);
pub type getattrfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut c_char) -> *mut PyObject;
pub type getattrofunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;
pub type setattrfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut c_char, arg3: *mut PyObject) -> c_int;
pub type setattrofunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject) -> c_int;
pub type reprfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> *mut PyObject;
pub type hashfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> Py_hash_t;
pub type richcmpfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject, arg3: c_int) -> *mut PyObject;
pub type getiterfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> *mut PyObject;
pub type iternextfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> *mut PyObject;
pub type descrgetfunc = unsafe extern "C" fn(
    arg1: *mut PyObject,
    arg2: *mut PyObject,
    arg3: *mut PyObject,
) -> *mut PyObject;
pub type descrsetfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject) -> c_int;
pub type initproc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject) -> c_int;
pub type newfunc = unsafe extern "C" fn(
    arg1: *mut PyTypeObject,
    arg2: *mut PyObject,
    arg3: *mut PyObject,
) -> *mut PyObject;
pub type allocfunc =
    unsafe extern "C" fn(arg1: *mut PyTypeObject, arg2: Py_ssize_t) -> *mut PyObject;
#[cfg(Py_3_11)]
pub type getbufferproc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut crate::Py_buffer, arg3: c_int) -> c_int;
#[cfg(Py_3_11)]
pub type releasebufferproc = unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut crate::Py_buffer);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyType_Slot {
    pub slot: c_int,
    pub pfunc: *mut c_void,
}

impl Default for PyType_Slot {
    fn default() -> PyType_Slot {
        unsafe { mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyType_Spec {
    pub name: *const c_char,
    pub basicsize: c_int,
    pub itemsize: c_int,
    pub flags: c_uint,
    pub slots: *mut PyType_Slot,
}

impl Default for PyType_Spec {
    fn default() -> PyType_Spec {
        unsafe { mem::zeroed() }
    }
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyType_FromSpec")]
    pub fn PyType_FromSpec(arg1: *mut PyType_Spec) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyType_FromSpecWithBases")]
    pub fn PyType_FromSpecWithBases(arg1: *mut PyType_Spec, arg2: *mut PyObject) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyType_GetSlot")]
    pub fn PyType_GetSlot(arg1: *mut PyTypeObject, arg2: c_int) -> *mut c_void;

    #[cfg(any(Py_3_10, all(Py_3_9, not(Py_LIMITED_API))))]
    #[cfg_attr(PyPy, link_name = "PyPyType_FromModuleAndSpec")]
    pub fn PyType_FromModuleAndSpec(
        module: *mut PyObject,
        spec: *mut PyType_Spec,
        bases: *mut PyObject,
    ) -> *mut PyObject;

    #[cfg(any(Py_3_10, all(Py_3_9, not(Py_LIMITED_API))))]
    #[cfg_attr(PyPy, link_name = "PyPyType_GetModule")]
    pub fn PyType_GetModule(arg1: *mut PyTypeObject) -> *mut PyObject;

    #[cfg(any(Py_3_10, all(Py_3_9, not(Py_LIMITED_API))))]
    #[cfg_attr(PyPy, link_name = "PyPyType_GetModuleState")]
    pub fn PyType_GetModuleState(arg1: *mut PyTypeObject) -> *mut c_void;
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyType_IsSubtype")]
    pub fn PyType_IsSubtype(a: *mut PyTypeObject, b: *mut PyTypeObject) -> c_int;
}

#[inline]
pub unsafe fn PyObject_TypeCheck(ob: *mut PyObject, tp: *mut PyTypeObject) -> c_int {
    (Py_TYPE(ob) == tp || PyType_IsSubtype(Py_TYPE(ob), tp) != 0) as c_int
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    /// built-in 'type'
    #[cfg_attr(PyPy, link_name = "PyPyType_Type")]
    pub static mut PyType_Type: PyTypeObject;
    /// built-in 'object'
    #[cfg_attr(PyPy, link_name = "PyPyBaseObject_Type")]
    pub static mut PyBaseObject_Type: PyTypeObject;
    /// built-in 'super'
    pub static mut PySuper_Type: PyTypeObject;
}

extern "C" {
    pub fn PyType_GetFlags(arg1: *mut PyTypeObject) -> c_ulong;
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyType_Ready")]
    pub fn PyType_Ready(t: *mut PyTypeObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyType_GenericAlloc")]
    pub fn PyType_GenericAlloc(t: *mut PyTypeObject, nitems: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyType_GenericNew")]
    pub fn PyType_GenericNew(
        t: *mut PyTypeObject,
        args: *mut PyObject,
        kwds: *mut PyObject,
    ) -> *mut PyObject;
    pub fn PyType_ClearCache() -> c_uint;
    #[cfg_attr(PyPy, link_name = "PyPyType_Modified")]
    pub fn PyType_Modified(t: *mut PyTypeObject);

    #[cfg_attr(PyPy, link_name = "PyPyObject_Repr")]
    pub fn PyObject_Repr(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Str")]
    pub fn PyObject_Str(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_ASCII")]
    pub fn PyObject_ASCII(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Bytes")]
    pub fn PyObject_Bytes(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_RichCompare")]
    pub fn PyObject_RichCompare(
        arg1: *mut PyObject,
        arg2: *mut PyObject,
        arg3: c_int,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_RichCompareBool")]
    pub fn PyObject_RichCompareBool(arg1: *mut PyObject, arg2: *mut PyObject, arg3: c_int)
        -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_GetAttrString")]
    pub fn PyObject_GetAttrString(arg1: *mut PyObject, arg2: *const c_char) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_SetAttrString")]
    pub fn PyObject_SetAttrString(
        arg1: *mut PyObject,
        arg2: *const c_char,
        arg3: *mut PyObject,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_HasAttrString")]
    pub fn PyObject_HasAttrString(arg1: *mut PyObject, arg2: *const c_char) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_GetAttr")]
    pub fn PyObject_GetAttr(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_SetAttr")]
    pub fn PyObject_SetAttr(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject)
        -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_HasAttr")]
    pub fn PyObject_HasAttr(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_SelfIter")]
    pub fn PyObject_SelfIter(arg1: *mut PyObject) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyObject_GenericGetAttr")]
    pub fn PyObject_GenericGetAttr(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_GenericSetAttr")]
    pub fn PyObject_GenericSetAttr(
        arg1: *mut PyObject,
        arg2: *mut PyObject,
        arg3: *mut PyObject,
    ) -> c_int;
    #[cfg(not(all(Py_LIMITED_API, not(Py_3_10))))]
    pub fn PyObject_GenericGetDict(arg1: *mut PyObject, arg2: *mut c_void) -> *mut PyObject;
    pub fn PyObject_GenericSetDict(
        arg1: *mut PyObject,
        arg2: *mut PyObject,
        arg3: *mut c_void,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Hash")]
    pub fn PyObject_Hash(arg1: *mut PyObject) -> Py_hash_t;
    #[cfg_attr(PyPy, link_name = "PyPyObject_HashNotImplemented")]
    pub fn PyObject_HashNotImplemented(arg1: *mut PyObject) -> Py_hash_t;
    #[cfg_attr(PyPy, link_name = "PyPyObject_IsTrue")]
    pub fn PyObject_IsTrue(arg1: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Not")]
    pub fn PyObject_Not(arg1: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyCallable_Check")]
    pub fn PyCallable_Check(arg1: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_ClearWeakRefs")]
    pub fn PyObject_ClearWeakRefs(arg1: *mut PyObject);

    #[cfg_attr(PyPy, link_name = "PyPyObject_Dir")]
    pub fn PyObject_Dir(arg1: *mut PyObject) -> *mut PyObject;
    pub fn Py_ReprEnter(arg1: *mut PyObject) -> c_int;
    pub fn Py_ReprLeave(arg1: *mut PyObject);
}

// Flag bits for printing:
pub const Py_PRINT_RAW: c_int = 1; // No string quotes etc.

#[cfg(all(Py_3_10, not(Py_LIMITED_API)))]
pub const Py_TPFLAGS_SEQUENCE: c_ulong = 1 << 5;

#[cfg(all(Py_3_10, not(Py_LIMITED_API)))]
pub const Py_TPFLAGS_MAPPING: c_ulong = 1 << 6;

#[cfg(Py_3_10)]
pub const Py_TPFLAGS_DISALLOW_INSTANTIATION: c_ulong = 1 << 7;

#[cfg(Py_3_10)]
pub const Py_TPFLAGS_IMMUTABLETYPE: c_ulong = 1 << 8;

/// Set if the type object is dynamically allocated
pub const Py_TPFLAGS_HEAPTYPE: c_ulong = 1 << 9;

/// Set if the type allows subclassing
pub const Py_TPFLAGS_BASETYPE: c_ulong = 1 << 10;

/// Set if the type implements the vectorcall protocol (PEP 590)
#[cfg(all(Py_3_8, not(Py_LIMITED_API)))]
pub const Py_TPFLAGS_HAVE_VECTORCALL: c_ulong = 1 << 11;
// skipped non-limited _Py_TPFLAGS_HAVE_VECTORCALL

/// Set if the type is 'ready' -- fully initialized
pub const Py_TPFLAGS_READY: c_ulong = 1 << 12;

/// Set while the type is being 'readied', to prevent recursive ready calls
pub const Py_TPFLAGS_READYING: c_ulong = 1 << 13;

/// Objects support garbage collection (see objimp.h)
pub const Py_TPFLAGS_HAVE_GC: c_ulong = 1 << 14;

const Py_TPFLAGS_HAVE_STACKLESS_EXTENSION: c_ulong = 0;

#[cfg(Py_3_8)]
pub const Py_TPFLAGS_METHOD_DESCRIPTOR: c_ulong = 1 << 17;

/// This flag does nothing in Python 3.10+
pub const Py_TPFLAGS_HAVE_VERSION_TAG: c_ulong = 1 << 18;

pub const Py_TPFLAGS_VALID_VERSION_TAG: c_ulong = 1 << 19;

/* Type is abstract and cannot be instantiated */
pub const Py_TPFLAGS_IS_ABSTRACT: c_ulong = 1 << 20;

// skipped non-limited / 3.10 Py_TPFLAGS_HAVE_AM_SEND

/* These flags are used to determine if a type is a subclass. */
pub const Py_TPFLAGS_LONG_SUBCLASS: c_ulong = 1 << 24;
pub const Py_TPFLAGS_LIST_SUBCLASS: c_ulong = 1 << 25;
pub const Py_TPFLAGS_TUPLE_SUBCLASS: c_ulong = 1 << 26;
pub const Py_TPFLAGS_BYTES_SUBCLASS: c_ulong = 1 << 27;
pub const Py_TPFLAGS_UNICODE_SUBCLASS: c_ulong = 1 << 28;
pub const Py_TPFLAGS_DICT_SUBCLASS: c_ulong = 1 << 29;
pub const Py_TPFLAGS_BASE_EXC_SUBCLASS: c_ulong = 1 << 30;
pub const Py_TPFLAGS_TYPE_SUBCLASS: c_ulong = 1 << 31;

pub const Py_TPFLAGS_DEFAULT: c_ulong =
    Py_TPFLAGS_HAVE_STACKLESS_EXTENSION | Py_TPFLAGS_HAVE_VERSION_TAG;

pub const Py_TPFLAGS_HAVE_FINALIZE: c_ulong = 1;

// skipped _Py_RefTotal
// skipped _Py_NegativeRefCount

extern "C" {
    #[cfg_attr(PyPy, link_name = "_PyPy_Dealloc")]
    pub fn _Py_Dealloc(arg1: *mut PyObject);
}

// Reference counting macros.
#[inline]
#[cfg(not(any(py_sys_config = "Py_REF_DEBUG", Py_NOGIL)))]
pub unsafe fn Py_INCREF(op: *mut PyObject) {
    (*op).ob_refcnt += 1
}

#[inline]
#[cfg(any(py_sys_config = "Py_REF_DEBUG", Py_NOGIL))]
pub unsafe fn Py_INCREF(op: *mut PyObject) {
    Py_IncRef(op)
}

#[inline]
#[cfg(not(any(py_sys_config = "Py_REF_DEBUG", Py_NOGIL)))]
pub unsafe fn Py_DECREF(op: *mut PyObject) {
    (*op).ob_refcnt -= 1;
    if (*op).ob_refcnt == 0 {
        _Py_Dealloc(op)
    }
}

#[inline]
#[cfg(any(py_sys_config = "Py_REF_DEBUG", Py_NOGIL))]
pub unsafe fn Py_DECREF(op: *mut PyObject) {
    Py_DecRef(op)
}

#[inline]
pub unsafe fn Py_CLEAR(op: *mut *mut PyObject) {
    let tmp = *op;
    if !tmp.is_null() {
        *op = ptr::null_mut();
        Py_DECREF(tmp);
    }
}

#[inline]
pub unsafe fn Py_XINCREF(op: *mut PyObject) {
    if !op.is_null() {
        Py_INCREF(op)
    }
}

#[inline]
pub unsafe fn Py_XDECREF(op: *mut PyObject) {
    if !op.is_null() {
        Py_DECREF(op)
    }
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPy_IncRef")]
    pub fn Py_IncRef(o: *mut PyObject);
    #[cfg_attr(PyPy, link_name = "PyPy_DecRef")]
    pub fn Py_DecRef(o: *mut PyObject);
    #[cfg(Py_NOGIL)]
    pub fn Py_RefCnt(o: *mut PyObject) -> Py_ssize_t;

    #[cfg(Py_3_10)]
    pub fn Py_NewRef(obj: *mut PyObject) -> *mut PyObject;
    #[cfg(Py_3_10)]
    pub fn Py_XNewRef(obj: *mut PyObject) -> *mut PyObject;
}

// Technically these macros are only available in the C header from 3.10 and up, however their
// implementation works on all supported Python versions so we define these macros on all
// versions for simplicity.

#[inline]
pub unsafe fn _Py_NewRef(obj: *mut PyObject) -> *mut PyObject {
    Py_INCREF(obj);
    obj
}

#[inline]
pub unsafe fn _Py_XNewRef(obj: *mut PyObject) -> *mut PyObject {
    Py_XINCREF(obj);
    obj
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "_PyPy_NoneStruct")]
    static mut _Py_NoneStruct: PyObject;
}

#[inline]
pub unsafe fn Py_None() -> *mut PyObject {
    addr_of_mut_shim!(_Py_NoneStruct)
}

#[inline]
pub unsafe fn Py_IsNone(x: *mut PyObject) -> c_int {
    Py_Is(x, Py_None())
}

// skipped Py_RETURN_NONE

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "_PyPy_NotImplementedStruct")]
    static mut _Py_NotImplementedStruct: PyObject;
}

#[inline]
pub unsafe fn Py_NotImplemented() -> *mut PyObject {
    addr_of_mut_shim!(_Py_NotImplementedStruct)
}

// skipped Py_RETURN_NOTIMPLEMENTED

/* Rich comparison opcodes */
pub const Py_LT: c_int = 0;
pub const Py_LE: c_int = 1;
pub const Py_EQ: c_int = 2;
pub const Py_NE: c_int = 3;
pub const Py_GT: c_int = 4;
pub const Py_GE: c_int = 5;

#[cfg(Py_3_10)]
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PySendResult {
    PYGEN_RETURN = 0,
    PYGEN_ERROR = -1,
    PYGEN_NEXT = 1,
}

// skipped Py_RETURN_RICHCOMPARE

#[inline]
#[cfg(Py_LIMITED_API)]
pub unsafe fn PyType_HasFeature(t: *mut PyTypeObject, f: c_ulong) -> c_int {
    ((PyType_GetFlags(t) & f) != 0) as c_int
}

#[inline]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn PyType_HasFeature(t: *mut PyTypeObject, f: c_ulong) -> c_int {
    (((*t).tp_flags & f) != 0) as c_int
}

#[inline]
pub unsafe fn PyType_FastSubclass(t: *mut PyTypeObject, f: c_ulong) -> c_int {
    PyType_HasFeature(t, f)
}

#[inline]
pub unsafe fn PyType_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_TYPE_SUBCLASS)
}

#[inline]
pub unsafe fn PyType_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut_shim!(PyType_Type)) as c_int
}
