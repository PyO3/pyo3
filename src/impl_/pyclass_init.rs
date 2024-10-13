//! Contains initialization utilities for `#[pyclass]`.
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::internal::get_slot::TP_ALLOC;
use crate::types::PyType;
use crate::{ffi, Borrowed, PyErr, PyResult, Python};
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
            type_object: *mut PyTypeObject,
            subtype: *mut PyTypeObject,
        ) -> PyResult<*mut ffi::PyObject> {
            // HACK (due to FIXME below): PyBaseObject_Type's tp_new isn't happy with NULL arguments
            let is_base_object = type_object == std::ptr::addr_of_mut!(ffi::PyBaseObject_Type);
            let subtype_borrowed: Borrowed<'_, '_, PyType> = subtype
                .cast::<ffi::PyObject>()
                .assume_borrowed_unchecked(py)
                .downcast_unchecked();

            if is_base_object {
                let alloc = subtype_borrowed
                    .get_slot(TP_ALLOC)
                    .unwrap_or(ffi::PyType_GenericAlloc);

                let obj = alloc(subtype, 0);
                return if obj.is_null() {
                    Err(PyErr::fetch(py))
                } else {
                    Ok(obj)
                };
            }

            #[cfg(Py_LIMITED_API)]
            unreachable!("subclassing native types is not possible with the `abi3` feature");

            #[cfg(not(Py_LIMITED_API))]
            {
                match (*type_object).tp_new {
                    // FIXME: Call __new__ with actual arguments
                    Some(newfunc) => {
                        let obj = newfunc(subtype, std::ptr::null_mut(), std::ptr::null_mut());
                        if obj.is_null() {
                            Err(PyErr::fetch(py))
                        } else {
                            Ok(obj)
                        }
                    }
                    None => Err(crate::exceptions::PyTypeError::new_err(
                        "base type without tp_new",
                    )),
                }
            }
        }
        let type_object = T::type_object_raw(py);
        inner(py, type_object, subtype)
    }

    #[inline]
    fn can_be_subclassed(&self) -> bool {
        true
    }
}
