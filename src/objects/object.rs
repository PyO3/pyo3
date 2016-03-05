// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::{mem, ptr};
use ffi;
use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, PythonObjectWithTypeObject, PythonObjectDowncastError};
use objects::PyType;
use err::PyResult;

/// Represents a reference to a Python object.
///
/// Python objects are reference counted.
/// Calling `clone_ref()` on a `PyObject` will return a new reference to the same object
/// (thus incrementing the reference count).
/// The `Drop` implementation will automatically decrement the reference count.
/// You can also call `release_ref()` to explicitly decrement the reference count.
/// This is slightly faster than relying on automatic drop, because `release_ref`
/// does not need to check whether the GIL needs to be acquired.
///
/// `PyObject` can be used with all Python objects, since all python types
/// derive from `object`. This crate also contains other, more specific types
/// that serve as references to Python objects (e.g. `PyTuple` for Python tuples, etc.).
///
/// You can convert from any Python object to `PyObject` by calling `as_object()` or `into_object()`
/// from the [PythonObject trait](trait.PythonObject.html).
/// In the other direction, you can call `cast_as()` or `cast_into()`
/// on `PyObject` to convert to more specific object types.
///
/// Most of the interesting methods are provided by the [ObjectProtocol trait](trait.ObjectProtocol.html).
#[unsafe_no_drop_flag]
#[repr(C)]
pub struct PyObject {
    // PyObject owns one reference to the *PyObject
    // ptr is not null
    #[cfg(feature="nightly")]
    ptr: ptr::Shared<ffi::PyObject>,
    #[cfg(not(feature="nightly"))]
    ptr: *mut ffi::PyObject,
}

// PyObject is thread-safe, because all operations on it require a Python<'p> token.
unsafe impl Send for PyObject {}
unsafe impl Sync for PyObject {}

/// Dropping a `PyObject` decrements the reference count on the object by 1.
impl Drop for PyObject {
    #[inline]
    fn drop(&mut self) {
        // TODO: remove `if` when #[unsafe_no_drop_flag] disappears
        if unpack_shared(self.ptr) as usize != mem::POST_DROP_USIZE {
            let _gil_guard = Python::acquire_gil();
            unsafe { ffi::Py_DECREF(unpack_shared(self.ptr)); }
        }
    }
}

#[inline]
#[cfg(feature="nightly")]
unsafe fn make_shared(ptr: *mut ffi::PyObject) -> ptr::Shared<ffi::PyObject> {
    ptr::Shared::new(ptr)
}

#[inline]
#[cfg(not(feature="nightly"))]
unsafe fn make_shared(ptr: *mut ffi::PyObject) -> *mut ffi::PyObject {
    ptr
}

#[inline]
#[cfg(feature="nightly")]
fn unpack_shared(ptr: ptr::Shared<ffi::PyObject>) -> *mut ffi::PyObject {
    *ptr
}

#[inline]
#[cfg(not(feature="nightly"))]
fn unpack_shared(ptr: *mut ffi::PyObject) -> *mut ffi::PyObject {
    ptr
}

pyobject_to_pyobject!(PyObject);

impl PythonObject for PyObject {
    #[inline]
    fn as_object(&self) -> &PyObject {
        self
    }

    #[inline]
    fn into_object(self) -> PyObject {
        self
    }

    #[inline]
    unsafe fn unchecked_downcast_from(o: PyObject) -> PyObject {
        o
    }

    #[inline]
    unsafe fn unchecked_downcast_borrow_from(o: &PyObject) -> &PyObject {
        o
    }
}

impl PythonObjectWithCheckedDowncast for PyObject {
    #[inline]
    fn downcast_from<'p>(_py: Python<'p>, obj: PyObject) -> Result<PyObject, PythonObjectDowncastError<'p>> {
        Ok(obj)
    }

    #[inline]
    fn downcast_borrow_from<'a, 'p>(_py: Python<'p>, obj: &'a PyObject) -> Result<&'a PyObject, PythonObjectDowncastError<'p>> {
        Ok(obj)
    }
}

impl PythonObjectWithTypeObject for PyObject {
    #[inline]
    fn type_object(py: Python) -> PyType {
        unsafe { PyType::from_type_ptr(py, &mut ffi::PyBaseObject_Type) }
    }
}

impl PyObject {
    /// Creates a PyObject instance for the given FFI pointer.
    /// This moves ownership over the pointer into the PyObject.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyObject {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        PyObject { ptr: make_shared(ptr) }
    }

    /// Creates a PyObject instance for the given FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(_py : Python, ptr : *mut ffi::PyObject) -> PyObject {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        ffi::Py_INCREF(ptr);
        PyObject { ptr: make_shared(ptr) }
    }

    /// Creates a PyObject instance for the given FFI pointer.
    /// This moves ownership over the pointer into the PyObject.
    /// Returns None for null pointers; undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_owned_ptr_opt(py: Python, ptr: *mut ffi::PyObject) -> Option<PyObject> {
        if ptr.is_null() {
            None
        } else {
            Some(PyObject::from_owned_ptr(py, ptr))
        }
    }

    /// Returns None for null pointers; undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr_opt(py: Python, ptr: *mut ffi::PyObject) -> Option<PyObject> {
        if ptr.is_null() {
            None
        } else {
            Some(PyObject::from_borrowed_ptr(py, ptr))
        }
    }

    /// Gets the underlying FFI pointer.
    /// Returns a borrowed pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::PyObject {
        unpack_shared(self.ptr)
    }

    /// Gets the underlying FFI pointer.
    /// Consumes `self` without calling `Py_DECREF()`, thus returning an owned pointer.
    #[inline]
    #[must_use]
    pub fn steal_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.as_ptr();
        mem::forget(self);
        ptr
    }

    /// Transmutes an FFI pointer to `&PyObject`.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn borrow_from_ptr<'a>(ptr : &'a *mut ffi::PyObject) -> &'a PyObject {
        debug_assert!(!ptr.is_null());
        mem::transmute(ptr)
    }

    /// Transmutes a slice of owned FFI pointers to `&[PyObject]`.
    /// Undefined behavior if any pointer in the slice is NULL or invalid.
    #[inline]
    pub unsafe fn borrow_from_owned_ptr_slice<'a>(ptr : &'a [*mut ffi::PyObject]) -> &'a [PyObject] {
        mem::transmute(ptr)
    }

    /// Gets the reference count of this Python object.
    #[inline]
    pub fn get_refcnt(&self, _py: Python) -> usize {
        unsafe { ffi::Py_REFCNT(self.as_ptr()) as usize }
    }

    /// Gets the Python type object for this object's type.
    #[inline]
    pub fn get_type(&self) -> &PyType {
        unsafe {
            let t : &*mut ffi::PyTypeObject = &(*self.as_ptr()).ob_type;
            let t : &*mut ffi::PyObject = mem::transmute(t);
            PyObject::borrow_from_ptr(t).unchecked_cast_as()
        }
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Causes undefined behavior if the object is not of the expected type.
    /// This is a wrapper function around `PythonObject::unchecked_downcast_from()`.
    #[inline]
    pub unsafe fn unchecked_cast_into<T>(self) -> T
        where T: PythonObject
    {
        PythonObject::unchecked_downcast_from(self)
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PythonObjectDowncastError` if the object is not of the expected type.
    /// This is a wrapper function around `PythonObjectWithCheckedDowncast::downcast_from()`.
    #[inline]
    pub fn cast_into<'p, T>(self, py: Python<'p>) -> Result<T, PythonObjectDowncastError<'p>>
        where T: PythonObjectWithCheckedDowncast
    {
        PythonObjectWithCheckedDowncast::downcast_from(py, self)
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Causes undefined behavior if the object is not of the expected type.
    /// This is a wrapper function around `PythonObject::unchecked_downcast_borrow_from()`.
    #[inline]
    pub unsafe fn unchecked_cast_as<'s, T>(&'s self) -> &'s T
        where T: PythonObject
    {
        PythonObject::unchecked_downcast_borrow_from(self)
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PythonObjectDowncastError` if the object is not of the expected type.
    /// This is a wrapper function around `PythonObjectWithCheckedDowncast::downcast_borrow_from()`.
    #[inline]
    pub fn cast_as<'s, 'p, T>(&'s self, py: Python<'p>) -> Result<&'s T, PythonObjectDowncastError<'p>>
        where T: PythonObjectWithCheckedDowncast
    {
        PythonObjectWithCheckedDowncast::downcast_borrow_from(py, self)
    }

    /// Extracts some type from the Python object.
    /// This is a wrapper function around `FromPyObject::from_py_object()`.
    #[inline]
    pub fn extract<T>(&self, py: Python) -> PyResult<T>
        where T: for<'prep> ::conversion::ExtractPyObject<'prep>
    {
        let prepared = try!(<T as ::conversion::ExtractPyObject>::prepare_extract(py, self));
        <T as ::conversion::ExtractPyObject>::extract(py, &prepared)
    }
}

/// PyObject implements the `==` operator using reference equality:
/// `obj1 == obj2` in rust is equivalent to `obj1 is obj2` in Python.
impl PartialEq for PyObject {
    #[inline]
    fn eq(&self, o : &PyObject) -> bool {
        self.as_ptr() == o.as_ptr()
    }
}

/// PyObject implements the `==` operator using reference equality:
/// `obj1 == obj2` in rust is equivalent to `obj1 is obj2` in Python.
impl Eq for PyObject { }

#[test]
fn test_sizeof() {
    // should be a static_assert, but size_of is not a compile-time const
    // these are necessary for the transmutes in this module
    assert_eq!(mem::size_of::<PyObject>(), mem::size_of::<*mut ffi::PyObject>());
    assert_eq!(mem::size_of::<PyType>(), mem::size_of::<*mut ffi::PyTypeObject>());
}

