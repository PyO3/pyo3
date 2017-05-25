// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::marker::PhantomData;
use std::ops::Deref;
use std::convert::{AsRef, AsMut};

use ffi;
use ::ToPyObject;
use err::{PyErr, PyResult, PyDowncastError};
use python::{Python, ToPythonPointer, IntoPythonPointer};
use objects::PyObject;
use typeob::{PyTypeInfo, PyObjectAlloc};


#[derive(Debug)]
pub struct PyPtr<T> {
    inner: *mut ffi::PyObject,
    _t: PhantomData<T>,
}

impl<T> PyPtr<T> {
    pub fn as_ref<'p>(&self, _py: Python<'p>) -> Py<'p, T> {
        Py{inner: self.inner, _t: PhantomData, _py: PhantomData}
    }

    pub fn into_ref<'p>(self, _py: Python<'p>) -> Py<'p, T> {
        Py{inner: self.inner, _t: PhantomData, _py: PhantomData}
    }

    /// Gets the reference count of this PyPtr object.
    #[inline]
    pub fn get_refcnt(&self) -> usize {
        unsafe { ffi::Py_REFCNT(self.inner) as usize }
    }

    #[inline]
    pub fn clone_ref(&self, _py: Python) -> PyPtr<T> {
        PyPtr{inner: self.inner.clone(), _t: PhantomData}
    }
}

impl<T> ToPythonPointer for PyPtr<T> {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.inner
    }
}

impl<T> IntoPythonPointer for PyPtr<T> {
    /// Gets the underlying FFI pointer.
    /// Consumes `self` without calling `Py_DECREF()`, thus returning an owned pointer.
    #[inline]
    #[must_use]
    fn into_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.inner;
        std::mem::forget(self);
        ptr
    }
}

// PyPtr is thread-safe, because all operations on it require a Python<'p> token.
unsafe impl<T> Send for PyPtr<T> {}
unsafe impl<T> Sync for PyPtr<T> {}

/// Dropping a `PyPtr` decrements the reference count on the object by 1.
impl<T> Drop for PyPtr<T> {
    fn drop(&mut self) {
        unsafe {
            println!("drop PyPtr: {:?} {}", self.inner, ffi::Py_REFCNT(self.inner));
        }

        let _gil_guard = Python::acquire_gil();
        unsafe { ffi::Py_DECREF(self.inner); }
    }
}


pub struct Py<'p, T> {
    pub inner: *mut ffi::PyObject,
    _t: PhantomData<T>,
    _py: PhantomData<Python<'p>>,
}

impl<'p, T> Py<'p, T>
{
    /// Creates a Py instance for the given FFI pointer.
    /// This moves ownership over the pointer into the Py.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(_py: Python<'p>, ptr: *mut ffi::PyObject) -> Py<'p, T> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        Py {inner: ptr, _t: PhantomData, _py: PhantomData}
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub unsafe fn from_owned_ptr_or_panic(py: Python<'p>, ptr: *mut ffi::PyObject) -> Py<'p, T>
    {
        if ptr.is_null() {
            ::err::panic_after_error(py);
        } else {
            Py::from_owned_ptr(py, ptr)
        }
    }

    /// Construct Py<'p, PyObj> from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is `null`.
    /// Unsafe because the pointer might be invalid.
    pub unsafe fn from_owned_ptr_or_err(py: Python<'p>, ptr: *mut ffi::PyObject)
                                        -> PyResult<Py<'p, T>>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(Py::from_owned_ptr(py, ptr))
        }
    }

    /// Creates a Py instance for the given FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(_py: Python<'p>, ptr: *mut ffi::PyObject) -> Py<'p, T> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        ffi::Py_INCREF(ptr);
        Py {inner: ptr, _t: PhantomData, _py: PhantomData}
    }

    /// Creates a Py instance for the given FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    #[inline]
    pub unsafe fn from_borrowed_ptr_opt(_py: Python<'p>,
                                        ptr: *mut ffi::PyObject) -> Option<Py<'p, T>> {
        if ptr.is_null() {
            None
        } else {
            debug_assert!(ffi::Py_REFCNT(ptr) > 0);
            ffi::Py_INCREF(ptr);
            Some(Py {inner: ptr, _t: PhantomData, _py: PhantomData})
        }
    }

    /// Retrieve Python instance, the GIL is already acquired and
    /// stays acquired for the lifetime `'p`.
    #[inline]
    pub fn py(&self) -> Python<'p> {
        unsafe { Python::assume_gil_acquired() }
    }

    /// Gets the reference count of this Py object.
    #[inline]
    pub fn get_refcnt(&self) -> usize {
        unsafe { ffi::Py_REFCNT(self.inner) as usize }
    }

    /// Creates a PyPtr instance. Calls Py_INCREF() on the ptr.
    #[inline]
    pub fn as_pptr(&self) -> PyPtr<T> {
        unsafe {
            ffi::Py_INCREF(self.inner);
        }
        PyPtr { inner: self.inner, _t: PhantomData }
    }

    /// Consumes a Py<T> instance and creates a PyPtr instance.
    /// Ownership moves over to the PyPtr<T> instance, Does not call Py_INCREF() on the ptr.
    #[inline]
    pub fn into_pptr(self) -> PyPtr<T> {
        let ptr = PyPtr { inner: self.inner, _t: PhantomData };
        std::mem::forget(self);
        ptr
    }

    /// Converts Py<'p, T> -> Py<'p, PyObject>
    /// Consumes `self` without calling `Py_DECREF()`
    #[inline]
    pub fn into_object(self) -> Py<'p, PyObject> {
        let p = Py {inner: self.inner, _t: PhantomData, _py: PhantomData};
        std::mem::forget(self);
        p
    }

    /// Unchecked downcast from other Py<S> to Self<T.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    pub unsafe fn unchecked_downcast_from<'a, S>(py: Py<'a, S>) -> Py<'a, T>
    {
        let res = Py {inner: py.inner, _t: PhantomData, _py: PhantomData};
        std::mem::forget(py);
        res
    }

    pub fn clone_ref(&self) -> Py<'p, T> {
        unsafe { ffi::Py_INCREF(self.inner) };
        Py {inner: self.inner, _t: self._t, _py: self._py}
    }
}


impl<'p, T> Py<'p, T> where T: PyTypeInfo
{
    /// Create new python object and move T instance under python management
    pub fn new(py: &Python<'p>, value: T) -> PyResult<Py<'p, T>> where T: PyObjectAlloc<Type=T>
    {
        let ob = unsafe {
            try!(<T as PyObjectAlloc>::alloc(py, value))
        };
        Ok(Py{inner: ob, _t: PhantomData, _py: PhantomData})
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub fn cast_from_borrowed_ptr(py: Python<'p>, ptr: *mut ffi::PyObject)
                                  -> Result<Py<'p, T>, ::PyDowncastError<'p>>
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(ptr, T::type_object()) != 0 };

        if checked {
            Ok( unsafe { Py::from_borrowed_ptr(py, ptr) })
        } else {
            Err(::PyDowncastError(py, None))
        }
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub fn cast_from_owned_ptr(py: Python<'p>, ptr: *mut ffi::PyObject)
                               -> Result<Py<'p, T>, ::PyDowncastError<'p>>
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(ptr, T::type_object()) != 0 };

        if checked {
            Ok( unsafe { Py::from_owned_ptr(py, ptr) })
        } else {
            Err(::PyDowncastError(py, None))
        }
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub unsafe fn cast_from_owned_ptr_or_panic(py: Python<'p>,
                                               ptr: *mut ffi::PyObject) -> Py<'p, T>
    {
        if ffi::PyObject_TypeCheck(ptr, T::type_object()) != 0 {
            Py::from_owned_ptr(py, ptr)
        } else {
            ::err::panic_after_error(py);
        }
    }

    pub fn cast_from_owned_nullptr(py: Python<'p>, ptr: *mut ffi::PyObject)
                                   -> PyResult<Py<'p, T>>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Py::cast_from_owned_ptr(py, ptr).map_err(|e| e.into())
        }
    }

    #[inline]
    pub fn as_ref(&self) -> &T {
        let offset = <T as PyTypeInfo>::offset();

        unsafe {
            let ptr = (self.inner as *mut u8).offset(offset) as *mut T;
            ptr.as_ref().unwrap()
        }
    }

    #[inline]
    pub fn as_mut(&self) -> &mut T {
        let offset = <T as PyTypeInfo>::offset();

        unsafe {
            let ptr = (self.inner as *mut u8).offset(offset) as *mut T;
            ptr.as_mut().unwrap()
        }
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast<D>(&self) -> Result<Py<'p, D>, PyDowncastError<'p>>
        where D: PyTypeInfo
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(self.inner, D::type_object()) != 0 };
        if checked {
            Ok( unsafe { Py::<D>::unchecked_downcast_from(self.clone_ref()) })
        } else {
            Err(PyDowncastError(self.py(), None))
        }
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_into<D>(self) -> Result<Py<'p, D>, PyDowncastError<'p>>
        where D: PyTypeInfo
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(self.inner, D::type_object()) != 0 };
        if checked {
            Ok( unsafe { Py::<D>::unchecked_downcast_from(self) })
        } else {
            Err(PyDowncastError(self.py(), None))
        }
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_as<'s, D>(&'s self) -> Result<&'s D, PyDowncastError<'p>>
        where D: PyTypeInfo
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(self.inner, D::type_object()) != 0 };

        if checked {
            Ok( unsafe { Py::<D>::unchecked_downcast_borrow_from(self) })
        } else {
            Err(PyDowncastError(self.py(), None))
        }
    }

    /// Unchecked downcast from Py<'p, S> to &'p S.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    pub unsafe fn unchecked_downcast_borrow_from<'a, S>(py: &'a Py<'a, S>) -> &'a T {
        let offset = <T as PyTypeInfo>::offset();

        let ptr = (py.inner as *mut u8).offset(offset) as *mut T;
        ptr.as_ref().unwrap()
    }

    /// Extracts some type from the Python object.
    /// This is a wrapper function around `FromPyObject::extract()`.
    #[inline]
    pub fn extract<D>(&'p self) -> PyResult<D> where D: ::conversion::FromPyObject<'p>
    {
        ::conversion::FromPyObject::extract(&self)
    }
}

impl<'p, T> ToPythonPointer for Py<'p, T> {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.inner
    }
}

impl<'p, T> IntoPythonPointer for Py<'p, T> {

    /// Gets the underlying FFI pointer.
    /// Consumes `self` without calling `Py_DECREF()`, thus returning an owned pointer.
    #[inline]
    #[must_use]
    fn into_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.inner;
        std::mem::forget(self);
        ptr
    }
}

/// Dropping a `Py` instance decrements the reference count on the object by 1.
impl<'p, T> Drop for Py<'p, T> {
    fn drop(&mut self) {
        unsafe {
            println!("drop Py: {:?} {}", self.inner, ffi::Py_REFCNT(self.inner));
        }
        unsafe { ffi::Py_DECREF(self.inner); }
    }
}

impl<'p, T> Clone for Py<'p, T> {
    fn clone(&self) -> Self {
        unsafe {
            debug_assert!(!self.inner.is_null() && ffi::Py_REFCNT(self.inner) > 0);
            ffi::Py_INCREF(self.inner);
            Py {inner: self.inner, _t: PhantomData, _py: PhantomData}
        }
    }
}

impl<'p, T> Deref for Py<'p, T> where T: PyTypeInfo {
    type Target = T;

    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<'p, T> AsRef<T> for Py<'p, T> where T: PyTypeInfo {
    #[inline]
    fn as_ref(&self) -> &T {
        self.as_ref()
    }
}

impl<'p, T> AsMut<T> for Py<'p, T> where T: PyTypeInfo {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        Py::<T>::as_mut(self)
    }
}

impl<'source, T> ::FromPyObject<'source> for &'source T
    where T: PyTypeInfo
{
    #[inline]
    default fn extract<S>(py: &'source Py<'source, S>) -> PyResult<&'source T>
        where S: PyTypeInfo
    {
        Ok(py.cast_as()?)
    }
}

impl<'source, T> ::FromPyObject<'source> for Py<'source, T>
    where T: PyTypeInfo
{
    #[inline]
    default fn extract<S>(py: &'source Py<'source, S>) -> PyResult<Py<'source, T>>
        where S: PyTypeInfo
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(py.inner, T::type_object()) != 0 };
        if checked {
            Ok( unsafe { Py::<T>::from_borrowed_ptr(py.py(), py.as_ptr()) })
        } else {
            Err(PyDowncastError(py.py(), None).into())
        }
    }
}

impl<'a, T> ToPyObject for Py<'a, T> {
    #[inline]
    default fn to_object<'p>(&self, py: Python<'p>) -> Py<'p, PyObject> {
        PyObject::from_owned_ptr(py, self.inner)
    }

    #[inline]
    default fn into_object<'p>(self, py: Python<'p>) -> Py<'p, PyObject> {
        PyObject::from_borrowed_ptr(py, self.inner)
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        f(self.inner)
    }
}
