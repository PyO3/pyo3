// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::rc::Rc;
use std::marker::PhantomData;

use ffi;
use err::{PyResult, PyErr, PyDowncastError};
use objects::PyObject;
use conversion::{ToPyObject, IntoPyObject};
use python::{Python, IntoPyPointer, ToPyPointer, PyDowncastInto};
use typeob::{PyTypeInfo, PyObjectAlloc};


pub struct PyToken(PhantomData<Rc<()>>);

impl PyToken {
    pub fn token(&self) -> Python {
        unsafe { Python::assume_gil_acquired() }
    }
}

pub trait PyObjectWithToken : Sized {
    fn token(&self) -> Python;
}

pub trait AsPyRef<T> : Sized {

    fn as_ref(&self, py: Python) -> &T;

    fn as_mut(&self, py: Python) -> &mut T;

    fn with<F, R>(&self, f: F) -> R where F: FnOnce(Python, &T) -> R
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        f(py, self.as_ref(py))
    }

    fn with_mut<F, R>(&self, f: F) -> R where F: FnOnce(Python, &mut T) -> R
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        f(py, self.as_mut(py))
    }

    fn into_py<F, R>(self, f: F) -> R
        where Self: IntoPyPointer, F: FnOnce(Python, &T) -> R
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let result = f(py, self.as_ref(py));
        py.release(self);
        result
    }

    fn into_mut_py<F, R>(self, f: F) -> R
        where Self: IntoPyPointer, F: FnOnce(Python, &mut T) -> R
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let result = f(py, self.as_mut(py));
        py.release(self);
        result
    }
}

/// Wrapper around unsafe `*mut ffi::PyObject` pointer. Decrement ref counter on `Drop`
pub struct Py<T>(*mut ffi::PyObject, std::marker::PhantomData<T>);

// `PyPtr` is thread-safe, because any python related operations require a Python<'p> token.
unsafe impl<T> Send for Py<T> {}
unsafe impl<T> Sync for Py<T> {}


impl<T> Py<T> {
    /// Creates a `Py<T>` instance for the given FFI pointer.
    /// This moves ownership over the pointer into the `Py<T>`.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(ptr: *mut ffi::PyObject) -> Py<T> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
                      format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr)));
        Py(ptr, std::marker::PhantomData)
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub fn from_owned_ptr_or_panic(ptr: *mut ffi::PyObject) -> Py<T>
    {
        if ptr.is_null() {
            ::err::panic_after_error();
        } else {
            unsafe{ Py::from_owned_ptr(ptr) }
        }
    }

    /// Construct `Py<T>` from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is `null`.
    /// Unsafe because the pointer might be invalid.
    pub fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<Py<T>>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(unsafe{ Py::from_owned_ptr(ptr) })
        }
    }

    /// Creates a `Py<T>` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(ptr: *mut ffi::PyObject) -> Py<T> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
                      format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr)));
        ffi::Py_INCREF(ptr);
        Py::from_owned_ptr(ptr)
    }

    /// Gets the reference count of the ffi::PyObject pointer.
    #[inline]
    pub fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.0) }
    }

    /// Get reference to &PyObject.
    #[inline]
    pub fn as_object<'p>(&self, _py: Python<'p>) -> &PyObject {
        unsafe { std::mem::transmute(self) }
    }

    /// Clone self, Calls Py_INCREF() on the ptr.
    #[inline]
    pub fn clone_ref(&self, _py: Python) -> Py<T> {
        unsafe { Py::from_borrowed_ptr(self.0) }
    }

    /// Casts the `Py<T>` imstance to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_into<D>(self, py: Python) -> Result<D, PyDowncastError>
        where D: PyDowncastInto
    {
        <D as PyDowncastInto>::downcast_into(py, self)
    }

    /// Calls `ffi::Py_DECREF` and sets ptr to null value.
    #[inline]
    pub unsafe fn drop_ref(&mut self) {
        ffi::Py_DECREF(self.0);
        self.0 = std::ptr::null_mut();
    }
}


impl<T> Py<T> where T: PyTypeInfo,
{
    /// Create new python object and move T instance under python management
    pub fn new<F>(py: Python, f: F) -> PyResult<Py<T>>
        where F: FnOnce(::PyToken) -> T,
              T: PyObjectAlloc<T>
    {
        let ob = f(PyToken(PhantomData));

        let ob = unsafe {
            let ob = try!(<T as PyObjectAlloc<T>>::alloc(py, ob));
            Py::from_owned_ptr(ob)
        };
        Ok(ob)
    }
}

impl<T> AsPyRef<T> for Py<T> where T: PyTypeInfo {

    #[inline]
    fn as_ref(&self, _py: Python) -> &T {
        let offset = <T as PyTypeInfo>::offset();
        unsafe {
            let ptr = (self.as_ptr() as *mut u8).offset(offset) as *mut T;
            ptr.as_ref().unwrap()
        }
    }
    #[inline]
    fn as_mut(&self, _py: Python) -> &mut T {
        let offset = <T as PyTypeInfo>::offset();
        unsafe {
            let ptr = (self.as_ptr() as *mut u8).offset(offset) as *mut T;
            ptr.as_mut().unwrap()
        }
    }
}

impl<T> ToPyObject for Py<T> {
    fn to_object(&self, py: Python) -> PyObject {
        PyObject::from_borrowed_ptr(py, self.as_ptr())
    }
}

impl<T> IntoPyObject for Py<T> {
    /// Converts `Py` instance -> PyObject.
    /// Consumes `self` without calling `Py_DECREF()`
    #[inline]
    fn into_object(self, _py: Python) -> PyObject {
        unsafe { std::mem::transmute(self) }
    }
}

impl<T> ToPyPointer for Py<T> {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0
    }
}

impl<T> IntoPyPointer for Py<T> {
    /// Gets the underlying FFI pointer, returns a owned pointer.
    #[inline]
    #[must_use]
    fn into_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }
}

impl<T> PartialEq for Py<T> {
    #[inline]
    fn eq(&self, o: &Py<T>) -> bool {
        self.0 == o.0
    }
}

/// Dropping a `Py` instance decrements the reference count on the object by 1.
impl<T> Drop for Py<T> {

    fn drop(&mut self) {
        if !self.0.is_null() {
            let _gil_guard = Python::acquire_gil();
            unsafe { ffi::Py_DECREF(self.0); }
        }
    }
}


impl<T> std::convert::From<Py<T>> for PyObject {
    fn from(ob: Py<T>) -> Self {
        unsafe{std::mem::transmute(ob)}
    }
}

impl<'a, T> std::convert::From<&'a T> for Py<T>
    where T: ToPyPointer
{
    fn from(ob: &'a T) -> Self {
        unsafe { Py::from_borrowed_ptr(ob.as_ptr()) }
    }
}

impl<'a, T> std::convert::From<&'a mut T> for Py<T>
    where T: ToPyPointer
{
    fn from(ob: &'a mut T) -> Self {
        unsafe { Py::from_borrowed_ptr(ob.as_ptr()) }
    }
}

impl<'a, T> std::convert::From<&'a T> for PyObject
    where T: ToPyPointer,
{
    fn from(ob: &'a T) -> Self {
        unsafe { Py::<T>::from_borrowed_ptr(ob.as_ptr()) }.into()
    }
}

impl<'a, T> std::convert::From<&'a mut T> for PyObject
    where T: ToPyPointer,
{
    fn from(ob: &'a mut T) -> Self {
        unsafe { Py::<T>::from_borrowed_ptr(ob.as_ptr()) }.into()
    }
}
