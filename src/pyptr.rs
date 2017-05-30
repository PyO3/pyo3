// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::convert::{AsRef, AsMut};

use ffi;
use err::{PyErr, PyResult, PyDowncastError};
use conversion::{ToPyObject, IntoPyObject};
use objects::PyObject;
use python::{Python, ToPythonPointer, IntoPythonPointer};
use token::{PyObjectMarker, PythonObjectWithGilToken};
use typeob::{PyTypeInfo, PyObjectAlloc};


pub struct PyPtr<T> {
    inner: *mut ffi::PyObject,
    _t: PhantomData<T>,
}

impl<T> PyPtr<T> {
    /// Creates a PyPtr instance for the given FFI pointer.
    /// This moves ownership over the pointer into the Py.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(ptr: *mut ffi::PyObject) -> PyPtr<T> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        PyPtr {inner: ptr, _t: PhantomData}
    }

    /// Creates a PyObject instance for the given FFI pointer.
    /// This moves ownership over the pointer into the PyObject.
    /// Returns None for null pointers; undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_owned_ptr_or_opt(_py: Python, ptr: *mut ffi::PyObject)
                                        -> Option<PyPtr<T>> {
        if ptr.is_null() {
            None
        } else {
            Some(PyPtr::from_owned_ptr(ptr))
        }
    }

    /// Construct PyPtr<PyObject> from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is `null`.
    /// Unsafe because the pointer might be invalid.
    pub unsafe fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<PyPtr<T>>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(PyPtr::from_owned_ptr(ptr))
        }
    }

    /// Cast from ffi::PyObject ptr to PyPtr pointer
    #[inline]
    pub unsafe fn from_owned_ptr_or_panic(ptr: *mut ffi::PyObject) -> PyPtr<T>
    {
        if ptr.is_null() {
            ::err::panic_after_error();
        } else {
            PyPtr::from_owned_ptr(ptr)
        }
    }

    /// Creates a PyPtr instance for the given FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    /// Caller of this method has to have valid Python object.
    #[inline]
    pub unsafe fn from_borrowed_ptr(ptr: *mut ffi::PyObject) -> PyPtr<T> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        ffi::Py_INCREF(ptr);
        PyPtr {inner: ptr, _t: PhantomData}
    }

    /// Creates a Py instance for the given FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    #[inline]
    pub unsafe fn from_borrowed_ptr_opt(_py: Python,
                                        ptr: *mut ffi::PyObject) -> Option<PyPtr<T>> {
        if ptr.is_null() {
            None
        } else {
            debug_assert!(ffi::Py_REFCNT(ptr) > 0);
            ffi::Py_INCREF(ptr);
            Some(PyPtr{inner: ptr, _t: PhantomData})
        }
    }

    pub fn as_ref<'p>(&self, py: Python<'p>) -> Py<'p, T> {
        unsafe { Py::from_borrowed_ptr(py, self.inner) }
    }

    pub fn into_ref<'p>(self, py: Python<'p>) -> Py<'p, T> {
        let p = Py{inner: self.inner, _t: PhantomData, py: py};
        std::mem::forget(self);
        p
    }

    /// Converts PyPtr<T> -> PyPtr<PyObject>
    /// Consumes `self` without calling `Py_INCREF()`
    #[inline]
    pub fn park(self) -> PyPtr<PyObjectMarker> {
        let p = PyPtr {inner: self.inner, _t: PhantomData};
        std::mem::forget(self);
        p
    }

    /// Converts PyPtr<T> -> &PyObject<'p>.
    #[inline]
    pub fn as_object<'p>(&self, _py: Python<'p>) -> &PyObject<'p> {
        unsafe {ffi::Py_INCREF(self.inner)};
        unsafe { std::mem::transmute(self) }
    }

    /// Converts PyPtr<T> -> PyObject<'p>
    /// Consumes `self` without calling `Py_DECREF()`
    #[inline]
    pub fn into_object<'p>(self, _py: Python<'p>) -> PyObject<'p> {
        unsafe { std::mem::transmute(self) }
    }

    /// Gets the reference count of this PyPtr object.
    #[inline]
    pub fn get_refcnt(&self) -> usize {
        unsafe { ffi::Py_REFCNT(self.inner) as usize }
    }

    #[inline]
    pub fn clone_ref(&self, _py: Python) -> PyPtr<T> {
        unsafe { PyPtr::from_borrowed_ptr(self.inner) }
    }

    /// Unchecked downcast from other PyPtr<S> to PyPtr<S>.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    pub unsafe fn unchecked_downcast_from<S>(py: PyPtr<S>) -> PyPtr<T>
    {
        let res = PyPtr {inner: py.inner, _t: PhantomData};
        std::mem::forget(py);
        res
    }

    /// Casts the PyPtr to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_into<'p, D>(self, py: Python<'p>) -> Result<D, PyDowncastError<'p>>
        where D: ::PyDowncastInto<'p>
    {
        match <D as ::PyDowncastInto>::downcast_from_owned_ptr(py, self.inner) {
            Ok(ptr) => {
                std::mem::forget(self);
                Ok(ptr)
            }
            Err(e) => Err(e)
        }
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

impl<T> IntoPyObject for PyPtr<T> {

    #[inline]
    fn into_object<'a>(self, _py: Python) -> PyPtr<PyObjectMarker> {
        self.park()
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
    py: Python<'p>,
}

impl<'p, T> Py<'p, T>
{
    /// Creates a Py instance for the given FFI pointer.
    /// This moves ownership over the pointer into the Py.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> Py<'p, T> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        Py {inner: ptr, _t: PhantomData, py: py}
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub unsafe fn from_owned_ptr_or_panic(py: Python<'p>, ptr: *mut ffi::PyObject) -> Py<'p, T>
    {
        if ptr.is_null() {
            ::err::panic_after_error();
        } else {
            Py::from_owned_ptr(py, ptr)
        }
    }

    #[inline]
    pub unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject)
                                        -> Option<Py<'p, T>>
    {
        if ptr.is_null() {
            None
        } else {
            Some(Py::from_owned_ptr(py, ptr))
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
    pub unsafe fn from_borrowed_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> Py<'p, T> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        ffi::Py_INCREF(ptr);
        Py {inner: ptr, _t: PhantomData, py: py}
    }

    /// Creates a Py instance for the given FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    #[inline]
    pub unsafe fn from_borrowed_ptr_opt(py: Python<'p>,
                                        ptr: *mut ffi::PyObject) -> Option<Py<'p, T>> {
        if ptr.is_null() {
            None
        } else {
            debug_assert!(ffi::Py_REFCNT(ptr) > 0);
            ffi::Py_INCREF(ptr);
            Some(Py {inner: ptr, _t: PhantomData, py: py})
        }
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
    pub fn into_object(self) -> Py<'p, PyObjectMarker> {
        let p = Py {inner: self.inner, _t: PhantomData, py: self.py};
        std::mem::forget(self);
        p
    }

    /// Converts Py<'p, T> -> PyObject<'p>. Calls Py_INCREF() on the ptr.
    #[inline]
    pub fn as_pyobject(&self) -> &PyObject<'p> {
        unsafe { std::mem::transmute(self) }
    }

    /// Converts Py<'p, T> -> PyObject<'p>
    /// Consumes `self` without calling `Py_DECREF()`
    #[inline]
    pub fn into_pyobject(self) -> PyObject<'p> {
        unsafe { std::mem::transmute(self) }
    }

    /// Unchecked downcast from other Py<S> to Py<S>.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    pub unsafe fn unchecked_downcast_from<'a, S>(ob: Py<'a, S>) -> Py<'a, T>
    {
        let res = Py {inner: ob.inner, _t: PhantomData, py: ob.py};
        std::mem::forget(ob);
        res
    }

    #[inline]
    pub fn clone_ref(&self) -> Py<'p, T> {
        unsafe { ffi::Py_INCREF(self.inner) };
        Py {inner: self.inner, _t: self._t, py: self.py}
    }

    #[inline]
    pub fn gil(&self) -> Python<'p> {
        self.py
    }
}

impl<'p, T> Py<'p, T> where T: PyTypeInfo
{
    /// Create new python object and move T instance under python management
    pub fn new(py: Python<'p>, value: T) -> PyResult<Py<'p, T>> where T: PyObjectAlloc<Type=T>
    {
        let ob = unsafe {
            try!(<T as PyObjectAlloc>::alloc(py, value))
        };
        Ok(Py{inner: ob, _t: PhantomData, py: py})
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

    #[inline]
    pub fn cast_from_owned_or_err(py: Python<'p>, ptr: *mut ffi::PyObject)
                                  -> PyResult<Py<'p, T>>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Py::cast_from_owned_ptr(py, ptr).map_err(|e| e.into())
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
            ::err::panic_after_error();
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
            Err(PyDowncastError(self.py, None))
        }
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_into<D>(self) -> Result<Py<'p, D>, PyDowncastError<'p>>
        where D: 'p + PyTypeInfo
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(self.inner, D::type_object()) != 0 };
        if checked {
            Ok( unsafe { Py::<D>::unchecked_downcast_from(self) })
        } else {
            Err(PyDowncastError(self.py, None))
        }
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_as<D>(&'p self) -> Result<&'p D, PyDowncastError<'p>>
        where D: PyTypeInfo
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(self.inner, D::type_object()) != 0 };

        if checked {
            Ok(
                unsafe {
                    let offset = <D as PyTypeInfo>::offset();
                    let ptr = (self.inner as *mut u8).offset(offset) as *mut D;
                    ptr.as_ref().unwrap() })
        } else {
            Err(PyDowncastError(self.py, None))
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
        ::conversion::FromPyObject::extract(&self.as_pyobject())
    }
}

impl<'p, T> PythonObjectWithGilToken<'p> for Py<'p, T> {
    fn gil(&self) -> Python<'p> {
        self.py
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
            println!("drop Py: {:?} {} {:?}",
                     self.inner,
                     ffi::Py_REFCNT(self.inner), &self as *const _);
        }
        unsafe { ffi::Py_DECREF(self.inner); }
    }
}

impl<'p, T> Clone for Py<'p, T> {
    fn clone(&self) -> Self {
        unsafe {
            debug_assert!(!self.inner.is_null() && ffi::Py_REFCNT(self.inner) > 0);
            ffi::Py_INCREF(self.inner);
            Py {inner: self.inner, _t: PhantomData, py: self.py}
        }
    }
}

impl<'p, T> Deref for Py<'p, T> where T: PyTypeInfo {
    type Target = T;

    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<'p, T> DerefMut for Py<'p, T> where T: PyTypeInfo {

    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
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
    where T: PyTypeInfo + ::PyDowncastFrom<'source>
{
    #[inline]
    default fn extract(py: &'source PyObject<'source>) -> PyResult<&'source T>
    {
        Ok(py.cast_as()?)
    }
}

impl<'source, T> ::FromPyObject<'source> for Py<'source, T> where T: PyTypeInfo
{
    #[inline]
    default fn extract(py: &'source PyObject<'source>) -> PyResult<Py<'source, T>>
    {
        let ptr = py.as_ptr();
        let checked = unsafe { ffi::PyObject_TypeCheck(ptr, T::type_object()) != 0 };
        if checked {
            Ok( unsafe { Py::<T>::from_borrowed_ptr(py.gil(), ptr) })
        } else {
            Err(PyDowncastError(py.gil(), None).into())
        }
    }
}

impl <'a, T> ToPyObject for Py<'a, T> {

    #[inline]
    default fn to_object<'p>(&self, _py: Python) -> PyPtr<PyObjectMarker> {
        unsafe { PyPtr::from_borrowed_ptr(self.inner) }
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        f(self.inner)
    }
}

impl<T> ToPyObject for PyPtr<T> {

    #[inline]
    default fn to_object(&self, _py: Python) -> PyPtr<PyObjectMarker> {
        unsafe { PyPtr::from_borrowed_ptr(self.inner) }
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        f(self.inner)
    }
}

impl<'p, T> IntoPyObject for Py<'p, T> {

    #[inline]
    default fn into_object(self, _py: Python) -> PyPtr<PyObjectMarker> {
        let ptr = unsafe { PyPtr::from_owned_ptr(self.inner) };
        std::mem::forget(self);
        ptr
    }
}

/// PyObject implements the `==` operator using reference equality:
/// `obj1 == obj2` in rust is equivalent to `obj1 is obj2` in Python.
impl<'p, T> PartialEq for Py<'p, T> {
    #[inline]
    fn eq(&self, o: &Py<T>) -> bool {
        self.as_ptr() == o.as_ptr()
    }
}

impl<'p, T> PartialEq for PyPtr<T> {
    #[inline]
    fn eq(&self, o: &PyPtr<T>) -> bool {
        self.as_ptr() == o.as_ptr()
    }
}
