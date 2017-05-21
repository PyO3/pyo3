// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::marker::PhantomData;
use std::os::raw::c_int;
use std::ops::Deref;

use ffi;
use err::PyResult;
use python::Python;
use class::{BaseObject, PyTypeObject};

use objects::{PyObject, PyType};


pub struct PyPtr<T> {
    inner: *mut ffi::PyObject,
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


pub struct Py<'p, T> {
    inner: *mut ffi::PyObject,
    _t: PhantomData<T>,
    _py: PhantomData<Python<'p>>,
}

impl<'p, T> Py<'p, T> where T: BaseObject + PyTypeObject {

    fn new(py: Python<'p>, value: T) -> PyResult<Py<'p, T>> {
        unsafe {
            let obj = try!(Py::<T>::alloc(py, value));

            Ok(Py{inner: obj, _t: PhantomData, _py: PhantomData})
        }
    }

    unsafe fn alloc(py: Python, value: T) -> PyResult<*mut ffi::PyObject>
    {
        let ty = py.get_type::<T>();
        let obj = try!(PyObject::alloc(py, &ty, ()));

        let align = std::mem::align_of::<T>();
        let bs = <T as BaseObject>::size();

        // round base_size up to next multiple of align
        let offset = (bs + align - 1) / align * align;

        let ptr = (obj.as_ptr() as *mut u8).offset(offset as isize) as *mut T;
        std::ptr::write(ptr, value);

        Ok(obj.as_ptr())
    }

    unsafe fn dealloc(py: Python, obj: *mut ffi::PyObject) {
        let align = std::mem::align_of::<T>();
        let bs = <T as BaseObject>::size();

        // round base_size up to next multiple of align
        let offset = (bs + align - 1) / align * align;

        let ptr = (obj as *mut u8).offset(offset as isize) as *mut T;
        std::ptr::drop_in_place(ptr);

        PyObject::dealloc(py, obj)
    }

    #[inline]
    pub fn as_ref(&self) -> &'p T {
        let align = std::mem::align_of::<T>();
        let bs = <T as BaseObject>::size();

        // round base_size up to next multiple of align
        let offset = (bs + align - 1) / align * align;

        unsafe {
            let ptr = (self.inner as *mut u8).offset(offset as isize) as *mut T;
            ptr.as_ref().unwrap()
        }
    }

    #[inline]
    pub fn as_mut(&self) -> &'p mut T {
        let align = std::mem::align_of::<T>();
        let bs = <T as BaseObject>::size();

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
}

impl<'p, T> Deref for Py<'p, T> where T: BaseObject {
    type Target = T;

    fn deref(&self) -> &T {
        let align = std::mem::align_of::<T>();
        let bs = <T as BaseObject>::size();

        // round base_size up to next multiple of align
        let offset = (bs + align - 1) / align * align;

        unsafe {
            let ptr = (self.inner as *mut u8).offset(offset as isize) as *mut T;
            ptr.as_ref().unwrap()
        }
    }
}
