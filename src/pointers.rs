// Copyright (c) 2017-present PyO3 Project and Contributors

use std;

use ffi;
use err::{PyErr, PyResult, PyDowncastError};
use objects::PyObject;
use python::{Python, ToPyPointer, IntoPyPointer};


#[allow(non_camel_case_types)]
pub struct PyPtr(*mut ffi::PyObject);

// `PyPtr` is thread-safe, because any python related operations require a Python<'p> token.
unsafe impl Send for PyPtr {}
unsafe impl Sync for PyPtr {}


impl PyPtr {
    /// Creates a `PyPtr` instance for the given FFI pointer.
    /// This moves ownership over the pointer into the `PyPtr`.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(ptr: *mut ffi::PyObject) -> PyPtr {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
                      format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr)));
        PyPtr(ptr)
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub fn from_owned_ptr_or_panic(ptr: *mut ffi::PyObject) -> PyPtr
    {
        if ptr.is_null() {
            ::err::panic_after_error();
        } else {
            unsafe{
                PyPtr::from_owned_ptr(ptr)
            }
        }
    }

    /// Construct `PyPtr` from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is `null`.
    /// Unsafe because the pointer might be invalid.
    pub fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<PyPtr>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(unsafe{
                PyPtr::from_owned_ptr(ptr)
            })
        }
    }

    /// Creates a `PyPtr` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(ptr: *mut ffi::PyObject) -> PyPtr {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
                      format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr)));
        ffi::Py_INCREF(ptr);
        PyPtr::from_owned_ptr(ptr)
    }

    /// Gets the reference count of the ffi::PyObject pointer.
    #[inline]
    pub fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.0) }
    }

    /// Get reference to &PyObject<'p>
    #[inline]
    pub fn as_object<'p>(&self, _py: Python<'p>) -> &PyObject {
        unsafe { std::mem::transmute(self) }
    }

    /// Converts `PyPtr` instance -> PyObject<'p>
    /// Consumes `self` without calling `Py_DECREF()`
    #[inline]
    pub fn into_object(self, _py: Python) -> PyObject {
        unsafe { std::mem::transmute(self) }
    }

    /// Converts `PyPtr` instance -> PyObject
    /// Consumes `self` without calling `Py_DECREF()`
    #[inline]
    pub fn park(self) -> PyObject {
        unsafe { std::mem::transmute(self) }
    }

    /// Clone self, Calls Py_INCREF() on the ptr.
    #[inline]
    pub fn clone_ref(&self, _py: Python) -> PyPtr {
        unsafe { PyPtr::from_borrowed_ptr(self.0) }
    }

    /// Casts the `PyPtr` imstance to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_into<D>(self, py: Python) -> Result<D, PyDowncastError>
        where D: ::PyDowncastInto
    {
        <D as ::PyDowncastInto>::downcast_into(py, self)
    }

    #[inline]
    pub unsafe fn drop_ref(&mut self) {
        ffi::Py_DECREF(self.0);
        self.0 = std::ptr::null_mut();
    }
}

impl ToPyPointer for PyPtr {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0
    }
}

impl IntoPyPointer for PyPtr {
    /// Gets the underlying FFI pointer, returns a owned pointer.
    #[inline]
    #[must_use]
    fn into_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }
}

impl PartialEq for PyPtr {
    #[inline]
    fn eq(&self, o: &PyPtr) -> bool {
        self.0 == o.0
    }
}

//use backtrace::Backtrace;

/// Dropping a `PyPtr` instance decrements the reference count on the object by 1.
impl Drop for PyPtr {

    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                debug!("drop PyPtr: {:?} {} {:?} {:?} {:?}",
                       self.0, ffi::Py_REFCNT(self.0), &self as *const _,
                       std::ffi::CStr::from_ptr((*(*self.0).ob_type).tp_name).to_string_lossy(),
                       &self);
                //let bt = Backtrace::new();
                //let bt = Backtrace::from(Vec::from(&bt.frames()[0..15]));
                //println!("{:?}", bt);
            }
            let _gil_guard = Python::acquire_gil();
            unsafe { ffi::Py_DECREF(self.0); }
        }
    }
}
