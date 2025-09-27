use crate::pyport::{Py_hash_t, Py_ssize_t};
#[cfg(Py_GIL_DISABLED)]
use crate::refcount;
#[cfg(Py_GIL_DISABLED)]
use crate::PyMutex;
use std::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use std::mem;
use std::ptr;
#[cfg(Py_GIL_DISABLED)]
use std::sync::atomic::{AtomicIsize, AtomicU32};

#[cfg(Py_LIMITED_API)]
opaque_struct!(pub PyTypeObject);

#[cfg(not(Py_LIMITED_API))]
pub use crate::cpython::object::PyTypeObject;

// skip PyObject_HEAD

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(all(Py_3_14, not(Py_GIL_DISABLED), target_endian = "big"))]
/// This struct is anonymous in CPython, so the name was given by PyO3 because
/// Rust structs need a name.
pub struct PyObjectObFlagsAndRefcnt {
    pub ob_flags: u16,
    pub ob_overflow: u16,
    pub ob_refcnt: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(all(Py_3_14, not(Py_GIL_DISABLED), target_endian = "little"))]
/// This struct is anonymous in CPython, so the name was given by PyO3 because
/// Rust structs need a name.
pub struct PyObjectObFlagsAndRefcnt {
    pub ob_refcnt: u32,
    pub ob_overflow: u16,
    pub ob_flags: u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(all(Py_3_12, not(Py_GIL_DISABLED)))]
/// This union is anonymous in CPython, so the name was given by PyO3 because
/// Rust union need a name.
pub union PyObjectObRefcnt {
    #[cfg(all(target_pointer_width = "64", Py_3_14))]
    pub ob_refcnt_full: crate::PY_INT64_T,
    #[cfg(all(target_pointer_width = "64", Py_3_14))]
    pub refcnt_and_flags: PyObjectObFlagsAndRefcnt,
    pub ob_refcnt: Py_ssize_t,
    #[cfg(all(target_pointer_width = "64", not(Py_3_14)))]
    pub ob_refcnt_split: [crate::PY_UINT32_T; 2],
}

#[cfg(all(Py_3_12, not(Py_GIL_DISABLED)))]
impl std::fmt::Debug for PyObjectObRefcnt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { self.ob_refcnt })
    }
}

#[cfg(all(not(Py_3_12), not(Py_GIL_DISABLED)))]
pub type PyObjectObRefcnt = Py_ssize_t;

// PyObject_HEAD_INIT comes before the PyObject definition in object.h
// but we put it after PyObject because HEAD_INIT uses PyObject

#[repr(C)]
#[derive(Debug)]
pub struct PyObject {
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    #[cfg(Py_GIL_DISABLED)]
    pub ob_tid: libc::uintptr_t,
    #[cfg(all(Py_GIL_DISABLED, not(Py_3_14)))]
    pub _padding: u16,
    #[cfg(all(Py_GIL_DISABLED, Py_3_14))]
    pub ob_flags: u16,
    #[cfg(Py_GIL_DISABLED)]
    pub ob_mutex: PyMutex, // per-object lock
    #[cfg(Py_GIL_DISABLED)]
    pub ob_gc_bits: u8, // gc-related state
    #[cfg(Py_GIL_DISABLED)]
    pub ob_ref_local: AtomicU32, // local reference count
    #[cfg(Py_GIL_DISABLED)]
    pub ob_ref_shared: AtomicIsize, // shared reference count
    #[cfg(not(Py_GIL_DISABLED))]
    pub ob_refcnt: PyObjectObRefcnt,
    #[cfg(PyPy)]
    pub ob_pypy_link: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
}

#[allow(clippy::declare_interior_mutable_const)]
pub const PyObject_HEAD_INIT: PyObject = PyObject {
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    _ob_next: std::ptr::null_mut(),
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    _ob_prev: std::ptr::null_mut(),
    #[cfg(Py_GIL_DISABLED)]
    ob_tid: 0,
    #[cfg(all(Py_GIL_DISABLED, Py_3_14))]
    ob_flags: 0,
    #[cfg(all(Py_GIL_DISABLED, not(Py_3_14)))]
    _padding: 0,
    #[cfg(Py_GIL_DISABLED)]
    ob_mutex: PyMutex::new(),
    #[cfg(Py_GIL_DISABLED)]
    ob_gc_bits: 0,
    #[cfg(Py_GIL_DISABLED)]
    ob_ref_local: AtomicU32::new(refcount::_Py_IMMORTAL_REFCNT_LOCAL),
    #[cfg(Py_GIL_DISABLED)]
    ob_ref_shared: AtomicIsize::new(0),
    #[cfg(all(not(Py_GIL_DISABLED), Py_3_12))]
    ob_refcnt: PyObjectObRefcnt { ob_refcnt: 1 },
    #[cfg(not(Py_3_12))]
    ob_refcnt: 1,
    #[cfg(PyPy)]
    ob_pypy_link: 0,
    ob_type: std::ptr::null_mut(),
};

// skipped _Py_UNOWNED_TID

// skipped _PyObject_CAST

#[repr(C)]
#[derive(Debug)]
pub struct PyVarObject {
    pub ob_base: PyObject,
    #[cfg(not(GraalPy))]
    pub ob_size: Py_ssize_t,
    // On GraalPy the field is physically there, but not always populated. We hide it to prevent accidental misuse
    #[cfg(GraalPy)]
    pub _ob_size_graalpy: Py_ssize_t,
}

// skipped private _PyVarObject_CAST

#[inline]
#[cfg(not(any(GraalPy, all(PyPy, Py_3_10))))]
#[cfg_attr(docsrs, doc(cfg(all())))]
pub unsafe fn Py_Is(x: *mut PyObject, y: *mut PyObject) -> c_int {
    (x == y).into()
}

#[cfg(any(GraalPy, all(PyPy, Py_3_10)))]
#[cfg_attr(docsrs, doc(cfg(all())))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPy_Is")]
    pub fn Py_Is(x: *mut PyObject, y: *mut PyObject) -> c_int;
}

// skipped _Py_GetThreadLocal_Addr

// skipped _Py_ThreadID

// skipped _Py_IsOwnedByCurrentThread

#[cfg(GraalPy)]
extern "C" {
    #[cfg(GraalPy)]
    fn _Py_TYPE(arg1: *const PyObject) -> *mut PyTypeObject;

    #[cfg(GraalPy)]
    fn _Py_SIZE(arg1: *const PyObject) -> Py_ssize_t;
}

#[inline]
#[cfg(not(Py_3_14))]
pub unsafe fn Py_TYPE(ob: *mut PyObject) -> *mut PyTypeObject {
    #[cfg(not(GraalPy))]
    return (*ob).ob_type;
    #[cfg(GraalPy)]
    return _Py_TYPE(ob);
}

#[cfg_attr(windows, link(name = "pythonXY"))]
#[cfg(Py_3_14)]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPy_TYPE")]
    pub fn Py_TYPE(ob: *mut PyObject) -> *mut PyTypeObject;
}

// skip _Py_TYPE compat shim

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyLong_Type")]
    pub static mut PyLong_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyBool_Type")]
    pub static mut PyBool_Type: PyTypeObject;
}

#[inline]
pub unsafe fn Py_SIZE(ob: *mut PyObject) -> Py_ssize_t {
    #[cfg(not(GraalPy))]
    {
        debug_assert_ne!((*ob).ob_type, std::ptr::addr_of_mut!(crate::PyLong_Type));
        debug_assert_ne!((*ob).ob_type, std::ptr::addr_of_mut!(crate::PyBool_Type));
        (*ob.cast::<PyVarObject>()).ob_size
    }
    #[cfg(GraalPy)]
    _Py_SIZE(ob)
}

#[inline]
pub unsafe fn Py_IS_TYPE(ob: *mut PyObject, tp: *mut PyTypeObject) -> c_int {
    (Py_TYPE(ob) == tp) as c_int
}

// skipped Py_SET_TYPE

// skipped Py_SET_SIZE

pub type unaryfunc = unsafe extern "C" fn(*mut PyObject) -> *mut PyObject;
pub type binaryfunc = unsafe extern "C" fn(*mut PyObject, *mut PyObject) -> *mut PyObject;
pub type ternaryfunc =
    unsafe extern "C" fn(*mut PyObject, *mut PyObject, *mut PyObject) -> *mut PyObject;
pub type inquiry = unsafe extern "C" fn(*mut PyObject) -> c_int;
pub type lenfunc = unsafe extern "C" fn(*mut PyObject) -> Py_ssize_t;
pub type ssizeargfunc = unsafe extern "C" fn(*mut PyObject, Py_ssize_t) -> *mut PyObject;
pub type ssizessizeargfunc =
    unsafe extern "C" fn(*mut PyObject, Py_ssize_t, Py_ssize_t) -> *mut PyObject;
pub type ssizeobjargproc = unsafe extern "C" fn(*mut PyObject, Py_ssize_t, *mut PyObject) -> c_int;
pub type ssizessizeobjargproc =
    unsafe extern "C" fn(*mut PyObject, Py_ssize_t, Py_ssize_t, arg4: *mut PyObject) -> c_int;
pub type objobjargproc = unsafe extern "C" fn(*mut PyObject, *mut PyObject, *mut PyObject) -> c_int;

pub type objobjproc = unsafe extern "C" fn(*mut PyObject, *mut PyObject) -> c_int;
pub type visitproc = unsafe extern "C" fn(object: *mut PyObject, arg: *mut c_void) -> c_int;
pub type traverseproc =
    unsafe extern "C" fn(slf: *mut PyObject, visit: visitproc, arg: *mut c_void) -> c_int;

pub type freefunc = unsafe extern "C" fn(*mut c_void);
pub type destructor = unsafe extern "C" fn(*mut PyObject);
pub type getattrfunc = unsafe extern "C" fn(*mut PyObject, *mut c_char) -> *mut PyObject;
pub type getattrofunc = unsafe extern "C" fn(*mut PyObject, *mut PyObject) -> *mut PyObject;
pub type setattrfunc = unsafe extern "C" fn(*mut PyObject, *mut c_char, *mut PyObject) -> c_int;
pub type setattrofunc = unsafe extern "C" fn(*mut PyObject, *mut PyObject, *mut PyObject) -> c_int;
pub type reprfunc = unsafe extern "C" fn(*mut PyObject) -> *mut PyObject;
pub type hashfunc = unsafe extern "C" fn(*mut PyObject) -> Py_hash_t;
pub type richcmpfunc = unsafe extern "C" fn(*mut PyObject, *mut PyObject, c_int) -> *mut PyObject;
pub type getiterfunc = unsafe extern "C" fn(*mut PyObject) -> *mut PyObject;
pub type iternextfunc = unsafe extern "C" fn(*mut PyObject) -> *mut PyObject;
pub type descrgetfunc =
    unsafe extern "C" fn(*mut PyObject, *mut PyObject, *mut PyObject) -> *mut PyObject;
pub type descrsetfunc = unsafe extern "C" fn(*mut PyObject, *mut PyObject, *mut PyObject) -> c_int;
pub type initproc = unsafe extern "C" fn(*mut PyObject, *mut PyObject, *mut PyObject) -> c_int;
pub type newfunc =
    unsafe extern "C" fn(*mut PyTypeObject, *mut PyObject, *mut PyObject) -> *mut PyObject;
pub type allocfunc = unsafe extern "C" fn(*mut PyTypeObject, Py_ssize_t) -> *mut PyObject;

#[cfg(Py_3_8)]
pub type vectorcallfunc = unsafe extern "C" fn(
    callable: *mut PyObject,
    args: *const *mut PyObject,
    nargsf: libc::size_t,
    kwnames: *mut PyObject,
) -> *mut PyObject;

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

    #[cfg(Py_3_11)]
    #[cfg_attr(PyPy, link_name = "PyPyType_GetName")]
    pub fn PyType_GetName(arg1: *mut PyTypeObject) -> *mut PyObject;

    #[cfg(Py_3_11)]
    #[cfg_attr(PyPy, link_name = "PyPyType_GetQualName")]
    pub fn PyType_GetQualName(arg1: *mut PyTypeObject) -> *mut PyObject;

    #[cfg(Py_3_13)]
    #[cfg_attr(PyPy, link_name = "PyPyType_GetFullyQualifiedName")]
    pub fn PyType_GetFullyQualifiedName(arg1: *mut PyTypeObject) -> *mut PyObject;

    #[cfg(Py_3_13)]
    #[cfg_attr(PyPy, link_name = "PyPyType_GetModuleName")]
    pub fn PyType_GetModuleName(arg1: *mut PyTypeObject) -> *mut PyObject;

    #[cfg(Py_3_12)]
    #[cfg_attr(PyPy, link_name = "PyPyType_FromMetaclass")]
    pub fn PyType_FromMetaclass(
        metaclass: *mut PyTypeObject,
        module: *mut PyObject,
        spec: *mut PyType_Spec,
        bases: *mut PyObject,
    ) -> *mut PyObject;

    #[cfg(Py_3_12)]
    #[cfg_attr(PyPy, link_name = "PyPyObject_GetTypeData")]
    pub fn PyObject_GetTypeData(obj: *mut PyObject, cls: *mut PyTypeObject) -> *mut c_void;

    #[cfg(Py_3_12)]
    #[cfg_attr(PyPy, link_name = "PyPyObject_GetTypeDataSize")]
    pub fn PyObject_GetTypeDataSize(cls: *mut PyTypeObject) -> Py_ssize_t;

    #[cfg_attr(PyPy, link_name = "PyPyType_IsSubtype")]
    pub fn PyType_IsSubtype(a: *mut PyTypeObject, b: *mut PyTypeObject) -> c_int;
}

#[inline]
pub unsafe fn PyObject_TypeCheck(ob: *mut PyObject, tp: *mut PyTypeObject) -> c_int {
    (Py_IS_TYPE(ob, tp) != 0 || PyType_IsSubtype(Py_TYPE(ob), tp) != 0) as c_int
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
    #[cfg(any(Py_3_13, all(PyPy, not(Py_3_11))))] // CPython defined in 3.12 as an inline function in abstract.h
    #[cfg_attr(PyPy, link_name = "PyPyObject_DelAttrString")]
    pub fn PyObject_DelAttrString(arg1: *mut PyObject, arg2: *const c_char) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_HasAttrString")]
    pub fn PyObject_HasAttrString(arg1: *mut PyObject, arg2: *const c_char) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_GetAttr")]
    pub fn PyObject_GetAttr(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;
    #[cfg(Py_3_13)]
    #[cfg_attr(PyPy, link_name = "PyPyObject_GetOptionalAttr")]
    pub fn PyObject_GetOptionalAttr(
        arg1: *mut PyObject,
        arg2: *mut PyObject,
        arg3: *mut *mut PyObject,
    ) -> c_int;
    #[cfg(Py_3_13)]
    #[cfg_attr(PyPy, link_name = "PyPyObject_GetOptionalAttrString")]
    pub fn PyObject_GetOptionalAttrString(
        arg1: *mut PyObject,
        arg2: *const c_char,
        arg3: *mut *mut PyObject,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_SetAttr")]
    pub fn PyObject_SetAttr(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject)
        -> c_int;
    #[cfg(any(Py_3_13, all(PyPy, not(Py_3_11))))] // CPython defined in 3.12 as an inline function in abstract.h
    #[cfg_attr(PyPy, link_name = "PyPyObject_DelAttr")]
    pub fn PyObject_DelAttr(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyObject_HasAttr")]
    pub fn PyObject_HasAttr(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int;
    #[cfg(Py_3_13)]
    #[cfg_attr(PyPy, link_name = "PyPyObject_HasAttrWithError")]
    pub fn PyObject_HasAttrWithError(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int;
    #[cfg(Py_3_13)]
    #[cfg_attr(PyPy, link_name = "PyPyObject_HasAttrStringWithError")]
    pub fn PyObject_HasAttrStringWithError(arg1: *mut PyObject, arg2: *const c_char) -> c_int;
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
    #[cfg_attr(PyPy, link_name = "PyPyObject_GenericGetDict")]
    pub fn PyObject_GenericGetDict(arg1: *mut PyObject, arg2: *mut c_void) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_GenericSetDict")]
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

// skipped because is a private API
// const _Py_TPFLAGS_STATIC_BUILTIN: c_ulong = 1 << 1;

#[cfg(all(Py_3_12, not(Py_LIMITED_API)))]
pub const Py_TPFLAGS_MANAGED_WEAKREF: c_ulong = 1 << 3;

#[cfg(all(Py_3_11, not(Py_LIMITED_API)))]
pub const Py_TPFLAGS_MANAGED_DICT: c_ulong = 1 << 4;

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
#[cfg(any(Py_3_12, all(Py_3_8, not(Py_LIMITED_API))))]
pub const Py_TPFLAGS_HAVE_VECTORCALL: c_ulong = 1 << 11;
// skipped backwards-compatibility alias _Py_TPFLAGS_HAVE_VECTORCALL

/// Set if the type is 'ready' -- fully initialized
pub const Py_TPFLAGS_READY: c_ulong = 1 << 12;

/// Set while the type is being 'readied', to prevent recursive ready calls
pub const Py_TPFLAGS_READYING: c_ulong = 1 << 13;

/// Objects support garbage collection (see objimp.h)
pub const Py_TPFLAGS_HAVE_GC: c_ulong = 1 << 14;

const Py_TPFLAGS_HAVE_STACKLESS_EXTENSION: c_ulong = 0;

#[cfg(Py_3_8)]
pub const Py_TPFLAGS_METHOD_DESCRIPTOR: c_ulong = 1 << 17;

pub const Py_TPFLAGS_VALID_VERSION_TAG: c_ulong = 1 << 19;

/* Type is abstract and cannot be instantiated */
pub const Py_TPFLAGS_IS_ABSTRACT: c_ulong = 1 << 20;

// skipped non-limited / 3.10 Py_TPFLAGS_HAVE_AM_SEND
#[cfg(Py_3_12)]
pub const Py_TPFLAGS_ITEMS_AT_END: c_ulong = 1 << 23;

/* These flags are used to determine if a type is a subclass. */
pub const Py_TPFLAGS_LONG_SUBCLASS: c_ulong = 1 << 24;
pub const Py_TPFLAGS_LIST_SUBCLASS: c_ulong = 1 << 25;
pub const Py_TPFLAGS_TUPLE_SUBCLASS: c_ulong = 1 << 26;
pub const Py_TPFLAGS_BYTES_SUBCLASS: c_ulong = 1 << 27;
pub const Py_TPFLAGS_UNICODE_SUBCLASS: c_ulong = 1 << 28;
pub const Py_TPFLAGS_DICT_SUBCLASS: c_ulong = 1 << 29;
pub const Py_TPFLAGS_BASE_EXC_SUBCLASS: c_ulong = 1 << 30;
pub const Py_TPFLAGS_TYPE_SUBCLASS: c_ulong = 1 << 31;

pub const Py_TPFLAGS_DEFAULT: c_ulong = if cfg!(Py_3_10) {
    Py_TPFLAGS_HAVE_STACKLESS_EXTENSION
} else {
    Py_TPFLAGS_HAVE_STACKLESS_EXTENSION | Py_TPFLAGS_HAVE_VERSION_TAG
};

pub const Py_TPFLAGS_HAVE_FINALIZE: c_ulong = 1;
pub const Py_TPFLAGS_HAVE_VERSION_TAG: c_ulong = 1 << 18;

#[cfg(Py_3_13)]
pub const Py_CONSTANT_NONE: c_uint = 0;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_FALSE: c_uint = 1;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_TRUE: c_uint = 2;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_ELLIPSIS: c_uint = 3;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_NOT_IMPLEMENTED: c_uint = 4;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_ZERO: c_uint = 5;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_ONE: c_uint = 6;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_EMPTY_STR: c_uint = 7;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_EMPTY_BYTES: c_uint = 8;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_EMPTY_TUPLE: c_uint = 9;

extern "C" {
    #[cfg(Py_3_13)]
    #[cfg_attr(PyPy, link_name = "PyPy_GetConstant")]
    pub fn Py_GetConstant(constant_id: c_uint) -> *mut PyObject;
    #[cfg(Py_3_13)]
    #[cfg_attr(PyPy, link_name = "PyPy_GetConstantBorrowed")]
    pub fn Py_GetConstantBorrowed(constant_id: c_uint) -> *mut PyObject;
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg(all(not(GraalPy), not(all(Py_3_13, Py_LIMITED_API))))]
    #[cfg_attr(PyPy, link_name = "_PyPy_NoneStruct")]
    static mut _Py_NoneStruct: PyObject;

    #[cfg(GraalPy)]
    static mut _Py_NoneStructReference: *mut PyObject;
}

#[inline]
pub unsafe fn Py_None() -> *mut PyObject {
    #[cfg(all(not(GraalPy), all(Py_3_13, Py_LIMITED_API)))]
    return Py_GetConstantBorrowed(Py_CONSTANT_NONE);

    #[cfg(all(not(GraalPy), not(all(Py_3_13, Py_LIMITED_API))))]
    return ptr::addr_of_mut!(_Py_NoneStruct);

    #[cfg(GraalPy)]
    return _Py_NoneStructReference;
}

#[inline]
pub unsafe fn Py_IsNone(x: *mut PyObject) -> c_int {
    Py_Is(x, Py_None())
}

// skipped Py_RETURN_NONE

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg(all(not(GraalPy), not(all(Py_3_13, Py_LIMITED_API))))]
    #[cfg_attr(PyPy, link_name = "_PyPy_NotImplementedStruct")]
    static mut _Py_NotImplementedStruct: PyObject;

    #[cfg(GraalPy)]
    static mut _Py_NotImplementedStructReference: *mut PyObject;
}

#[inline]
pub unsafe fn Py_NotImplemented() -> *mut PyObject {
    #[cfg(all(not(GraalPy), all(Py_3_13, Py_LIMITED_API)))]
    return Py_GetConstantBorrowed(Py_CONSTANT_NOT_IMPLEMENTED);

    #[cfg(all(not(GraalPy), not(all(Py_3_13, Py_LIMITED_API))))]
    return ptr::addr_of_mut!(_Py_NotImplementedStruct);

    #[cfg(GraalPy)]
    return _Py_NotImplementedStructReference;
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
pub unsafe fn PyType_HasFeature(ty: *mut PyTypeObject, feature: c_ulong) -> c_int {
    #[cfg(Py_LIMITED_API)]
    let flags = PyType_GetFlags(ty);

    #[cfg(all(not(Py_LIMITED_API), Py_GIL_DISABLED))]
    let flags = (*ty).tp_flags.load(std::sync::atomic::Ordering::Relaxed);

    #[cfg(all(not(Py_LIMITED_API), not(Py_GIL_DISABLED)))]
    let flags = (*ty).tp_flags;

    ((flags & feature) != 0) as c_int
}

#[inline]
pub unsafe fn PyType_FastSubclass(t: *mut PyTypeObject, f: c_ulong) -> c_int {
    PyType_HasFeature(t, f)
}

#[inline]
pub unsafe fn PyType_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_TYPE_SUBCLASS)
}

// skipped _PyType_CAST

#[inline]
pub unsafe fn PyType_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, ptr::addr_of_mut!(PyType_Type))
}

extern "C" {
    #[cfg(any(Py_3_13, all(Py_3_11, not(Py_LIMITED_API))))]
    #[cfg_attr(PyPy, link_name = "PyPyType_GetModuleByDef")]
    pub fn PyType_GetModuleByDef(
        arg1: *mut crate::PyTypeObject,
        arg2: *mut crate::PyModuleDef,
    ) -> *mut PyObject;

    #[cfg(Py_3_14)]
    pub fn PyType_Freeze(tp: *mut crate::PyTypeObject) -> c_int;
}
