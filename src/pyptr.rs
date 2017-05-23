// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::convert::{AsRef, AsMut};

use ffi;
use ::ToPyObject;
use err::{self, PyResult};
use python::Python;
use class::BaseObject;
use objects::PyObject;
use class::typeob::PyTypeInfo;


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

    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::PyObject {
        self.inner
    }

    /// Gets the underlying FFI pointer.
    /// Consumes `self` without calling `Py_DECREF()`, thus returning an owned pointer.
    #[inline]
    #[must_use]
    pub fn into_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.inner;
        std::mem::forget(self);
        ptr
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

    /// Creates a Py instance for the given FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(_py: Python<'p>, ptr: *mut ffi::PyObject) -> Py<'p, T> {
        debug_assert!(!ptr.is_null() && ffi::Py_REFCNT(ptr) > 0);
        ffi::Py_INCREF(ptr);
        Py {inner: ptr, _t: PhantomData, _py: PhantomData}
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

    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::PyObject {
        self.inner
    }

    /// Gets the underlying FFI pointer.
    /// Consumes `self` without calling `Py_DECREF()`, thus returning an owned pointer.
    #[inline]
    #[must_use]
    pub fn into_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.inner;
        std::mem::forget(self);
        ptr
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
}


impl<'p, T> Py<'p, T> where T: PyTypeInfo
{
    /// Create new python object and move T instance under python management
    pub fn new(py: &Python<'p>, value: T) -> PyResult<Py<'p, T>> where T: BaseObject<Type=T>
    {
        let ob = unsafe {
            try!(<T as BaseObject>::alloc(py, value))
        };
        Ok(Py{inner: ob, _t: PhantomData, _py: PhantomData})
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub fn cast_from_borrowed(py: Python<'p>, ptr: *mut ffi::PyObject)
                              -> Result<Py<'p, T>, ::PythonObjectDowncastError<'p>>
    {
        println!("cast from borrowed {:?}", ptr);
        let checked = unsafe { ffi::PyObject_TypeCheck(ptr, T::type_object()) != 0 };

        if checked {
            Ok( unsafe { Py::from_borrowed_ptr(py, ptr) })
        } else {
            Err(::PythonObjectDowncastError(py, None))
        }
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub fn cast_from_owned(py: Python<'p>, ptr: *mut ffi::PyObject)
                           -> Result<Py<'p, T>, ::PythonObjectDowncastError<'p>>
    {
        println!("cast from owned {:?}", ptr);
        let checked = unsafe { ffi::PyObject_TypeCheck(ptr, T::type_object()) != 0 };

        if checked {
            Ok( unsafe { Py::from_owned_ptr(py, ptr) })
        } else {
            Err(::PythonObjectDowncastError(py, None))
        }
    }

    /// Cast from ffi::PyObject ptr to a concrete object.
    #[inline]
    pub fn cast_from_owned_or_panic(py: Python<'p>, ptr: *mut ffi::PyObject) -> Py<'p, T>
    {
        println!("cast from {:?}", ptr);
        let checked = unsafe { ffi::PyObject_TypeCheck(ptr, T::type_object()) != 0 };

        if checked {
            unsafe { Py::from_owned_ptr(py, ptr) }
        } else {
            ::err::panic_after_error(py);
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

    /// Unchecked downcast from Py<'p, S> to &'p S.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    pub unsafe fn unchecked_downcast_borrow_from<'a, S>(py: &'a Py<'a, S>) -> &'a T {
        let offset = <T as PyTypeInfo>::offset();

        let ptr = (py.inner as *mut u8).offset(offset) as *mut T;
        ptr.as_ref().unwrap()
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

impl<'p, T> ::PyWithCheckedDowncast<'p> for T where T: PyTypeInfo
{
    #[inline]
    default fn downcast_from<S>(ob: Py<'p, S>)
                                -> Result<Py<'p, T>, ::PythonObjectDowncastError<'p>>
    {
        println!("downcast from");
        let checked = unsafe { ffi::PyObject_TypeCheck(ob.inner, T::type_object()) != 0 };

        if checked {
            Ok( unsafe { Py::<T>::unchecked_downcast_from(ob) })
        } else {
            Err(::PythonObjectDowncastError(ob.py(), None))
        }
    }

    #[inline]
    default fn downcast_borrow_from<'source, S>(
        ob: &'source Py<'p, S>) -> Result<&'source T, ::PythonObjectDowncastError<'p>>
        where S: PyTypeInfo
    {
        println!("downcast borrow from {:?}", ob);
        let checked = unsafe { ffi::PyObject_TypeCheck(ob.inner, T::type_object()) != 0 };

        println!("downcast borrow from {:?} {:?}", checked, ob.inner);

        if checked {
            Ok( unsafe { Py::<T>::unchecked_downcast_borrow_from(ob) })
        } else {
            Err(::PythonObjectDowncastError(ob.py(), None))
        }
    }
}

impl<'source, T> ::FromPyObj<'source> for &'source T
    where T: PyTypeInfo
{
    #[inline]
    default fn extr<S>(py: &'source Py<'source, S>) -> PyResult<&'source T>
        where S: PyTypeInfo
    {
        Ok(::PyWithCheckedDowncast::downcast_borrow_from(py)?)
    }
}

impl<'source, T> ::FromPyObj<'source> for Py<'source, T>
    where T: PyTypeInfo
{
    #[inline]
    default fn extr<S>(py: &'source Py<'source, S>) -> PyResult<Py<'source, T>>
        where S: PyTypeInfo
    {
        Ok(::PyWithCheckedDowncast::downcast_from(py.clone())?)
    }
}

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

impl<'p, T> fmt::Debug for Py<'p, T> where T: PyTypeInfo {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let repr_obj  = try!(unsafe {
            err::result_cast_from_owned_ptr::<::PyString>(self.py(), ffi::PyObject_Repr(self.as_ptr()))
        }.map_err(|_| fmt::Error));
        f.write_str(&repr_obj.to_string_lossy(self.py()))
    }
}
