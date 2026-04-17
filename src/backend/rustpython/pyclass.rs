use crate::impl_::pycell::PyClassObjectBaseLayout;
use crate::impl_::pyclass::{PyClassBaseType, PyClassImpl};
use crate::types::any::PyAnyMethods;
use crate::types::PyType;
use crate::{ffi, Bound, PyResult, PyTypeInfo, Python};
use std::ffi::{c_int, c_void};
use std::thread;

const PYO3_RUSTPYTHON_HEAP_TYPE_ATTR: &str = "__pyo3_rustpython_heap_type__";

#[cfg(PyRustPython)]
pub(crate) fn install_post_init_storage<T: PyClassImpl + PyTypeInfo>(
    py: Python<'_>,
    obj: *mut ffi::PyObject,
) where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyClassObjectBaseLayout<T::BaseType>,
{
    crate::backend::rustpython_storage::install_sidecar_owner::<T>(py, obj);
}

#[cfg(not(PyRustPython))]
pub(crate) fn install_post_init_storage<T: PyClassImpl + PyTypeInfo>(
    _py: Python<'_>,
    _obj: *mut ffi::PyObject,
) where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyClassObjectBaseLayout<T::BaseType>,
{
}

pub(crate) fn supports_managed_dict_and_weaklist_offsets() -> bool {
    true
}

pub(crate) fn use_generic_dict_getter() -> bool {
    true
}

pub(crate) fn use_pre_310_heaptype_doc_cleanup() -> bool {
    false
}

pub(crate) fn use_pre_39_type_object_fixup() -> bool {
    false
}

pub(crate) fn maybe_object_init_slot(
    py: Python<'_>,
    has_new: bool,
    has_init: bool,
    tp_base: *mut ffi::PyTypeObject,
) -> Option<*mut c_void> {
    (has_new && !has_init && tp_base == crate::PyAny::type_object_raw(py))
        .then_some(rustpython_noop_init as *mut c_void)
}

pub(crate) fn finalize_type(
    type_object: &Bound<'_, PyType>,
    module_name: Option<&'static str>,
) -> PyResult<()> {
    type_object.setattr(PYO3_RUSTPYTHON_HEAP_TYPE_ATTR, true)?;
    if let Some(module_name) = module_name {
        type_object.setattr("__module__", module_name)?;
    }
    Ok(())
}

pub(crate) fn object_init_slot_type() -> c_int {
    ffi::Py_tp_init
}

pub(crate) fn thread_checker_matches_runtime_or_owner(owner: thread::ThreadId) -> bool {
    let current = thread::current().id();
    current == owner
        || {
            #[cfg(PyRustPython)]
            {
                crate::ffi::rustpython_runtime_thread_id()
                    .is_some_and(|runtime_thread| runtime_thread == current)
            }
            #[cfg(not(PyRustPython))]
            {
                false
            }
        }
}

unsafe extern "C" fn rustpython_noop_init(
    _slf: *mut ffi::PyObject,
    _args: *mut ffi::PyObject,
    _kwargs: *mut ffi::PyObject,
) -> c_int {
    0
}
