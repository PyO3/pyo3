// Copyright (c) 2017-present PyO3 Project and Contributors

use ffi;
use err::{PyErr, PyResult};
use python::{Python, ToPythonPointer};
use typeob::{PyTypeInfo, PyObjectAlloc};


pub struct pptr<'p>(Python<'p>, *mut ffi::PyObject);


impl<'p> pptr<'p> {

    /// Create new python object and move T instance under python management
    pub fn new<T>(py: Python<'p>, value: T) -> PyResult<pptr<'p>> where T: PyObjectAlloc<Type=T>
    {
        let ptr = unsafe {
            try!(<T as PyObjectAlloc>::alloc(py, value))
        };
        Ok(pptr(py, ptr))
    }

    /// Creates a Py instance for the given FFI pointer.
    /// This moves ownership over the pointer into the Py.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> pptr<'p> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        pptr(py, ptr)
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub unsafe fn from_owned_ptr_or_panic(py: Python<'p>, ptr: *mut ffi::PyObject) -> pptr<'p>
    {
        if ptr.is_null() {
            ::err::panic_after_error();
        } else {
            pptr::from_owned_ptr(py, ptr)
        }
    }

    /// Construct pppt<'p> from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is `null`.
    /// Unsafe because the pointer might be invalid.
    pub unsafe fn from_owned_ptr_or_err(py: Python<'p>, ptr: *mut ffi::PyObject)
                                        -> PyResult<pptr<'p>>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(pptr::from_owned_ptr(py, ptr))
        }
    }

    /// Creates a pptr<'p> instance for the given FFI pointer.
    /// This moves ownership over the pointer into the pptr<'p>.
    /// Returns None for null pointers; undefined behavior if the pointer is invalid.
    #[inline]
     pub unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject)
                                         -> Option<pptr<'p>> {
        if ptr.is_null() {
            None
        } else {
            Some(pptr::from_owned_ptr(py, ptr))
        }
    }

    /// Creates a Py instance for the given FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> pptr<'p> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        ffi::Py_INCREF(ptr);
        pptr(py, ptr)
    }

    /// Creates a Py instance for the given FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    #[inline]
    pub unsafe fn from_borrowed_ptr_opt(py: Python<'p>,
                                        ptr: *mut ffi::PyObject) -> Option<pptr<'p>> {
        if ptr.is_null() {
            None
        } else {
            debug_assert!(ffi::Py_REFCNT(ptr) > 0);
            ffi::Py_INCREF(ptr);
            Some(pptr(py, ptr))
        }
    }

    /// Gets the reference count of this Py object.
    #[inline]
    pub fn get_refcnt(&self) -> usize {
        unsafe { ffi::Py_REFCNT(self.1) as usize }
    }

    pub fn token<'a>(&'a self) -> Python<'p> {
        self.0
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub fn cast_from_owned_ptr<T>(py: Python<'p>, ptr: *mut ffi::PyObject)
                                  -> Result<pptr<'p>, ::PyDowncastError<'p>>
        where T: PyTypeInfo
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(ptr, T::type_object()) != 0 };

        if checked {
            Ok( unsafe { pptr::from_owned_ptr(py, ptr) })
        } else {
            Err(::PyDowncastError(py, None))
        }
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub fn cast_from_borrowed_ptr<T>(py: Python<'p>, ptr: *mut ffi::PyObject)
                                     -> Result<pptr<'p>, ::PyDowncastError<'p>>
        where T: PyTypeInfo
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(ptr, T::type_object()) != 0 };

        if checked {
            Ok( unsafe { pptr::from_borrowed_ptr(py, ptr) })
        } else {
            Err(::PyDowncastError(py, None))
        }
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub unsafe fn cast_from_owned_ptr_or_panic<T>(py: Python<'p>,
                                                  ptr: *mut ffi::PyObject) -> pptr<'p>
        where T: PyTypeInfo
    {
        if ffi::PyObject_TypeCheck(ptr, T::type_object()) != 0 {
            pptr::from_owned_ptr(py, ptr)
        } else {
            ::err::panic_after_error();
        }
    }

    #[inline]
    pub fn cast_from_owned_nullptr<T>(py: Python<'p>, ptr: *mut ffi::PyObject)
                                      -> PyResult<pptr<'p>>
        where T: PyTypeInfo
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            pptr::cast_from_owned_ptr::<T>(py, ptr).map_err(|e| e.into())
        }
    }
}

impl<'p> ToPythonPointer for pptr<'p> {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.1
    }
}

/// Dropping a `pptr` instance decrements the reference count on the object by 1.
impl<'p> Drop for pptr<'p> {

    fn drop(&mut self) {
        unsafe {
            println!("drop pptr: {:?} {} {:?}",
                     self.1, ffi::Py_REFCNT(self.1), &self as *const _);
        }
        unsafe { ffi::Py_DECREF(self.1); }
    }
}
