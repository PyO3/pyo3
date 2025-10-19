use crate::{
    ffi,
    types::{PyType, PyTypeMethods},
    Borrowed, Bound,
};
use std::ffi::c_int;

impl Bound<'_, PyType> {
    #[inline]
    pub(crate) fn get_slot<const S: c_int>(&self, slot: Slot<S>) -> <Slot<S> as GetSlotImpl>::Type
    where
        Slot<S>: GetSlotImpl,
    {
        // SAFETY: `self` is a valid type object.
        unsafe {
            slot.get_slot(
                self.as_type_ptr(),
                #[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
                is_runtime_3_10(self.py()),
            )
        }
    }
}

impl Borrowed<'_, '_, PyType> {
    #[inline]
    pub(crate) fn get_slot<const S: c_int>(self, slot: Slot<S>) -> <Slot<S> as GetSlotImpl>::Type
    where
        Slot<S>: GetSlotImpl,
    {
        // SAFETY: `self` is a valid type object.
        unsafe {
            slot.get_slot(
                self.as_type_ptr(),
                #[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
                is_runtime_3_10(self.py()),
            )
        }
    }
}

/// Gets a slot from a raw FFI pointer.
///
/// Safety:
///   - `ty` must be a valid non-null pointer to a `PyTypeObject`.
///   - The Python runtime must be initialized
pub(crate) unsafe fn get_slot<const S: c_int>(
    ty: *mut ffi::PyTypeObject,
    slot: Slot<S>,
) -> <Slot<S> as GetSlotImpl>::Type
where
    Slot<S>: GetSlotImpl,
{
    unsafe {
        slot.get_slot(
            ty,
            // SAFETY: the Python runtime is initialized
            #[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
            is_runtime_3_10(crate::Python::assume_attached()),
        )
    }
}

pub(crate) trait GetSlotImpl {
    type Type;

    /// Gets the requested slot from a type object.
    ///
    /// Safety:
    ///  - `ty` must be a valid non-null pointer to a `PyTypeObject`.
    ///  - `is_runtime_3_10` must be `false` if the runtime is not Python 3.10 or later.
    unsafe fn get_slot(
        self,
        ty: *mut ffi::PyTypeObject,
        #[cfg(all(Py_LIMITED_API, not(Py_3_10)))] is_runtime_3_10: bool,
    ) -> Self::Type;
}

#[derive(Copy, Clone)]
pub(crate) struct Slot<const S: c_int>;

macro_rules! impl_slots {
    ($($name:ident: ($slot:ident, $field:ident) -> $tp:ty),+ $(,)?) => {
        $(
            pub (crate) const $name: Slot<{ ffi::$slot }> = Slot;

            impl GetSlotImpl for Slot<{ ffi::$slot }> {
                type Type = $tp;

                #[inline]
                unsafe fn get_slot(
                    self,
                    ty: *mut ffi::PyTypeObject,
                    #[cfg(all(Py_LIMITED_API, not(Py_3_10)))] is_runtime_3_10: bool
                ) -> Self::Type {
                    #[cfg(not(Py_LIMITED_API))]
                    {
                        unsafe {(*ty).$field }
                    }

                    #[cfg(Py_LIMITED_API)]
                    {
                        #[cfg(not(Py_3_10))]
                        {
                            // Calling PyType_GetSlot on static types is not valid before Python 3.10
                            // ... so the workaround is to first do a runtime check for these versions
                            // (3.7, 3.8, 3.9) and then look in the type object anyway. This is only ok
                            // because we know that the interpreter is not going to change the size
                            // of the type objects for these historical versions.
                            if !is_runtime_3_10 && unsafe {ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE)} == 0
                            {
                                return unsafe {(*ty.cast::<PyTypeObject39Snapshot>()).$field};
                            }
                        }

                        // SAFETY: slot type is set carefully to be valid
                        unsafe {std::mem::transmute(ffi::PyType_GetSlot(ty, ffi::$slot))}
                    }
                }
            }
        )*
    };
}

// Slots are implemented on-demand as needed.)
impl_slots! {
    TP_ALLOC: (Py_tp_alloc, tp_alloc) -> Option<ffi::allocfunc>,
    TP_BASE: (Py_tp_base, tp_base) -> *mut ffi::PyTypeObject,
    TP_CLEAR: (Py_tp_clear, tp_clear) -> Option<ffi::inquiry>,
    TP_DESCR_GET: (Py_tp_descr_get, tp_descr_get) -> Option<ffi::descrgetfunc>,
    TP_FREE: (Py_tp_free, tp_free) -> Option<ffi::freefunc>,
    TP_TRAVERSE: (Py_tp_traverse, tp_traverse) -> Option<ffi::traverseproc>,
}

#[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
fn is_runtime_3_10(py: crate::Python<'_>) -> bool {
    use crate::sync::PyOnceLock;

    static IS_RUNTIME_3_10: PyOnceLock<bool> = PyOnceLock::new();
    *IS_RUNTIME_3_10.get_or_init(py, || py.version_info() >= (3, 10))
}

#[repr(C)]
#[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
pub struct PyNumberMethods39Snapshot {
    pub nb_add: Option<ffi::binaryfunc>,
    pub nb_subtract: Option<ffi::binaryfunc>,
    pub nb_multiply: Option<ffi::binaryfunc>,
    pub nb_remainder: Option<ffi::binaryfunc>,
    pub nb_divmod: Option<ffi::binaryfunc>,
    pub nb_power: Option<ffi::ternaryfunc>,
    pub nb_negative: Option<ffi::unaryfunc>,
    pub nb_positive: Option<ffi::unaryfunc>,
    pub nb_absolute: Option<ffi::unaryfunc>,
    pub nb_bool: Option<ffi::inquiry>,
    pub nb_invert: Option<ffi::unaryfunc>,
    pub nb_lshift: Option<ffi::binaryfunc>,
    pub nb_rshift: Option<ffi::binaryfunc>,
    pub nb_and: Option<ffi::binaryfunc>,
    pub nb_xor: Option<ffi::binaryfunc>,
    pub nb_or: Option<ffi::binaryfunc>,
    pub nb_int: Option<ffi::unaryfunc>,
    pub nb_reserved: *mut std::ffi::c_void,
    pub nb_float: Option<ffi::unaryfunc>,
    pub nb_inplace_add: Option<ffi::binaryfunc>,
    pub nb_inplace_subtract: Option<ffi::binaryfunc>,
    pub nb_inplace_multiply: Option<ffi::binaryfunc>,
    pub nb_inplace_remainder: Option<ffi::binaryfunc>,
    pub nb_inplace_power: Option<ffi::ternaryfunc>,
    pub nb_inplace_lshift: Option<ffi::binaryfunc>,
    pub nb_inplace_rshift: Option<ffi::binaryfunc>,
    pub nb_inplace_and: Option<ffi::binaryfunc>,
    pub nb_inplace_xor: Option<ffi::binaryfunc>,
    pub nb_inplace_or: Option<ffi::binaryfunc>,
    pub nb_floor_divide: Option<ffi::binaryfunc>,
    pub nb_true_divide: Option<ffi::binaryfunc>,
    pub nb_inplace_floor_divide: Option<ffi::binaryfunc>,
    pub nb_inplace_true_divide: Option<ffi::binaryfunc>,
    pub nb_index: Option<ffi::unaryfunc>,
    pub nb_matrix_multiply: Option<ffi::binaryfunc>,
    pub nb_inplace_matrix_multiply: Option<ffi::binaryfunc>,
}

#[repr(C)]
#[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
pub struct PySequenceMethods39Snapshot {
    pub sq_length: Option<ffi::lenfunc>,
    pub sq_concat: Option<ffi::binaryfunc>,
    pub sq_repeat: Option<ffi::ssizeargfunc>,
    pub sq_item: Option<ffi::ssizeargfunc>,
    pub was_sq_slice: *mut std::ffi::c_void,
    pub sq_ass_item: Option<ffi::ssizeobjargproc>,
    pub was_sq_ass_slice: *mut std::ffi::c_void,
    pub sq_contains: Option<ffi::objobjproc>,
    pub sq_inplace_concat: Option<ffi::binaryfunc>,
    pub sq_inplace_repeat: Option<ffi::ssizeargfunc>,
}

#[repr(C)]
#[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
pub struct PyMappingMethods39Snapshot {
    pub mp_length: Option<ffi::lenfunc>,
    pub mp_subscript: Option<ffi::binaryfunc>,
    pub mp_ass_subscript: Option<ffi::objobjargproc>,
}

#[repr(C)]
#[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
pub struct PyAsyncMethods39Snapshot {
    pub am_await: Option<ffi::unaryfunc>,
    pub am_aiter: Option<ffi::unaryfunc>,
    pub am_anext: Option<ffi::unaryfunc>,
}

#[repr(C)]
#[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
pub struct PyBufferProcs39Snapshot {
    // not available in limited api, but structure needs to have the right size
    pub bf_getbuffer: *mut std::ffi::c_void,
    pub bf_releasebuffer: *mut std::ffi::c_void,
}

/// Snapshot of the structure of PyTypeObject for Python 3.7 through 3.9.
///
/// This is used as a fallback for static types in abi3 when the Python version is less than 3.10;
/// this is a bit of a hack but there's no better option and the structure of the type object is
/// not going to change for those historical versions.
#[repr(C)]
#[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
struct PyTypeObject39Snapshot {
    pub ob_base: ffi::PyVarObject,
    pub tp_name: *const std::ffi::c_char,
    pub tp_basicsize: ffi::Py_ssize_t,
    pub tp_itemsize: ffi::Py_ssize_t,
    pub tp_dealloc: Option<ffi::destructor>,
    #[cfg(not(Py_3_8))]
    pub tp_print: *mut std::ffi::c_void, // stubbed out, not available in limited API
    #[cfg(Py_3_8)]
    pub tp_vectorcall_offset: ffi::Py_ssize_t,
    pub tp_getattr: Option<ffi::getattrfunc>,
    pub tp_setattr: Option<ffi::setattrfunc>,
    pub tp_as_async: *mut PyAsyncMethods39Snapshot,
    pub tp_repr: Option<ffi::reprfunc>,
    pub tp_as_number: *mut PyNumberMethods39Snapshot,
    pub tp_as_sequence: *mut PySequenceMethods39Snapshot,
    pub tp_as_mapping: *mut PyMappingMethods39Snapshot,
    pub tp_hash: Option<ffi::hashfunc>,
    pub tp_call: Option<ffi::ternaryfunc>,
    pub tp_str: Option<ffi::reprfunc>,
    pub tp_getattro: Option<ffi::getattrofunc>,
    pub tp_setattro: Option<ffi::setattrofunc>,
    pub tp_as_buffer: *mut PyBufferProcs39Snapshot,
    pub tp_flags: std::ffi::c_ulong,
    pub tp_doc: *const std::ffi::c_char,
    pub tp_traverse: Option<ffi::traverseproc>,
    pub tp_clear: Option<ffi::inquiry>,
    pub tp_richcompare: Option<ffi::richcmpfunc>,
    pub tp_weaklistoffset: ffi::Py_ssize_t,
    pub tp_iter: Option<ffi::getiterfunc>,
    pub tp_iternext: Option<ffi::iternextfunc>,
    pub tp_methods: *mut ffi::PyMethodDef,
    pub tp_members: *mut ffi::PyMemberDef,
    pub tp_getset: *mut ffi::PyGetSetDef,
    pub tp_base: *mut ffi::PyTypeObject,
    pub tp_dict: *mut ffi::PyObject,
    pub tp_descr_get: Option<ffi::descrgetfunc>,
    pub tp_descr_set: Option<ffi::descrsetfunc>,
    pub tp_dictoffset: ffi::Py_ssize_t,
    pub tp_init: Option<ffi::initproc>,
    pub tp_alloc: Option<ffi::allocfunc>,
    pub tp_new: Option<ffi::newfunc>,
    pub tp_free: Option<ffi::freefunc>,
    pub tp_is_gc: Option<ffi::inquiry>,
    pub tp_bases: *mut ffi::PyObject,
    pub tp_mro: *mut ffi::PyObject,
    pub tp_cache: *mut ffi::PyObject,
    pub tp_subclasses: *mut ffi::PyObject,
    pub tp_weaklist: *mut ffi::PyObject,
    pub tp_del: Option<ffi::destructor>,
    pub tp_version_tag: std::ffi::c_uint,
    pub tp_finalize: Option<ffi::destructor>,
    #[cfg(Py_3_8)]
    pub tp_vectorcall: Option<ffi::vectorcallfunc>,
}
