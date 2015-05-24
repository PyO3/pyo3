use std::ptr;
use libc::{c_void, c_int, c_uint, c_ulong, c_char};
use pyport::{Py_ssize_t, Py_hash_t};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
}

#[cfg(py_sys_config="Py_TRACE_REFS")]
pub const PyObject_HEAD_INIT: PyObject = PyObject {
    _ob_next: 0 as *mut PyObject,
    _ob_prev: 0 as *mut PyObject,
    ob_refcnt: 1,
    ob_type: 0 as *mut PyTypeObject
};

#[cfg(not(py_sys_config="Py_TRACE_REFS"))]
pub const PyObject_HEAD_INIT: PyObject = PyObject {
    ob_refcnt: 1,
    ob_type: 0 as *mut PyTypeObject
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyVarObject {
    pub ob_base: PyObject,
    pub ob_size: Py_ssize_t,
}

#[inline(always)]
pub unsafe fn Py_REFCNT(ob : *mut PyObject) -> Py_ssize_t {
    (*ob).ob_refcnt
}

#[inline(always)]
pub unsafe fn Py_TYPE(ob : *mut PyObject) -> *mut PyTypeObject {
    (*ob).ob_type
}

#[inline(always)]
pub unsafe fn Py_SIZE(ob : *mut PyObject) -> Py_ssize_t {
    (*(ob as *mut PyVarObject)).ob_size
}

pub type unaryfunc =
    unsafe extern "C" fn(arg1: *mut PyObject)
                              -> *mut PyObject;
pub type binaryfunc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut PyObject)
                              -> *mut PyObject;
pub type ternaryfunc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut PyObject,
                               arg3: *mut PyObject) -> *mut PyObject;
pub type inquiry =
    unsafe extern "C" fn(arg1: *mut PyObject)
                              -> c_int;
pub type lenfunc =
    unsafe extern "C" fn(arg1: *mut PyObject) -> Py_ssize_t;
pub type ssizeargfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: Py_ssize_t)
                              -> *mut PyObject;
pub type ssizessizeargfunc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: Py_ssize_t,
                               arg3: Py_ssize_t) -> *mut PyObject;
pub type ssizeobjargproc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: Py_ssize_t,
                               arg3: *mut PyObject) -> c_int;
pub type ssizessizeobjargproc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: Py_ssize_t,
                               arg3: Py_ssize_t, arg4: *mut PyObject)
                              -> c_int;
pub type objobjargproc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut PyObject,
                               arg3: *mut PyObject) -> c_int;
pub type objobjproc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut PyObject)
                              -> c_int;
pub type visitproc =
    unsafe extern "C" fn
                              (object: *mut PyObject, arg: *mut c_void)
                              -> c_int;
pub type traverseproc =
    unsafe extern "C" fn
                              (slf: *mut PyObject, visit: visitproc,
                               arg: *mut c_void) -> c_int;
pub type freefunc =
    unsafe extern "C" fn(arg1: *mut c_void);
pub type destructor =
    unsafe extern "C" fn(arg1: *mut PyObject);
pub type getattrfunc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut c_char)
                              -> *mut PyObject;
pub type getattrofunc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut PyObject)
                              -> *mut PyObject;
pub type setattrfunc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut c_char,
                               arg3: *mut PyObject) -> c_int;
pub type setattrofunc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut PyObject,
                               arg3: *mut PyObject) -> c_int;
pub type reprfunc =
    unsafe extern "C" fn(arg1: *mut PyObject)
                              -> *mut PyObject;
pub type hashfunc =
    unsafe extern "C" fn(arg1: *mut PyObject)
                              -> Py_hash_t;
pub type richcmpfunc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut PyObject,
                               arg3: c_int) -> *mut PyObject;
pub type getiterfunc =
    unsafe extern "C" fn(arg1: *mut PyObject)
                              -> *mut PyObject;
pub type iternextfunc =
    unsafe extern "C" fn(arg1: *mut PyObject)
                              -> *mut PyObject;
pub type descrgetfunc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut PyObject,
                               arg3: *mut PyObject) -> *mut PyObject;
pub type descrsetfunc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut PyObject,
                               arg3: *mut PyObject) -> c_int;
pub type initproc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut PyObject,
                               arg3: *mut PyObject) -> c_int;
pub type newfunc =
    unsafe extern "C" fn
                              (arg1: *mut PyTypeObject,
                               arg2: *mut PyObject, arg3: *mut PyObject)
                              -> *mut PyObject;
pub type allocfunc =
    unsafe extern "C" fn
                              (arg1: *mut PyTypeObject,
                               arg2: Py_ssize_t) -> *mut PyObject;

pub enum PyTypeObject { }

#[repr(C)]
#[derive(Copy)]
pub struct PyType_Slot {
    pub slot: c_int,
    pub pfunc: *mut c_void,
}
impl Clone for PyType_Slot {
    fn clone(&self) -> PyType_Slot { *self }
}
impl ::std::default::Default for PyType_Slot {
    fn default() -> PyType_Slot { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy)]
pub struct PyType_Spec {
    pub name: *const c_char,
    pub basicsize: c_int,
    pub itemsize: c_int,
    pub flags: c_uint,
    pub slots: *mut PyType_Slot,
}
impl Clone for PyType_Spec {
    fn clone(&self) -> PyType_Spec { *self }
}
impl ::std::default::Default for PyType_Spec {
    fn default() -> PyType_Spec { unsafe { ::std::mem::zeroed() } }
}

extern "C" {
    pub fn PyType_FromSpec(arg1: *mut PyType_Spec) -> *mut PyObject;

    #[cfg(feature = "python3_3")]
    pub fn PyType_FromSpecWithBases(arg1: *mut PyType_Spec, arg2: *mut PyObject)
        -> *mut PyObject;

    #[cfg(feature = "python3_4")]
    pub fn PyType_GetSlot(arg1: *mut PyTypeObject, arg2: c_int)
        -> *mut c_void;
}

extern "C" {
    pub fn PyType_IsSubtype(a: *mut PyTypeObject, b: *mut PyTypeObject) -> c_int;
}

#[inline(always)]
pub unsafe fn PyObject_TypeCheck(ob: *mut PyObject, tp: *mut PyTypeObject) -> c_int {
    (Py_TYPE(ob) == tp || PyType_IsSubtype(Py_TYPE(ob), tp) != 0) as c_int
}

extern "C" {
    /// built-in 'type'
    pub static mut PyType_Type: PyTypeObject;
    /// built-in 'object'
    pub static mut PyBaseObject_Type: PyTypeObject;
    /// built-in 'super'
    pub static mut PySuper_Type: PyTypeObject;
    
    pub fn PyType_GetFlags(arg1: *mut PyTypeObject) -> c_ulong;
}

#[inline(always)]
pub unsafe fn PyType_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_TYPE_SUBCLASS)
}

#[inline(always)]
pub unsafe fn PyType_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyType_Type) as c_int
}

extern "C" {
    pub fn PyType_Ready(t: *mut PyTypeObject) -> c_int;
    pub fn PyType_GenericAlloc(t: *mut PyTypeObject, nitems: Py_ssize_t)
     -> *mut PyObject;
    pub fn PyType_GenericNew(t: *mut PyTypeObject, args: *mut PyObject,
                             kwds: *mut PyObject) -> *mut PyObject;
    pub fn PyType_ClearCache() -> c_uint;
    pub fn PyType_Modified(t: *mut PyTypeObject);
    
    pub fn PyObject_Repr(o: *mut PyObject) -> *mut PyObject;
    pub fn PyObject_Str(o: *mut PyObject) -> *mut PyObject;
    pub fn PyObject_ASCII(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyObject_Bytes(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyObject_RichCompare(arg1: *mut PyObject, arg2: *mut PyObject,
                                arg3: c_int) -> *mut PyObject;
    pub fn PyObject_RichCompareBool(arg1: *mut PyObject, arg2: *mut PyObject,
                                    arg3: c_int) -> c_int;
    pub fn PyObject_GetAttrString(arg1: *mut PyObject,
                                  arg2: *const c_char)
     -> *mut PyObject;
    pub fn PyObject_SetAttrString(arg1: *mut PyObject,
                                  arg2: *const c_char,
                                  arg3: *mut PyObject) -> c_int;
    pub fn PyObject_HasAttrString(arg1: *mut PyObject,
                                  arg2: *const c_char)
     -> c_int;
    pub fn PyObject_GetAttr(arg1: *mut PyObject, arg2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyObject_SetAttr(arg1: *mut PyObject, arg2: *mut PyObject,
                            arg3: *mut PyObject) -> c_int;
    pub fn PyObject_HasAttr(arg1: *mut PyObject, arg2: *mut PyObject)
     -> c_int;
    /*pub fn _PyObject_IsAbstract(arg1: *mut PyObject) -> c_int;
    pub fn _PyObject_GetAttrId(arg1: *mut PyObject,
                               arg2: *mut Struct__Py_Identifier)
     -> *mut PyObject;
    pub fn _PyObject_SetAttrId(arg1: *mut PyObject,
                               arg2: *mut Struct__Py_Identifier,
                               arg3: *mut PyObject) -> c_int;
    pub fn _PyObject_HasAttrId(arg1: *mut PyObject,
                               arg2: *mut Struct__Py_Identifier)
     -> c_int;*/
    pub fn PyObject_SelfIter(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyObject_GenericGetAttr(arg1: *mut PyObject, arg2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyObject_GenericSetAttr(arg1: *mut PyObject, arg2: *mut PyObject,
                                   arg3: *mut PyObject) -> c_int;
    pub fn PyObject_GenericSetDict(arg1: *mut PyObject, arg2: *mut PyObject,
                                   arg3: *mut c_void)
     -> c_int;
    pub fn PyObject_Hash(arg1: *mut PyObject) -> Py_hash_t;
    pub fn PyObject_HashNotImplemented(arg1: *mut PyObject) -> Py_hash_t;
    pub fn PyObject_IsTrue(arg1: *mut PyObject) -> c_int;
    pub fn PyObject_Not(arg1: *mut PyObject) -> c_int;
    pub fn PyCallable_Check(arg1: *mut PyObject) -> c_int;
    pub fn PyObject_ClearWeakRefs(arg1: *mut PyObject) -> ();

    pub fn PyObject_Dir(arg1: *mut PyObject) -> *mut PyObject;
    pub fn Py_ReprEnter(arg1: *mut PyObject) -> c_int;
    pub fn Py_ReprLeave(arg1: *mut PyObject) -> ();
}

// Flag bits for printing:
pub const Py_PRINT_RAW : c_int = 1;       // No string quotes etc.

/// Set if the type object is dynamically allocated
pub const Py_TPFLAGS_HEAPTYPE : c_ulong = (1<<9);

/// Set if the type allows subclassing
pub const Py_TPFLAGS_BASETYPE : c_ulong = (1<<10);

/// Set if the type is 'ready' -- fully initialized
pub const Py_TPFLAGS_READY : c_ulong = (1<<12);

/// Set while the type is being 'readied', to prevent recursive ready calls
pub const Py_TPFLAGS_READYING : c_ulong = (1<<13);

/// Objects support garbage collection (see objimp.h)
pub const Py_TPFLAGS_HAVE_GC : c_ulong = (1<<14);

const Py_TPFLAGS_HAVE_STACKLESS_EXTENSION : c_ulong = 0;

/// Objects support type attribute cache
pub const Py_TPFLAGS_HAVE_VERSION_TAG  : c_ulong = (1<<18);
pub const Py_TPFLAGS_VALID_VERSION_TAG : c_ulong = (1<<19);

/* Type is abstract and cannot be instantiated */
pub const Py_TPFLAGS_IS_ABSTRACT : c_ulong = (1<<20);

/* These flags are used to determine if a type is a subclass. */
pub const Py_TPFLAGS_LONG_SUBCLASS        : c_ulong = (1<<24);
pub const Py_TPFLAGS_LIST_SUBCLASS        : c_ulong = (1<<25);
pub const Py_TPFLAGS_TUPLE_SUBCLASS       : c_ulong = (1<<26);
pub const Py_TPFLAGS_BYTES_SUBCLASS       : c_ulong = (1<<27);
pub const Py_TPFLAGS_UNICODE_SUBCLASS     : c_ulong = (1<<28);
pub const Py_TPFLAGS_DICT_SUBCLASS        : c_ulong = (1<<29);
pub const Py_TPFLAGS_BASE_EXC_SUBCLASS    : c_ulong = (1<<30);
pub const Py_TPFLAGS_TYPE_SUBCLASS        : c_ulong = (1<<31);

pub const Py_TPFLAGS_DEFAULT : c_ulong = (
                 Py_TPFLAGS_HAVE_STACKLESS_EXTENSION |
                 Py_TPFLAGS_HAVE_VERSION_TAG |
                 0);

pub const Py_TPFLAGS_HAVE_FINALIZE        : c_ulong = (1<<0);

#[inline(always)]
pub unsafe fn PyType_HasFeature(t : *mut PyTypeObject, f : c_ulong) -> c_int {
    ((PyType_GetFlags(t) & f) != 0) as c_int
}

#[inline(always)]
pub unsafe fn PyType_FastSubclass(t : *mut PyTypeObject, f : c_ulong) -> c_int {
    PyType_HasFeature(t, f)
}

extern "C" {
    pub fn _Py_Dealloc(arg1: *mut PyObject) -> ();
}

// Reference counting macros.
#[inline(always)]
pub unsafe fn Py_INCREF(op : *mut PyObject) {
    if cfg!(py_sys_config="Py_REF_DEBUG") {
        Py_IncRef(op)
    } else {
        (*op).ob_refcnt += 1
    }
}

#[inline(always)]
pub unsafe fn Py_DECREF(op: *mut PyObject) {
    if cfg!(py_sys_config="Py_REF_DEBUG") {
        Py_DecRef(op)
    } else {
        (*op).ob_refcnt -= 1;
        if (*op).ob_refcnt == 0 {
            _Py_Dealloc(op)
        }
    }
}

#[inline(always)]
pub unsafe fn Py_CLEAR(op: &mut *mut PyObject) {
    let tmp = *op;
    if !tmp.is_null() {
        *op = ptr::null_mut();
        Py_DECREF(tmp);
    }
}

#[inline(always)]
pub unsafe fn Py_XINCREF(op : *mut PyObject) {
    if !op.is_null() {
        Py_INCREF(op)
    }
}

#[inline(always)]
pub unsafe fn Py_XDECREF(op : *mut PyObject) {
    if !op.is_null() {
        Py_DECREF(op)
    }
}

extern "C" {
    pub fn Py_IncRef(o: *mut PyObject);
    pub fn Py_DecRef(o: *mut PyObject);

    static mut _Py_NoneStruct: PyObject;
    static mut _Py_NotImplementedStruct: PyObject;
}

#[inline(always)]
pub unsafe fn Py_None() -> *mut PyObject {
    &mut _Py_NoneStruct
}

#[inline(always)]
pub unsafe fn Py_NotImplemented() -> *mut PyObject {
    &mut _Py_NotImplementedStruct
}

/* Rich comparison opcodes */
pub const Py_LT : c_int = 0;
pub const Py_LE : c_int = 1;
pub const Py_EQ : c_int = 2;
pub const Py_NE : c_int = 3;
pub const Py_GT : c_int = 4;
pub const Py_GE : c_int = 5;

