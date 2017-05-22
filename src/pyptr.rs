// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::marker::PhantomData;
use std::os::raw::{c_int, c_void};
use std::ops::Deref;

use ffi;
use err::{PyErr, PyResult};
use python::Python;
use class::{BaseObject, PyTypeObject};

use objects::{PyObject, PyType};
use ::ToPyObject;
use class::typeob::PyTypeObjectInfo;


#[derive(Debug)]
pub struct PyPtr<T> {
    pub inner: *mut ffi::PyObject,
    _t: PhantomData<T>,
}

impl<T> PyPtr<T> {
    fn as_ref<'p>(&self, _py: Python<'p>) -> Py<'p, T> {
        Py{inner: self.inner, _t: PhantomData, _py: PhantomData}
    }

    fn into_ref<'p>(self, _py: Python<'p>) -> Py<'p, T> {
        Py{inner: self.inner, _t: PhantomData, _py: PhantomData}
    }
}

// PyObject is thread-safe, because all operations on it require a Python<'p> token.
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

/// Creates a Py instance for the given FFI pointer.
/// Calls Py_INCREF() on the ptr.
/// Undefined behavior if the pointer is NULL or invalid.
#[inline]
pub unsafe fn from_borrowed_ptr<'p, T>(_py: Python<'p>, ptr: *mut ffi::PyObject) -> Py<'p, T> {
    debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
    ffi::Py_INCREF(ptr);
    Py {inner: ptr, _t: PhantomData, _py: PhantomData}
}

pub fn new<'p, T>(py: Python<'p>, value: T) -> PyResult<Py<'p, T>> where T: BaseObject<Type=T>
{
    unsafe {
        let obj = try!(<T as BaseObject>::alloc(py, value));

        Ok(Py{inner: obj, _t: PhantomData, _py: PhantomData})
    }
}


impl<'p, T> Py<'p, T> where T: PyTypeObjectInfo
{
    /// Creates a Py instance for the given FFI pointer.
    /// This moves ownership over the pointer into the Py.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(_py: Python<'p>, ptr: *mut ffi::PyObject) -> Py<'p, T> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        Py {inner: ptr, _t: PhantomData, _py: PhantomData}
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

    /// Gets the reference count of this Py object.
    #[inline]
    pub fn get_refcnt(&self, _py: Python) -> usize {
        unsafe { ffi::Py_REFCNT(self.inner) as usize }
    }

    #[inline]
    pub fn as_ref(&self) -> &T {
        let align = std::mem::align_of::<T>();
        let bs = <T as PyTypeObjectInfo>::size();

        // round base_size up to next multiple of align
        let offset = (bs + align - 1) / align * align;

        unsafe {
            let ptr = (self.inner as *mut u8).offset(offset as isize) as *mut T;
            ptr.as_ref().unwrap()
        }
    }

    #[inline]
    pub fn as_mut(&self) -> &mut T {
        let align = std::mem::align_of::<T>();
        let bs = <T as PyTypeObjectInfo>::size();

        // round base_size up to next multiple of align
        let offset = (bs + align - 1) / align * align;

        unsafe {
            let ptr = (self.inner as *mut u8).offset(offset as isize) as *mut T;
            ptr.as_mut().unwrap()
        }
    }

    /// Creates a PyPtr instance. Calls Py_INCREF() on the ptr.
    #[inline]
    pub fn as_ptr(&self) -> PyPtr<T> {
        unsafe {
            ffi::Py_INCREF(self.inner);
        }
        PyPtr { inner: self.inner, _t: PhantomData }
    }

    /// Consumes a Py<T> instance and creates a PyPtr instance.
    /// Ownership moves over to the PyPtr<T> instance, Does not call Py_INCREF() on the ptr.
    #[inline]
    pub fn into_ptr(self) -> PyPtr<T> {
        PyPtr { inner: self.inner, _t: PhantomData }
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PythonObjectDowncastError` if the object is not of the expected type.
    /// This is a wrapper function around `PythonObjectWithCheckedDowncast::downcast_from()`.
    #[inline]
    pub fn cast<D>(&self) -> Result<Py<'p, D>, ::PythonObjectDowncastError<'p>>
        where D: ::PyWithCheckedDowncast<'p>
    {
        ::PyWithCheckedDowncast::downcast_from(self.clone())
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PythonObjectDowncastError` if the object is not of the expected type.
    /// This is a wrapper function around `PythonObjectWithCheckedDowncast::downcast_from()`.
    #[inline]
    pub fn cast_into<D>(self) -> Result<Py<'p, D>, ::PythonObjectDowncastError<'p>>
        where D: ::PyWithCheckedDowncast<'p>
    {
        ::PyWithCheckedDowncast::downcast_from(self)
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PythonObjectDowncastError` if the object is not of the expected type.
    /// This is a wrapper function around
    /// `PythonObjectWithCheckedDowncast::downcast_borrow_from()`.
    #[inline]
    pub fn cast_as<D>(&'p self) -> Result<&'p D, ::PythonObjectDowncastError<'p>>
        where D: ::PyWithCheckedDowncast<'p>
    {
        ::PyWithCheckedDowncast::downcast_borrow_from(&self)
    }

    /// Unchecked downcast from other Py<> to Self.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    pub unsafe fn unchecked_downcast_from<'a, S>(py: Py<'a, S>) -> Py<'a, T>
    {
        let res = Py {inner: py.inner, _t: PhantomData, _py: PhantomData};
        std::mem::forget(py);
        res
    }

    /// Unchecked downcast from Py<'p, S> to &'p S.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    pub unsafe fn unchecked_downcast_borrow_from<'a, S>(py: &'a Py<'a, S>) -> &'a T {
        let align = std::mem::align_of::<T>();
        let bs = <T as PyTypeObjectInfo>::size();

        // round base_size up to next multiple of align
        let offset = (bs + align - 1) / align * align;

        unsafe {
            let ptr = (py.inner as *mut u8).offset(offset as isize) as *mut T;
            ptr.as_ref().unwrap()
        }
    }

    /// Extracts some type from the Python object.
    /// This is a wrapper function around `FromPyObject::from_py_object()`.
    #[inline]
    pub fn extr<D>(&'p self) -> PyResult<D>
        where D: ::conversion::FromPyObj<'p>
    {
        ::conversion::FromPyObj::extr(&self)
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

impl<'p, T> Deref for Py<'p, T> where T: PyTypeObjectInfo + PyTypeObject + ::PythonObject {
    type Target = T;

    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<'p, T> ::PyWithCheckedDowncast<'p> for T where T: PyTypeObjectInfo
{
    #[inline]
    default fn downcast_from<S>(ob: Py<'p, S>)
                                -> Result<Py<'p, T>, ::PythonObjectDowncastError<'p>>
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(ob.inner, T::type_object()) != 0 };

        if checked {
            Ok( unsafe { Py::<T>::unchecked_downcast_from(ob) })
        } else {
            let py = unsafe {Python::assume_gil_acquired()};
            Err(::PythonObjectDowncastError(py, None))
        }
    }

    #[inline]
    default fn downcast_borrow_from<'source, S>(
        ob: &'source Py<'p, S>) -> Result<&'source T, ::PythonObjectDowncastError<'p>>
        where S: PyTypeObjectInfo
    {
        let checked = unsafe { ffi::PyObject_TypeCheck(ob.inner, T::type_object()) != 0 };

        if checked {
            Ok( unsafe { Py::<T>::unchecked_downcast_borrow_from(ob) })
        } else {
            let py = unsafe {Python::assume_gil_acquired()};
            Err(::PythonObjectDowncastError(py, None))
        }
    }
}

impl<'source, T> ::FromPyObj<'source> for &'source T
    where T: PyTypeObjectInfo
{
    #[inline]
    default fn extr<S>(py: &'source Py<'source, S>) -> PyResult<&'source T>
        where S: PyTypeObjectInfo
    {
        Ok(::PyWithCheckedDowncast::downcast_borrow_from(py)?)
        //Ok(py.cast_as::<T>()?)
    }
}

impl<'source, T> ::FromPyObj<'source> for Py<'source, T>
    where T: PyTypeObjectInfo
{
    #[inline]
    default fn extr<S>(py: &'source Py<'source, S>) -> PyResult<Py<'source, T>>
        where S: PyTypeObjectInfo
    {
        Ok(::PyWithCheckedDowncast::downcast_from(py.clone())?)
    }
}

//impl<'p, T> Deref for Py<'p, T> where T: BaseObject {
//}

impl<'p, T> ToPyObject for Py<'p, T> {
    #[inline]
    fn to_py_object(&self, py: Python) -> PyObject {
        unsafe {PyObject::from_owned_ptr(py, self.inner)}
    }

    #[inline]
    fn into_py_object(self, py: Python) -> PyObject {
        unsafe {PyObject::from_borrowed_ptr(py, self.inner)}
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        f(self.inner)
    }
}
