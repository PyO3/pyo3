// skipped private _Py_NewReference
// skipped private _Py_NewReferenceNoTotal
// skipped private _Py_ResurrectReference
// skipped private _Py_ForgetReference

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
    use core::ffi::{c_char, c_int, c_void};
    use core::ptr;

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
    pub nb_add: Option<binaryfunc>,
    pub nb_subtract: Option<binaryfunc>,
    pub nb_multiply: Option<binaryfunc>,
    pub nb_remainder: Option<binaryfunc>,
    pub nb_divmod: Option<binaryfunc>,
    pub nb_power: Option<ternaryfunc>,
    pub nb_negative: Option<unaryfunc>,
    pub nb_positive: Option<unaryfunc>,
    pub nb_absolute: Option<unaryfunc>,
    pub nb_bool: Option<inquiry>,
    pub nb_invert: Option<unaryfunc>,
    pub nb_lshift: Option<binaryfunc>,
    pub nb_rshift: Option<binaryfunc>,
    pub nb_and: Option<binaryfunc>,
    pub nb_xor: Option<binaryfunc>,
    pub nb_or: Option<binaryfunc>,
    pub nb_int: Option<unaryfunc>,
    pub nb_reserved: *mut c_void,
    pub nb_float: Option<unaryfunc>,
    pub nb_inplace_add: Option<binaryfunc>,
    pub nb_inplace_subtract: Option<binaryfunc>,
    pub nb_inplace_multiply: Option<binaryfunc>,
    pub nb_inplace_remainder: Option<binaryfunc>,
    pub nb_inplace_power: Option<ternaryfunc>,
    pub nb_inplace_lshift: Option<binaryfunc>,
    pub nb_inplace_rshift: Option<binaryfunc>,
    pub nb_inplace_and: Option<binaryfunc>,
    pub nb_inplace_xor: Option<binaryfunc>,
    pub nb_inplace_or: Option<binaryfunc>,
    pub nb_floor_divide: Option<binaryfunc>,
    pub nb_true_divide: Option<binaryfunc>,
    pub nb_inplace_floor_divide: Option<binaryfunc>,
    pub nb_inplace_true_divide: Option<binaryfunc>,
    pub nb_index: Option<unaryfunc>,
    pub nb_matrix_multiply: Option<binaryfunc>,
    pub nb_inplace_matrix_multiply: Option<binaryfunc>,
}

#[repr(C)]
#[derive(Clone)]
pub struct PySequenceMethods {
    pub sq_length: Option<lenfunc>,
    pub sq_concat: Option<binaryfunc>,
    pub sq_repeat: Option<ssizeargfunc>,
    pub sq_item: Option<ssizeargfunc>,
    pub was_sq_slice: *mut c_void,
    pub sq_ass_item: Option<ssizeobjargproc>,
    pub was_sq_ass_slice: *mut c_void,
    pub sq_contains: Option<objobjproc>,
    pub sq_inplace_concat: Option<binaryfunc>,
    pub sq_inplace_repeat: Option<ssizeargfunc>,
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct PyMappingMethods {
    pub mp_length: Option<lenfunc>,
    pub mp_subscript: Option<binaryfunc>,
    pub mp_ass_subscript: Option<objobjargproc>,
}

#[cfg(Py_3_10)]
pub type sendfunc = unsafe extern "C" fn(
    iter: *mut PyObject,
    value: *mut PyObject,
    result: *mut *mut PyObject,
) -> PySendResult;

#[repr(C)]
#[derive(Clone, Default)]
pub struct PyAsyncMethods {
    pub am_await: Option<unaryfunc>,
    pub am_aiter: Option<unaryfunc>,
    pub am_anext: Option<unaryfunc>,
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
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut FILE, arg3: c_int) -> c_int;

#[repr(C)]
#[derive(Debug)]
pub struct PyTypeObject {
    pub ob_base: PyVarObject,
    pub tp_name: *const c_char,
    pub tp_basicsize: Py_ssize_t,
    pub tp_itemsize: Py_ssize_t,
    pub tp_dealloc: Option<destructor>,
    pub tp_vectorcall_offset: Py_ssize_t,
    pub tp_getattr: Option<getattrfunc>,
    pub tp_setattr: Option<setattrfunc>,
    pub tp_as_async: *mut PyAsyncMethods,
    pub tp_repr: Option<reprfunc>,
    pub tp_as_number: *mut PyNumberMethods,
    pub tp_as_sequence: *mut PySequenceMethods,
    pub tp_as_mapping: *mut PyMappingMethods,
    pub tp_hash: Option<hashfunc>,
    pub tp_call: Option<ternaryfunc>,
    pub tp_str: Option<reprfunc>,
    pub tp_getattro: Option<getattrofunc>,
    pub tp_setattro: Option<setattrofunc>,
    pub tp_as_buffer: *mut PyBufferProcs,
    #[cfg(not(Py_GIL_DISABLED))]
    pub tp_flags: core::ffi::c_ulong,
    #[cfg(Py_GIL_DISABLED)]
    pub tp_flags: crate::impl_::AtomicCULong,
    pub tp_doc: *const c_char,
    pub tp_traverse: Option<traverseproc>,
    pub tp_clear: Option<inquiry>,
    pub tp_richcompare: Option<richcmpfunc>,
    pub tp_weaklistoffset: Py_ssize_t,
    pub tp_iter: Option<getiterfunc>,
    pub tp_iternext: Option<iternextfunc>,
    pub tp_methods: *mut PyMethodDef,
    pub tp_members: *mut PyMemberDef,
    pub tp_getset: *mut PyGetSetDef,
    pub tp_base: *mut PyTypeObject,
    pub tp_dict: *mut PyObject,
    pub tp_descr_get: Option<descrgetfunc>,
    pub tp_descr_set: Option<descrsetfunc>,
    pub tp_dictoffset: Py_ssize_t,
    pub tp_init: Option<initproc>,
    pub tp_alloc: Option<allocfunc>,
    pub tp_new: Option<newfunc>,
    pub tp_free: Option<freefunc>,
    pub tp_is_gc: Option<inquiry>,
    pub tp_bases: *mut PyObject,
    pub tp_mro: *mut PyObject,
    pub tp_cache: *mut PyObject,
    pub tp_subclasses: *mut PyObject,
    pub tp_weaklist: *mut PyObject,
    pub tp_del: Option<destructor>,
    pub tp_version_tag: c_uint,
    pub tp_finalize: Option<destructor>,
    pub tp_vectorcall: Option<vectorcallfunc>,
    #[cfg(Py_3_12)]
    pub tp_watched: c_char,
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
    #[cfg(Py_3_13)]
    pub tp_versions_used: u16,
    #[cfg(Py_3_15)]
    pub _tp_iteritem: Option<_Py_iteritemfunc>,
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
    pub ht_name: *mut PyObject,
    pub ht_slots: *mut PyObject,
    pub ht_qualname: *mut PyObject,
    #[cfg(not(PyPy))]
    pub ht_cached_keys: *mut c_void,
    pub ht_module: *mut PyObject,
    #[cfg(all(Py_3_11, not(PyPy)))]
    _ht_tpname: *mut c_char,
    #[cfg(Py_3_14)]
    pub ht_token: *mut c_void,
    #[cfg(all(Py_3_11, not(PyPy)))]
    _spec_cache: _specialization_cache,
    #[cfg(all(Py_GIL_DISABLED, Py_3_14))]
    pub unique_id: Py_ssize_t,
}

#[inline]
#[cfg(not(Py_3_11))]
pub unsafe fn PyHeapType_GET_MEMBERS(etype: *mut PyHeapTypeObject) -> *mut PyMemberDef {
    let py_type = Py_TYPE(etype as *mut PyObject);
    let ptr = etype.byte_offset((*py_type).tp_basicsize);
    ptr as *mut PyMemberDef
}

// skipped private _PyType_Name
// skipped private _PyType_Lookup
// skipped private _PyType_LookupRef

extern_libpython! {
    #[cfg(Py_3_12)]
    pub fn PyType_GetDict(o: *mut PyTypeObject) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyObject_Print")]
    pub fn PyObject_Print(o: *mut PyObject, fp: *mut FILE, flags: c_int) -> c_int;

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

#[cfg(Py_3_14)]
extern_libpython! {
    // skipped PyUnstable_Object_EnableDeferredRefcount

    pub fn PyUnstable_Object_IsUniqueReferencedTemporary(obj: *mut PyObject) -> c_int;

    // skipped PyUnstable_IsImmortal

    pub fn PyUnstable_TryIncRef(obj: *mut PyObject) -> c_int;

    pub fn PyUnstable_EnableTryIncRef(obj: *mut PyObject);

    pub fn PyUnstable_Object_IsUniquelyReferenced(op: *mut PyObject) -> c_int;
}

#[cfg(Py_3_15)]
extern_libpython! {
    pub fn PyUnstable_SetImmortal(op: *mut PyObject) -> c_int;
}
