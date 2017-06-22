// Copyright (c) 2017-present PyO3 Project and Contributors

use std;

use ffi;
use err::{PyErr, PyResult, PyDowncastError};
use instance::AsPyRef;
use objects::PyObject;
use conversion::{ToPyObject, IntoPyObject, FromPyObject};
use python::{Python, PyClone, ToPyPointer, IntoPyPointer};


/// Wrapper around unsafe `*mut ffi::PyObject` pointer. Decrement ref counter on `Drop`
#[derive(Debug)]
pub struct PyObjectPtr(*mut ffi::PyObject);

// `PyPtr` is thread-safe, because any python related operations require a Python<'p> token.
unsafe impl Send for PyObjectPtr {}
unsafe impl Sync for PyObjectPtr {}


impl PyObjectPtr {
    /// Creates a `PyPtr` instance for the given FFI pointer.
    /// This moves ownership over the pointer into the `PyPtr`.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyObjectPtr {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
                      format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr)));
        PyObjectPtr(ptr)
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub fn from_owned_ptr_or_panic(py: Python, ptr: *mut ffi::PyObject) -> PyObjectPtr
    {
        if ptr.is_null() {
            ::err::panic_after_error();
        } else {
            unsafe{
                PyObjectPtr::from_owned_ptr(py, ptr)
            }
        }
    }

    /// Construct `PyPtr` from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is `null`.
    /// Unsafe because the pointer might be invalid.
    pub fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<PyObjectPtr>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(unsafe{
                PyObjectPtr::from_owned_ptr(py, ptr)
            })
        }
    }

    /// Construct `PyPtr` from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `None` if the pointer is `null`.
    /// Unsafe because the pointer might be invalid.
    pub fn from_owned_ptr_or_opt(py: Python, ptr: *mut ffi::PyObject) -> Option<PyObjectPtr>
    {
        if ptr.is_null() {
            None
        } else {
            Some(unsafe{
                PyObjectPtr::from_owned_ptr(py, ptr)
            })
        }
    }

    /// Creates a `PyPtr` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyObjectPtr {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
                      format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr)));
        ffi::Py_INCREF(ptr);
        PyObjectPtr(ptr)
    }

    /// Creates a `PyPtr` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Returns `Err(PyErr)` if the pointer is `null`.
    /// Unsafe because the pointer might be invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr_or_err(py: Python, ptr: *mut ffi::PyObject)
                                           -> PyResult<PyObjectPtr>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(PyObjectPtr::from_owned_ptr(py, ptr))
        }
    }

    /// Creates a `PyPtr` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Returns `None` if the pointer is `null`.
    /// Unsafe because the pointer might be invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr_or_opt(py: Python, ptr: *mut ffi::PyObject)
                                           -> Option<PyObjectPtr>
    {
        if ptr.is_null() {
            None
        } else {
            Some(PyObjectPtr::from_owned_ptr(py, ptr))
        }
    }

    /// Transmutes a slice of owned FFI pointers to `&[Py<'p, PyObject>]`.
    /// Undefined behavior if any pointer in the slice is NULL or invalid.
    #[inline]
    pub unsafe fn borrow_from_owned_ptr_slice<'a>(ptr: &'a [*mut ffi::PyObject])
                                                  -> &'a [PyObjectPtr] {
        std::mem::transmute(ptr)
    }

    /// Gets the reference count of the ffi::PyObject pointer.
    #[inline]
    pub fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.0) }
    }

    /// Returns whether the object is considered to be None.
    /// This is equivalent to the Python expression: 'is None'
    #[inline]
    pub fn is_none(&self) -> bool {
        unsafe { ffi::Py_None() == self.as_ptr() }
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_as<D>(&self, py: Python) -> Result<&D, PyDowncastError>
        where D: ::PyDowncastFrom
    {
        <D as ::PyDowncastFrom>::downcast_from(self.as_ref(py))
    }

    /// Extracts some type from the Python object.
    /// This is a wrapper function around `FromPyObject::extract()`.
    #[inline]
    pub fn extract<'p, D>(&'p self, py: Python) -> PyResult<D> where D: FromPyObject<'p>
    {
        FromPyObject::extract(self.as_ref(py))
    }

    /// Calls `ffi::Py_DECREF` and sets ptr to null value.
    #[inline]
    pub unsafe fn drop_ref(&mut self) {
        ffi::Py_DECREF(self.0);
        self.0 = std::ptr::null_mut();
    }
}

impl AsPyRef<PyObject> for PyObjectPtr {

    #[inline]
    fn as_ref(&self, _py: Python) -> &PyObject {
        unsafe {std::mem::transmute(self)}
    }
    #[inline]
    fn as_mut(&self, _py: Python) -> &mut PyObject {
        unsafe {std::mem::transmute(self as *const _ as *mut PyObject)}
    }
}

impl ToPyObject for PyObjectPtr
{
    #[inline]
    fn to_object<'p>(&self, py: Python<'p>) -> PyObjectPtr {
        unsafe {PyObjectPtr::from_borrowed_ptr(py, self.as_ptr())}
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        f(self.as_ptr())
    }
}

impl ToPyPointer for PyObjectPtr {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0
    }
}

impl<'a> ToPyPointer for &'a PyObjectPtr {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0
    }
}

impl IntoPyPointer for PyObjectPtr {
    /// Gets the underlying FFI pointer, returns a owned pointer.
    #[inline]
    #[must_use]
    fn into_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }
}

impl PartialEq for PyObjectPtr {
    #[inline]
    fn eq(&self, o: &PyObjectPtr) -> bool {
        self.0 == o.0
    }
}

impl PyClone for PyObjectPtr {
    fn clone_ref(&self, py: Python) -> Self {
        unsafe {
            PyObjectPtr::from_borrowed_ptr(py, self.as_ptr())
        }
    }
}

impl IntoPyObject for PyObjectPtr
{
    #[inline]
    fn into_object(self, _py: Python) -> PyObjectPtr
    {
        self
    }
}

//use backtrace::Backtrace;

/// Dropping a `PyObject` instance decrements the reference count on the object by 1.
impl Drop for PyObjectPtr {

    fn drop(&mut self) {
        if !self.0.is_null() {
            //unsafe {
                //debug!("drop PyPtr: {:?} {} {:?} {:?} {:?}",
                //       self.0, ffi::Py_REFCNT(self.0), &self as *const _,
                //       std::ffi::CStr::from_ptr((*(*self.0).ob_type).tp_name).to_string_lossy(),
                //       &self);
                //let bt = Backtrace::new();
                //let bt = Backtrace::from(Vec::from(&bt.frames()[0..15]));
                //println!("{:?}", bt);
            //}
            let _gil_guard = Python::acquire_gil();
            unsafe { ffi::Py_DECREF(self.0); }
        }
    }
}
