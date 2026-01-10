//! Contains initialization utilities for `#[pyclass]`.
use crate::exceptions::PyTypeError;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::internal::get_slot::TP_NEW;
use crate::types::{PyTuple, PyType};
use crate::{ffi, PyErr, PyResult, Python};
use crate::{ffi::PyTypeObject, sealed::Sealed, type_object::PyTypeInfo};
use std::marker::PhantomData;

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
    ) -> PyResult<*mut ffi::PyObject>;

    #[doc(hidden)]
    fn can_be_subclassed(&self) -> bool;
}

/// Initializer for Python native types, like `PyDict`.
pub struct PyNativeTypeInitializer<T: PyTypeInfo>(pub PhantomData<T>);

impl<T: PyTypeInfo> PyObjectInit<T> for PyNativeTypeInitializer<T> {
    unsafe fn into_new_object(
        self,
        py: Python<'_>,
        subtype: *mut PyTypeObject,
    ) -> PyResult<*mut ffi::PyObject> {
        unsafe fn inner(
            py: Python<'_>,
            type_ptr: *mut PyTypeObject,
            subtype: *mut PyTypeObject,
        ) -> PyResult<*mut ffi::PyObject> {
            let tp_new = unsafe {
                type_ptr
                    .cast::<ffi::PyObject>()
                    .assume_borrowed_unchecked(py)
                    .cast_unchecked::<PyType>()
                    .get_slot(TP_NEW)
                    .ok_or_else(|| PyTypeError::new_err("base type without tp_new"))?
            };

            // TODO: make it possible to provide real arguments to the base tp_new
            let obj = unsafe { tp_new(subtype, PyTuple::empty(py).as_ptr(), std::ptr::null_mut()) };
            if obj.is_null() {
                Err(PyErr::fetch(py))
            } else {
                Ok(obj)
            }
        }
        unsafe { inner(py, T::type_object_raw(py), subtype) }
    }

    #[inline]
    fn can_be_subclassed(&self) -> bool {
        true
    }
}
