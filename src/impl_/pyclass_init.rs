//! Contains initialization utilities for `#[pyclass]`.
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::internal::get_slot::TP_NEW;
use crate::types::{PyDict, PyTuple, PyType};
use crate::{ffi, Borrowed, Bound, PyErr, PyResult, Python};
use crate::{ffi::PyTypeObject, sealed::Sealed, type_object::PyTypeInfo};
use std::marker::PhantomData;

use super::pyclass::PyClassBaseType;

/// Initializer for Python types.
///
/// This trait is intended to use internally for distinguishing `#[pyclass]` and
/// Python native types.
pub trait PyObjectInit<T>: Sized + Sealed {
    /// # Safety
    /// - `subtype` must be a valid pointer to a type object of T or a subclass.
    unsafe fn into_new_object(
        self,
        py: Python<'_>,
        subtype: *mut PyTypeObject,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<*mut ffi::PyObject>;

    #[doc(hidden)]
    fn can_be_subclassed(&self) -> bool;
}

/// Initializer for Python native types, like [PyDict].
pub struct PyNativeTypeInitializer<T: PyTypeInfo + PyClassBaseType>(pub PhantomData<T>);

impl<T: PyTypeInfo + PyClassBaseType> PyObjectInit<T> for PyNativeTypeInitializer<T> {
    /// call `__new__` ([ffi::PyTypeObject::tp_new]) for the native base type.
    /// This will allocate a new python object and initialize the part relating to the native base type.
    unsafe fn into_new_object(
        self,
        py: Python<'_>,
        subtype: *mut PyTypeObject,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<*mut ffi::PyObject> {
        unsafe fn inner(
            py: Python<'_>,
            native_base_type: *mut PyTypeObject,
            subtype: *mut PyTypeObject,
            args: &Bound<'_, PyTuple>,
            kwargs: Option<&Bound<'_, PyDict>>,
            new_accepts_arguments: bool,
        ) -> PyResult<*mut ffi::PyObject> {
            let native_base_type_borrowed: Borrowed<'_, '_, PyType> = native_base_type
                .cast::<ffi::PyObject>()
                .assume_borrowed_unchecked(py)
                .downcast_unchecked();
            let tp_new = native_base_type_borrowed
                .get_slot(TP_NEW)
                .unwrap_or(ffi::PyType_GenericNew);

            let obj = if new_accepts_arguments {
                tp_new(
                    subtype,
                    args.as_ptr(),
                    kwargs
                        .map(|obj| obj.as_ptr())
                        .unwrap_or(std::ptr::null_mut()),
                )
            } else {
                let args = PyTuple::empty(py);
                tp_new(subtype, args.as_ptr(), std::ptr::null_mut())
            };

            if obj.is_null() {
                Err(PyErr::fetch(py))
            } else {
                Ok(obj)
            }
        }
        inner(
            py,
            T::type_object_raw(py),
            subtype,
            args,
            kwargs,
            T::NEW_ACCEPTS_ARGUMENTS,
        )
    }

    #[inline]
    fn can_be_subclassed(&self) -> bool {
        true
    }
}
