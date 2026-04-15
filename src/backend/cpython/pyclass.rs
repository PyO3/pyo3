use crate::impl_::pyclass::PyClassImpl;
use crate::types::PyType;
use crate::{ffi, Bound, PyResult, PyTypeInfo, Python};
use std::ffi::{c_int, c_void};

pub(crate) fn install_post_init_storage<T: PyClassImpl + PyTypeInfo>(
    _py: Python<'_>,
    _obj: *mut ffi::PyObject,
) {
}

pub(crate) fn supports_managed_dict_and_weaklist_offsets() -> bool {
    cfg!(any(Py_3_9, not(Py_LIMITED_API)))
}

pub(crate) fn use_generic_dict_getter() -> bool {
    cfg!(any(Py_3_10, not(Py_LIMITED_API)))
}

pub(crate) fn use_pre_310_heaptype_doc_cleanup() -> bool {
    cfg!(all(not(Py_LIMITED_API), not(Py_3_10)))
}

pub(crate) fn use_pre_39_type_object_fixup() -> bool {
    cfg!(all(not(Py_LIMITED_API), not(Py_3_9)))
}

pub(crate) fn maybe_object_init_slot(
    _py: Python<'_>,
    _has_new: bool,
    _has_init: bool,
    _tp_base: *mut ffi::PyTypeObject,
) -> Option<*mut c_void> {
    None
}

pub(crate) fn finalize_type(
    _type_object: &Bound<'_, PyType>,
    _module_name: Option<&'static str>,
) -> PyResult<()> {
    Ok(())
}

pub(crate) fn object_init_slot_type() -> c_int {
    ffi::Py_tp_init
}
