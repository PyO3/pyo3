// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::rc::Rc;
use std::marker::PhantomData;

use ffi;
use pythonrun;
use err::{PyResult, PyErr};
use object::PyObject;
use objects::PyObjectRef;
use objectprotocol::ObjectProtocol;
use conversion::{ToPyObject, IntoPyObject, FromPyObject};
use python::{Python, IntoPyPointer, ToPyPointer};
use typeob::{PyTypeInfo, PyTypeObject};


pub struct PyToken(PhantomData<Rc<()>>);

impl PyToken {

    pub(crate) fn new() -> PyToken {
        PyToken(PhantomData)
    }

    #[inline(always)]
    #[cfg_attr(feature = "cargo-clippy", allow(inline_always))]
    pub fn py(&self) -> Python {
        unsafe { Python::assume_gil_acquired() }
    }
}

/// Any instance that is managed Python can have access to `gil`.
pub trait PyObjectWithToken: Sized {
    fn py(&self) -> Python;
}

#[doc(hidden)]
pub trait PyNativeType: PyObjectWithToken {}


/// Trait implements object reference extraction from python managed pointer.
pub trait AsPyRef<T>: Sized {

    /// Return reference to object.
    fn as_ref(&self, py: Python) -> &T;

    /// Return mutable reference to object.
    #[cfg_attr(feature = "cargo-clippy", allow(mut_from_ref))]
    fn as_mut(&self, py: Python) -> &mut T;

    /// Acquire python gil and call closure with object reference.
    fn with<F, R>(&self, f: F) -> R where F: FnOnce(Python, &T) -> R
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        f(py, self.as_ref(py))
    }

    /// Acquire python gil and call closure with mutable object reference.
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
        py.xdecref(self.into_ptr());
        result
    }

    fn into_mut_py<F, R>(self, f: F) -> R
        where Self: IntoPyPointer, F: FnOnce(Python, &mut T) -> R
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let result = f(py, self.as_mut(py));
        py.xdecref(self.into_ptr());
        result
    }
}

/// Safe wrapper around unsafe `*mut ffi::PyObject` pointer with specified type information.
#[derive(Debug)]
pub struct Py<T>(pub *mut ffi::PyObject, std::marker::PhantomData<T>);

// `Py<T>` is thread-safe, because any python related operations require a Python<'p> token.
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

    /// Creates a `Py<T>` instance for the given FFI pointer.
    /// Panics if the pointer is `null`.
    /// Undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_owned_ptr_or_panic(ptr: *mut ffi::PyObject) -> Py<T>
    {
        if ptr.is_null() {
            ::err::panic_after_error();
        } else {
            Py::from_owned_ptr(ptr)
        }
    }

    /// Construct `Py<T>` from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is `null`.
    /// Unsafe because the pointer might be invalid.
    pub unsafe fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<Py<T>>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(Py::from_owned_ptr(ptr))
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

    /// Clone self, Calls Py_INCREF() on the ptr.
    #[inline]
    pub fn clone_ref(&self, _py: Python) -> Py<T> {
        unsafe { Py::from_borrowed_ptr(self.0) }
    }
}


impl<T> Py<T> where T: PyTypeInfo,
{
    /// Create new instance of T and move it under python management
    /// Returns `Py<T>`.
    pub fn new<F>(py: Python, f: F) -> PyResult<Py<T>>
        where F: FnOnce(::PyToken) -> T,
              T: PyTypeObject + PyTypeInfo
    {
        let ob = <T as PyTypeObject>::create(py)?;
        ob.init(f)?;

        let ob = unsafe {
            Py::from_owned_ptr(ob.into_ptr())
        };
        Ok(ob)
    }

    /// Create new instance of `T` and move it under python management.
    /// Returns references to `T`
    pub fn new_ref<F>(py: Python, f: F) -> PyResult<&T>
        where F: FnOnce(::PyToken) -> T,
              T: PyTypeObject + PyTypeInfo
    {
        let ob = <T as PyTypeObject>::create(py)?;
        ob.init(f)?;

        unsafe {
            Ok(py.from_owned_ptr(ob.into_ptr()))
        }
    }

    /// Create new instance of `T` and move it under python management.
    /// Returns mutable references to `T`
    pub fn new_mut<F>(py: Python, f: F) -> PyResult<&mut T>
        where F: FnOnce(::PyToken) -> T,
              T: PyTypeObject + PyTypeInfo
    {
        let ob = <T as PyTypeObject>::create(py)?;
        ob.init(f)?;

        unsafe {
            Ok(py.mut_from_owned_ptr(ob.into_ptr()))
        }
    }

}

impl<T> AsPyRef<T> for Py<T> where T: PyTypeInfo {

    #[inline]
    default fn as_ref(&self, _py: Python) -> &T {
        unsafe {
            let ptr = (self.as_ptr() as *mut u8).offset(T::OFFSET) as *mut T;
            ptr.as_ref().unwrap()
        }
    }
    #[inline]
    default fn as_mut(&self, _py: Python) -> &mut T {
        unsafe {
            let ptr = (self.as_ptr() as *mut u8).offset(T::OFFSET) as *mut T;
            ptr.as_mut().unwrap()
        }
    }
}

impl<T> AsPyRef<T> for Py<T> where T: PyTypeInfo + PyNativeType {

    #[inline]
    fn as_ref(&self, _py: Python) -> &T {
        unsafe {std::mem::transmute(self)}
    }
    #[inline]
    fn as_mut(&self, _py: Python) -> &mut T {
        unsafe { &mut *(self as *const _ as *mut T) }
    }
}

impl<T> ToPyObject for Py<T> {
    /// Converts `Py` instance -> PyObject.
    fn to_object(&self, py: Python) -> PyObject {
        unsafe {
            PyObject::from_borrowed_ptr(py, self.as_ptr())
        }
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
        unsafe { pythonrun::register_pointer(self.0); }
    }
}


impl<T> std::convert::From<Py<T>> for PyObject {
    #[inline]
    fn from(ob: Py<T>) -> Self {
        unsafe {std::mem::transmute(ob)}
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

impl<'a, T> FromPyObject<'a> for Py<T>
    where T: ToPyPointer, &'a T: 'a + FromPyObject<'a>
{
    /// Extracts `Self` from the source `PyObject`.
    fn extract(ob: &'a PyObjectRef) -> PyResult<Self>
    {
        unsafe {
            ob.extract::<&T>().map(|val| Py::from_borrowed_ptr(val.as_ptr()))
        }
    }
}
