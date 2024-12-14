//! Contains initialization utilities for `#[pyclass]`.
use crate::exceptions::PyTypeError;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::internal::get_slot::TP_NEW;
use crate::types::{PyDict, PyTuple, PyType};
use crate::{ffi, Bound, PyErr, PyResult, Python};
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
            override_tp_new: Option<ffi::newfunc>,
        ) -> PyResult<*mut ffi::PyObject> {
            let tp_new = if let Some(tp_new) = override_tp_new {
                tp_new
            } else {
                native_base_type
                    .cast::<ffi::PyObject>()
                    .assume_borrowed_unchecked(py)
                    .downcast_unchecked::<PyType>()
                    .get_slot(TP_NEW)
                    .ok_or_else(|| {
                        PyTypeError::new_err("cannot construct type that does not define __new__")
                    })?
            };

            let obj = tp_new(
                subtype,
                args.as_ptr(),
                kwargs
                    .map(|obj| obj.as_ptr())
                    .unwrap_or(std::ptr::null_mut()),
            );

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
            T::OVERRIDE_TP_NEW,
        )
    }

    #[inline]
    fn can_be_subclassed(&self) -> bool {
        true
    }
}
