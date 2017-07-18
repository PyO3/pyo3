// Copyright (c) 2017-present PyO3 Project and Contributors

use std;

use ffi;
use pythonrun;
use err::{PyErr, PyResult, PyDowncastError};
use instance::{AsPyRef, PyObjectWithToken};
use objects::{PyObjectRef, PyDict};
use conversion::{ToPyObject, IntoPyObject, IntoPyTuple, FromPyObject};
use python::{Python, ToPyPointer, IntoPyPointer};


/// Safe wrapper around unsafe `*mut ffi::PyObject` pointer.
#[derive(Debug)]
pub struct PyObject(*mut ffi::PyObject);

// `PyObject` is thread-safe, because any python related operations require a Python<'p> token.
unsafe impl Send for PyObject {}
unsafe impl Sync for PyObject {}


impl PyObject {
    /// Creates a `PyObject` instance for the given FFI pointer.
    /// This moves ownership over the pointer into the `PyObject`.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyObject {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
                      format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr)));
        PyObject(ptr)
    }

    /// Creates a `PyObject` instance for the given FFI pointer.
    /// Panics if the pointer is `null`.
    /// Undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_owned_ptr_or_panic(py: Python, ptr: *mut ffi::PyObject) -> PyObject
    {
        if ptr.is_null() {
            ::err::panic_after_error();
        } else {
            PyObject::from_owned_ptr(py, ptr)
        }
    }

    /// Construct `PyObject` from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is `null`.
    pub fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<PyObject>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(unsafe{
                PyObject::from_owned_ptr(py, ptr)
            })
        }
    }

    /// Construct `PyObject` from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `None` if the pointer is `null`.
    pub fn from_owned_ptr_or_opt(py: Python, ptr: *mut ffi::PyObject) -> Option<PyObject>
    {
        if ptr.is_null() {
            None
        } else {
            Some(unsafe{
                PyObject::from_owned_ptr(py, ptr)
            })
        }
    }

    /// Creates a `PyObject` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyObject {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
                      format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr)));
        ffi::Py_INCREF(ptr);
        PyObject(ptr)
    }

    /// Creates a `PyObject` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Returns `Err(PyErr)` if the pointer is `null`.
    #[inline]
    pub fn from_borrowed_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<PyObject>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(unsafe{PyObject::from_borrowed_ptr(py, ptr)})
        }
    }

    /// Creates a `PyObject` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Returns `None` if the pointer is `null`.
    #[inline]
    pub fn from_borrowed_ptr_or_opt(py: Python, ptr: *mut ffi::PyObject) -> Option<PyObject>
    {
        if ptr.is_null() {
            None
        } else {
            Some(unsafe{PyObject::from_borrowed_ptr(py, ptr)})
        }
    }

    /// Transmutes a slice of owned FFI pointers to `&[PyObject]`.
    /// Undefined behavior if any pointer in the slice is NULL or invalid.
    #[inline]
    pub unsafe fn borrow_from_owned_ptr_slice(ptr: &[*mut ffi::PyObject])
                                                  -> &[PyObject] {
        std::mem::transmute(ptr)
    }

    /// Gets the reference count of the ffi::PyObject pointer.
    #[inline]
    pub fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.0) }
    }

    /// Clone self, Calls Py_INCREF() on the ptr.
    pub fn clone_ref(&self, py: Python) -> Self {
        unsafe {
            PyObject::from_borrowed_ptr(py, self.as_ptr())
        }
    }

    /// Returns whether the object is considered to be None.
    /// This is equivalent to the Python expression: 'is None'
    #[inline]
    pub fn is_none(&self) -> bool {
        unsafe { ffi::Py_None() == self.as_ptr() }
    }

    /// Returns whether the object is considered to be true.
    /// This is equivalent to the Python expression: 'not not self'
    #[inline]
    pub fn is_true(&self, py: Python) -> PyResult<bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(py))
        } else {
            Ok(v != 0)
        }
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

    /// Calls the object.
    /// This is equivalent to the Python expression: 'self(*args, **kwargs)'
    #[inline]
    pub fn call<A>(&self, py: Python, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: IntoPyTuple
    {
        let t = args.into_tuple(py);
        let result = unsafe {
            PyObject::from_borrowed_ptr_or_err(
                py, ffi::PyObject_Call(self.as_ptr(), t.as_ptr(), kwargs.as_ptr()))
        };
        py.release(t);
        result
    }

    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    #[inline]
    pub fn getattr<N>(&self, py: Python, attr_name: N) -> PyResult<PyObject>
        where N: ToPyObject
    {
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            PyObject::from_owned_ptr_or_err(
                py, ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args, **kwargs)'
    #[inline]
    pub fn call_method<A>(&self, py: Python,
                          name: &str, args: A,
                          kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: IntoPyTuple
    {
        name.with_borrowed_ptr(py, |name| unsafe {
            let t = args.into_tuple(py);
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            let result = PyObject::from_borrowed_ptr_or_err(
                py, ffi::PyObject_Call(ptr, t.as_ptr(), kwargs.as_ptr()));
            py.release(t);
            result
        })
    }
}

impl AsPyRef<PyObjectRef> for PyObject {

    #[inline]
    fn as_ref(&self, _py: Python) -> &PyObjectRef {
        unsafe {std::mem::transmute(self)}
    }
    #[inline]
    fn as_mut(&self, _py: Python) -> &mut PyObjectRef {
        unsafe {&mut *(self as *const _ as *mut PyObjectRef)}
    }
}

impl ToPyObject for PyObject
{
    #[inline]
    fn to_object<'p>(&self, py: Python<'p>) -> PyObject {
        unsafe {PyObject::from_borrowed_ptr(py, self.as_ptr())}
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        f(self.as_ptr())
    }
}

impl ToPyPointer for PyObject {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0
    }
}

impl<'a> ToPyPointer for &'a PyObject {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0
    }
}

impl IntoPyPointer for PyObject {
    /// Gets the underlying FFI pointer, returns a owned pointer.
    #[inline]
    #[must_use]
    fn into_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }
}

impl PartialEq for PyObject {
    #[inline]
    fn eq(&self, o: &PyObject) -> bool {
        self.0 == o.0
    }
}

impl IntoPyObject for PyObject
{
    #[inline]
    fn into_object(self, _py: Python) -> PyObject {
        self
    }
}

impl<'a> FromPyObject<'a> for PyObject
{
    #[inline]
    /// Extracts `Self` from the source `PyObject`.
    fn extract(ob: &'a PyObjectRef) -> PyResult<Self>
    {
        unsafe {
            Ok(PyObject::from_borrowed_ptr(ob.py(), ob.as_ptr()))
        }
    }
}

/// Dropping a `PyObject` instance decrements the reference count on the object by 1.
impl Drop for PyObject {

    fn drop(&mut self) {
        unsafe { pythonrun::register_pointer(self.0); }
    }
}
