#[cfg(Py_3_8)]
use crate::vectorcallfunc;
use crate::{object, PyGetSetDef, PyMemberDef, PyMethodDef, PyObject, Py_ssize_t};
use std::ffi::{c_char, c_int, c_uint, c_void};
use std::mem;

// skipped private _Py_NewReference
// skipped private _Py_NewReferenceNoTotal
// skipped private _Py_ResurrectReference

// skipped private _Py_GetGlobalRefTotal
// skipped private _Py_GetRefTotal
// skipped private _Py_GetLegacyRefTotal
// skipped private _PyInterpreterState_GetRefTotal

// skipped private _Py_Identifier

// skipped private _Py_static_string_init
// skipped private _Py_static_string
// skipped private _Py_IDENTIFIER

#[cfg(not(Py_3_11))] // moved to src/buffer.rs from Python
mod bufferinfo {
    use crate::Py_ssize_t;
    use std::ffi::{c_char, c_int, c_void};
    use std::ptr;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Py_buffer {
        pub buf: *mut c_void,
        /// Owned reference
        pub obj: *mut crate::PyObject,
        pub len: Py_ssize_t,
        pub itemsize: Py_ssize_t,
        pub readonly: c_int,
        pub ndim: c_int,
        pub format: *mut c_char,
        pub shape: *mut Py_ssize_t,
        pub strides: *mut Py_ssize_t,
        pub suboffsets: *mut Py_ssize_t,
        pub internal: *mut c_void,
        #[cfg(PyPy)]
        pub flags: c_int,
        #[cfg(PyPy)]
        pub _strides: [Py_ssize_t; PyBUF_MAX_NDIM as usize],
        #[cfg(PyPy)]
        pub _shape: [Py_ssize_t; PyBUF_MAX_NDIM as usize],
    }

    impl Py_buffer {
        #[allow(clippy::new_without_default)]
        pub const fn new() -> Self {
            Py_buffer {
                buf: ptr::null_mut(),
                obj: ptr::null_mut(),
                len: 0,
                itemsize: 0,
                readonly: 0,
                ndim: 0,
                format: ptr::null_mut(),
                shape: ptr::null_mut(),
                strides: ptr::null_mut(),
                suboffsets: ptr::null_mut(),
                internal: ptr::null_mut(),
                #[cfg(PyPy)]
                flags: 0,
                #[cfg(PyPy)]
                _strides: [0; PyBUF_MAX_NDIM as usize],
                #[cfg(PyPy)]
                _shape: [0; PyBUF_MAX_NDIM as usize],
            }
        }
    }

    pub type getbufferproc = unsafe extern "C" fn(
        arg1: *mut crate::PyObject,
        arg2: *mut Py_buffer,
        arg3: c_int,
    ) -> c_int;
    pub type releasebufferproc =
        unsafe extern "C" fn(arg1: *mut crate::PyObject, arg2: *mut Py_buffer);

    /// Maximum number of dimensions
    pub const PyBUF_MAX_NDIM: c_int = if cfg!(PyPy) { 36 } else { 64 };

    /* Flags for getting buffers */
    pub const PyBUF_SIMPLE: c_int = 0;
    pub const PyBUF_WRITABLE: c_int = 0x0001;
    /* we used to include an E, backwards compatible alias */
    pub const PyBUF_WRITEABLE: c_int = PyBUF_WRITABLE;
    pub const PyBUF_FORMAT: c_int = 0x0004;
    pub const PyBUF_ND: c_int = 0x0008;
    pub const PyBUF_STRIDES: c_int = 0x0010 | PyBUF_ND;
    pub const PyBUF_C_CONTIGUOUS: c_int = 0x0020 | PyBUF_STRIDES;
    pub const PyBUF_F_CONTIGUOUS: c_int = 0x0040 | PyBUF_STRIDES;
    pub const PyBUF_ANY_CONTIGUOUS: c_int = 0x0080 | PyBUF_STRIDES;
    pub const PyBUF_INDIRECT: c_int = 0x0100 | PyBUF_STRIDES;

    pub const PyBUF_CONTIG: c_int = PyBUF_ND | PyBUF_WRITABLE;
    pub const PyBUF_CONTIG_RO: c_int = PyBUF_ND;

    pub const PyBUF_STRIDED: c_int = PyBUF_STRIDES | PyBUF_WRITABLE;
    pub const PyBUF_STRIDED_RO: c_int = PyBUF_STRIDES;

    pub const PyBUF_RECORDS: c_int = PyBUF_STRIDES | PyBUF_WRITABLE | PyBUF_FORMAT;
    pub const PyBUF_RECORDS_RO: c_int = PyBUF_STRIDES | PyBUF_FORMAT;

    pub const PyBUF_FULL: c_int = PyBUF_INDIRECT | PyBUF_WRITABLE | PyBUF_FORMAT;
    pub const PyBUF_FULL_RO: c_int = PyBUF_INDIRECT | PyBUF_FORMAT;

    pub const PyBUF_READ: c_int = 0x100;
    pub const PyBUF_WRITE: c_int = 0x200;
}

#[cfg(not(Py_3_11))]
pub use self::bufferinfo::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyNumberMethods {
    pub nb_add: Option<object::binaryfunc>,
    pub nb_subtract: Option<object::binaryfunc>,
    pub nb_multiply: Option<object::binaryfunc>,
    pub nb_remainder: Option<object::binaryfunc>,
    pub nb_divmod: Option<object::binaryfunc>,
    pub nb_power: Option<object::ternaryfunc>,
    pub nb_negative: Option<object::unaryfunc>,
    pub nb_positive: Option<object::unaryfunc>,
    pub nb_absolute: Option<object::unaryfunc>,
    pub nb_bool: Option<object::inquiry>,
    pub nb_invert: Option<object::unaryfunc>,
    pub nb_lshift: Option<object::binaryfunc>,
    pub nb_rshift: Option<object::binaryfunc>,
    pub nb_and: Option<object::binaryfunc>,
    pub nb_xor: Option<object::binaryfunc>,
    pub nb_or: Option<object::binaryfunc>,
    pub nb_int: Option<object::unaryfunc>,
    pub nb_reserved: *mut c_void,
    pub nb_float: Option<object::unaryfunc>,
    pub nb_inplace_add: Option<object::binaryfunc>,
    pub nb_inplace_subtract: Option<object::binaryfunc>,
    pub nb_inplace_multiply: Option<object::binaryfunc>,
    pub nb_inplace_remainder: Option<object::binaryfunc>,
    pub nb_inplace_power: Option<object::ternaryfunc>,
    pub nb_inplace_lshift: Option<object::binaryfunc>,
    pub nb_inplace_rshift: Option<object::binaryfunc>,
    pub nb_inplace_and: Option<object::binaryfunc>,
    pub nb_inplace_xor: Option<object::binaryfunc>,
    pub nb_inplace_or: Option<object::binaryfunc>,
    pub nb_floor_divide: Option<object::binaryfunc>,
    pub nb_true_divide: Option<object::binaryfunc>,
    pub nb_inplace_floor_divide: Option<object::binaryfunc>,
    pub nb_inplace_true_divide: Option<object::binaryfunc>,
    pub nb_index: Option<object::unaryfunc>,
    pub nb_matrix_multiply: Option<object::binaryfunc>,
    pub nb_inplace_matrix_multiply: Option<object::binaryfunc>,
}

#[repr(C)]
#[derive(Clone)]
pub struct PySequenceMethods {
    pub sq_length: Option<object::lenfunc>,
    pub sq_concat: Option<object::binaryfunc>,
    pub sq_repeat: Option<object::ssizeargfunc>,
    pub sq_item: Option<object::ssizeargfunc>,
    pub was_sq_slice: *mut c_void,
    pub sq_ass_item: Option<object::ssizeobjargproc>,
    pub was_sq_ass_slice: *mut c_void,
    pub sq_contains: Option<object::objobjproc>,
    pub sq_inplace_concat: Option<object::binaryfunc>,
    pub sq_inplace_repeat: Option<object::ssizeargfunc>,
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct PyMappingMethods {
    pub mp_length: Option<object::lenfunc>,
    pub mp_subscript: Option<object::binaryfunc>,
    pub mp_ass_subscript: Option<object::objobjargproc>,
}

#[cfg(Py_3_10)]
pub type sendfunc = unsafe extern "C" fn(
    iter: *mut PyObject,
    value: *mut PyObject,
    result: *mut *mut PyObject,
) -> object::PySendResult;

#[repr(C)]
#[derive(Clone, Default)]
pub struct PyAsyncMethods {
    pub am_await: Option<object::unaryfunc>,
    pub am_aiter: Option<object::unaryfunc>,
    pub am_anext: Option<object::unaryfunc>,
    #[cfg(Py_3_10)]
    pub am_send: Option<sendfunc>,
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct PyBufferProcs {
    pub bf_getbuffer: Option<crate::getbufferproc>,
    pub bf_releasebuffer: Option<crate::releasebufferproc>,
}

pub type printfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut ::libc::FILE, arg3: c_int) -> c_int;

#[repr(C)]
#[derive(Debug)]
pub struct PyTypeObject {
    pub ob_base: object::PyVarObject,
    pub tp_name: *const c_char,
    pub tp_basicsize: Py_ssize_t,
    pub tp_itemsize: Py_ssize_t,
    pub tp_dealloc: Option<object::destructor>,
    #[cfg(not(Py_3_8))]
    pub tp_print: Option<printfunc>,
    #[cfg(Py_3_8)]
    pub tp_vectorcall_offset: Py_ssize_t,
    pub tp_getattr: Option<object::getattrfunc>,
    pub tp_setattr: Option<object::setattrfunc>,
    pub tp_as_async: *mut PyAsyncMethods,
    pub tp_repr: Option<object::reprfunc>,
    pub tp_as_number: *mut PyNumberMethods,
    pub tp_as_sequence: *mut PySequenceMethods,
    pub tp_as_mapping: *mut PyMappingMethods,
    pub tp_hash: Option<object::hashfunc>,
    pub tp_call: Option<object::ternaryfunc>,
    pub tp_str: Option<object::reprfunc>,
    pub tp_getattro: Option<object::getattrofunc>,
    pub tp_setattro: Option<object::setattrofunc>,
    pub tp_as_buffer: *mut PyBufferProcs,
    #[cfg(not(Py_GIL_DISABLED))]
    pub tp_flags: std::ffi::c_ulong,
    #[cfg(Py_GIL_DISABLED)]
    pub tp_flags: crate::impl_::AtomicCULong,
    pub tp_doc: *const c_char,
    pub tp_traverse: Option<object::traverseproc>,
    pub tp_clear: Option<object::inquiry>,
    pub tp_richcompare: Option<object::richcmpfunc>,
    pub tp_weaklistoffset: Py_ssize_t,
    pub tp_iter: Option<object::getiterfunc>,
    pub tp_iternext: Option<object::iternextfunc>,
    pub tp_methods: *mut PyMethodDef,
    pub tp_members: *mut PyMemberDef,
    pub tp_getset: *mut PyGetSetDef,
    pub tp_base: *mut PyTypeObject,
    pub tp_dict: *mut object::PyObject,
    pub tp_descr_get: Option<object::descrgetfunc>,
    pub tp_descr_set: Option<object::descrsetfunc>,
    pub tp_dictoffset: Py_ssize_t,
    pub tp_init: Option<object::initproc>,
    pub tp_alloc: Option<object::allocfunc>,
    pub tp_new: Option<object::newfunc>,
    pub tp_free: Option<object::freefunc>,
    pub tp_is_gc: Option<object::inquiry>,
    pub tp_bases: *mut object::PyObject,
    pub tp_mro: *mut object::PyObject,
    pub tp_cache: *mut object::PyObject,
    pub tp_subclasses: *mut object::PyObject,
    pub tp_weaklist: *mut object::PyObject,
    pub tp_del: Option<object::destructor>,
    pub tp_version_tag: c_uint,
    pub tp_finalize: Option<object::destructor>,
    #[cfg(Py_3_8)]
    pub tp_vectorcall: Option<vectorcallfunc>,
    #[cfg(Py_3_12)]
    pub tp_watched: c_char,
    #[cfg(any(all(PyPy, Py_3_8, not(Py_3_10)), all(not(PyPy), Py_3_8, not(Py_3_9))))]
    pub tp_print: Option<printfunc>,
    #[cfg(all(PyPy, not(Py_3_10)))]
    pub tp_pypy_flags: std::ffi::c_long,
    #[cfg(py_sys_config = "COUNT_ALLOCS")]
    pub tp_allocs: Py_ssize_t,
    #[cfg(py_sys_config = "COUNT_ALLOCS")]
    pub tp_frees: Py_ssize_t,
    #[cfg(py_sys_config = "COUNT_ALLOCS")]
    pub tp_maxalloc: Py_ssize_t,
    #[cfg(py_sys_config = "COUNT_ALLOCS")]
    pub tp_prev: *mut PyTypeObject,
    #[cfg(py_sys_config = "COUNT_ALLOCS")]
    pub tp_next: *mut PyTypeObject,
}

#[cfg(Py_3_11)]
#[repr(C)]
#[derive(Clone)]
struct _specialization_cache {
    getitem: *mut PyObject,
    #[cfg(Py_3_12)]
    getitem_version: u32,
    #[cfg(Py_3_13)]
    init: *mut PyObject,
}

#[repr(C)]
pub struct PyHeapTypeObject {
    pub ht_type: PyTypeObject,
    pub as_async: PyAsyncMethods,
    pub as_number: PyNumberMethods,
    pub as_mapping: PyMappingMethods,
    pub as_sequence: PySequenceMethods,
    pub as_buffer: PyBufferProcs,
    pub ht_name: *mut object::PyObject,
    pub ht_slots: *mut object::PyObject,
    pub ht_qualname: *mut object::PyObject,
    #[cfg(not(PyPy))]
    pub ht_cached_keys: *mut c_void,
    #[cfg(Py_3_9)]
    pub ht_module: *mut object::PyObject,
    #[cfg(all(Py_3_11, not(PyPy)))]
    _ht_tpname: *mut c_char,
    #[cfg(Py_3_14)]
    pub ht_token: *mut c_void,
    #[cfg(all(Py_3_11, not(PyPy)))]
    _spec_cache: _specialization_cache,
    #[cfg(all(Py_GIL_DISABLED, Py_3_14))]
    pub unique_id: Py_ssize_t,
}

impl Default for PyHeapTypeObject {
    #[inline]
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

#[inline]
#[cfg(not(Py_3_11))]
pub unsafe fn PyHeapType_GET_MEMBERS(etype: *mut PyHeapTypeObject) -> *mut PyMemberDef {
    let py_type = object::Py_TYPE(etype as *mut object::PyObject);
    let ptr = etype.offset((*py_type).tp_basicsize);
    ptr as *mut PyMemberDef
}

// skipped private _PyType_Name
// skipped private _PyType_Lookup
// skipped private _PyType_LookupRef

extern "C" {
    #[cfg(Py_3_12)]
    pub fn PyType_GetDict(o: *mut PyTypeObject) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyObject_Print")]
    pub fn PyObject_Print(o: *mut PyObject, fp: *mut ::libc::FILE, flags: c_int) -> c_int;

    // skipped private _Py_BreakPoint
    // skipped private _PyObject_Dump

    // skipped _PyObject_GetAttrId

    // skipped private _PyObject_GetDictPtr
    pub fn PyObject_CallFinalizer(arg1: *mut PyObject);
    #[cfg_attr(PyPy, link_name = "PyPyObject_CallFinalizerFromDealloc")]
    pub fn PyObject_CallFinalizerFromDealloc(arg1: *mut PyObject) -> c_int;

    // skipped private _PyObject_GenericGetAttrWithDict
    // skipped private _PyObject_GenericSetAttrWithDict
    // skipped private _PyObject_FunctionStr
}

// skipped Py_SETREF
// skipped Py_XSETREF

// skipped private _PyObject_ASSERT_FROM
// skipped private _PyObject_ASSERT_WITH_MSG
// skipped private _PyObject_ASSERT
// skipped private _PyObject_ASSERT_FAILED_MSG
// skipped private _PyObject_AssertFailed

// skipped private _PyTrash_begin
// skipped private _PyTrash_end

// skipped _PyTrash_thread_deposit_object
// skipped _PyTrash_thread_destroy_chain

// skipped Py_TRASHCAN_BEGIN
// skipped Py_TRASHCAN_END

// skipped PyObject_GetItemData

// skipped PyObject_VisitManagedDict
// skipped _PyObject_SetManagedDict
// skipped PyObject_ClearManagedDict

// skipped TYPE_MAX_WATCHERS

// skipped PyType_WatchCallback
// skipped PyType_AddWatcher
// skipped PyType_ClearWatcher
// skipped PyType_Watch
// skipped PyType_Unwatch

// skipped PyUnstable_Type_AssignVersionTag

// skipped PyRefTracerEvent

// skipped PyRefTracer
// skipped PyRefTracer_SetTracer
// skipped PyRefTracer_GetTracer
