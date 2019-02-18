// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::conversion::{FromPyObject, IntoPyObject, ToPyObject};
use crate::err::{PyErr, PyResult};
use crate::ffi;
use crate::instance;
use crate::object::PyObject;
use crate::objectprotocol::ObjectProtocol;
use crate::python::{IntoPyPointer, Python, ToPyPointer};
use crate::pythonrun;
use crate::typeob::PyTypeCreate;
use crate::typeob::{PyTypeInfo, PyTypeObject};
use crate::types::PyObjectRef;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::rc::Rc;

/// Any instance that is managed Python can have access to `gil`.
///
/// Originally, this was given to all classes with a `PyToken` field, but since `PyToken` was
/// removed this is only given to native types.
pub trait PyObjectWithGIL: Sized {
    fn py(&self) -> Python;
}

#[doc(hidden)]
pub trait PyNativeType: PyObjectWithGIL {}

/// A special reference of type `T`. `PyRef<T>` refers a instance of T, which exists in the Python
/// heap as a part of a Python object.
///
/// We can't implement `ToPyPointer` or `ToPyObject` for `pyclass`es, because they're not Python
/// objects until copied to the Python heap. So, instead of treating `&pyclass`es as Python objects,
/// we need to use special reference types `PyRef` and `PyRefMut`.
///
/// # Example
///
/// ```
/// use pyo3::prelude::*;
/// use pyo3::types::IntoPyDict;
/// #[pyclass]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
/// #[pymethods]
/// impl Point {
///     fn length(&self) -> i32 {
///         self.x * self.y
///     }
/// }
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// let obj = PyRef::new(gil.python(), Point { x: 3, y: 4 }).unwrap();
/// let d = [("p", obj)].into_py_dict(py);
/// py.run("assert p.length() == 12", None, Some(d)).unwrap();
/// ```
#[derive(Debug)]
pub struct PyRef<'a, T: PyTypeInfo>(&'a T, PhantomData<Rc<()>>);

impl<'a, T: PyTypeInfo> PyRef<'a, T> {
    pub(crate) fn from_ref(r: &'a T) -> Self {
        PyRef(r, PhantomData)
    }
}

impl<'a, T> PyRef<'a, T>
where
    T: PyTypeInfo + PyTypeObject + PyTypeCreate,
{
    pub fn new(py: Python, value: T) -> PyResult<PyRef<T>> {
        let obj = T::create(py)?;
        obj.init(value);
        let ref_ = unsafe { py.from_owned_ptr(obj.into_ptr()) };
        Ok(PyRef::from_ref(ref_))
    }
}

impl<'a, T: PyTypeInfo> ToPyPointer for PyRef<'a, T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0.as_ptr_dispatch()
    }
}

impl<'a, T: PyTypeInfo> ToPyObject for PyRef<'a, T> {
    fn to_object(&self, py: Python) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

impl<'a, T: PyTypeInfo> Deref for PyRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

/// Mutable version of [`PyRef`](struct.PyRef.html).
/// # Example
/// ```
/// use pyo3::prelude::*;
/// use pyo3::types::IntoPyDict;
/// #[pyclass]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
/// #[pymethods]
/// impl Point {
///     fn length(&self) -> i32 {
///         self.x * self.y
///     }
/// }
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// let mut obj = PyRefMut::new(gil.python(), Point { x: 3, y: 4 }).unwrap();
/// let d = vec![("p", obj.to_object(py))].into_py_dict(py);
/// obj.x = 5; obj.y = 20;
/// py.run("assert p.length() == 100", None, Some(d)).unwrap();
/// ```
#[derive(Debug)]
pub struct PyRefMut<'a, T: PyTypeInfo>(&'a mut T, PhantomData<Rc<()>>);

impl<'a, T: PyTypeInfo> PyRefMut<'a, T> {
    pub(crate) fn from_mut(t: &'a mut T) -> Self {
        PyRefMut(t, PhantomData)
    }
}

impl<'a, T> PyRefMut<'a, T>
where
    T: PyTypeInfo + PyTypeObject + PyTypeCreate,
{
    pub fn new(py: Python, value: T) -> PyResult<PyRefMut<T>> {
        let obj = T::create(py)?;
        obj.init(value);
        let ref_ = unsafe { py.mut_from_owned_ptr(obj.into_ptr()) };
        Ok(PyRefMut::from_mut(ref_))
    }
}

impl<'a, T: PyTypeInfo> ToPyPointer for PyRefMut<'a, T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        (self.0 as &T).as_ptr_dispatch()
    }
}

impl<'a, T: PyTypeInfo> ToPyObject for PyRefMut<'a, T> {
    fn to_object(&self, py: Python) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

impl<'a, T: PyTypeInfo> Deref for PyRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

impl<'a, T: PyTypeInfo> DerefMut for PyRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

impl<'a, T> From<PyRef<'a, T>> for &'a PyObjectRef
where
    T: PyTypeInfo,
{
    fn from(pref: PyRef<'a, T>) -> &'a PyObjectRef {
        unsafe { &*(pref.as_ptr() as *const PyObjectRef) }
    }
}

impl<'a, T> From<PyRefMut<'a, T>> for &'a PyObjectRef
where
    T: PyTypeInfo,
{
    fn from(pref: PyRefMut<'a, T>) -> &'a PyObjectRef {
        unsafe { &*(pref.as_ptr() as *const PyObjectRef) }
    }
}

/// Specialization workaround
trait PyRefDispatch<T: PyTypeInfo> {
    #[allow(clippy::cast_ptr_alignment)]
    fn as_ptr_dispatch(&self) -> *mut ffi::PyObject {
        unsafe { (self as *const _ as *mut u8).offset(-T::OFFSET) as *mut _ }
    }
}

impl<T: PyTypeInfo> PyRefDispatch<T> for T {}

impl<T: PyTypeInfo + PyNativeType> PyRefDispatch<T> for T {
    fn as_ptr_dispatch(&self) -> *mut ffi::PyObject {
        self as *const _ as *mut _
    }
}

/// Trait implements object reference extraction from python managed pointer.
pub trait AsPyRef<T: PyTypeInfo>: Sized {
    /// Return reference to object.
    fn as_ref(&self, py: Python) -> PyRef<T>;

    /// Return mutable reference to object.
    fn as_mut(&mut self, py: Python) -> PyRefMut<T>;

    /// Acquire python gil and call closure with object reference.
    fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Python, PyRef<T>) -> R,
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        f(py, self.as_ref(py))
    }

    /// Acquire python gil and call closure with mutable object reference.
    fn with_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(Python, PyRefMut<T>) -> R,
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        f(py, self.as_mut(py))
    }

    fn into_py<F, R>(self, f: F) -> R
    where
        Self: IntoPyPointer,
        F: FnOnce(Python, PyRef<T>) -> R,
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let result = f(py, self.as_ref(py));
        py.xdecref(self);
        result
    }

    fn into_mut_py<F, R>(mut self, f: F) -> R
    where
        Self: IntoPyPointer,
        F: FnOnce(Python, PyRefMut<T>) -> R,
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let result = f(py, self.as_mut(py));
        py.xdecref(self);
        result
    }
}

/// Safe wrapper around unsafe `*mut ffi::PyObject` pointer with specified type information.
///
/// `Py<T>` is thread-safe, because any python related operations require a Python<'p> token.
#[derive(Debug)]
#[repr(transparent)]
pub struct Py<T>(NonNull<ffi::PyObject>, std::marker::PhantomData<T>);

unsafe impl<T> Send for Py<T> {}

unsafe impl<T> Sync for Py<T> {}

impl<T> Py<T>
where
    T: PyTypeCreate + PyTypeObject,
{
    /// Create new instance of T and move it under python management
    pub fn new(py: Python, value: T) -> PyResult<Py<T>> {
        let ob = T::create(py)?;
        ob.init(value);

        let ob = unsafe { Py::from_owned_ptr(ob.into_ptr()) };
        Ok(ob)
    }
}

impl<T> Py<T> {
    /// Creates a `Py<T>` instance for the given FFI pointer.
    /// This moves ownership over the pointer into the `Py<T>`.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(ptr: *mut ffi::PyObject) -> Py<T> {
        debug_assert!(
            !ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
            format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr))
        );
        Py(NonNull::new_unchecked(ptr), std::marker::PhantomData)
    }

    /// Creates a `Py<T>` instance for the given FFI pointer.
    /// Panics if the pointer is `null`.
    /// Undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_owned_ptr_or_panic(ptr: *mut ffi::PyObject) -> Py<T> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Py(nonnull_ptr, std::marker::PhantomData),
            None => {
                crate::err::panic_after_error();
            }
        }
    }

    /// Construct `Py<T>` from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is `null`.
    /// Unsafe because the pointer might be invalid.
    pub unsafe fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<Py<T>> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Ok(Py(nonnull_ptr, std::marker::PhantomData)),
            None => Err(PyErr::fetch(py)),
        }
    }

    /// Creates a `Py<T>` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(ptr: *mut ffi::PyObject) -> Py<T> {
        debug_assert!(
            !ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
            format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr))
        );
        ffi::Py_INCREF(ptr);
        Py(NonNull::new_unchecked(ptr), std::marker::PhantomData)
    }

    /// Gets the reference count of the ffi::PyObject pointer.
    #[inline]
    pub fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.0.as_ptr()) }
    }

    /// Clone self, Calls Py_INCREF() on the ptr.
    #[inline]
    pub fn clone_ref(&self, _py: Python) -> Py<T> {
        unsafe { Py::from_borrowed_ptr(self.0.as_ptr()) }
    }

    /// Returns the inner pointer without decreasing the refcount
    ///
    /// This will eventually move into its own trait
    pub(crate) fn into_non_null(self) -> NonNull<ffi::PyObject> {
        let pointer = self.0;
        mem::forget(self);
        pointer
    }
}

/// Specialization workaround
trait AsPyRefDispatch<T: PyTypeInfo>: ToPyPointer {
    fn as_ref_dispatch(&self, _py: Python) -> &T {
        unsafe {
            let ptr = (self.as_ptr() as *mut u8).offset(T::OFFSET) as *mut T;
            ptr.as_ref().unwrap()
        }
    }
    fn as_mut_dispatch(&mut self, _py: Python) -> &mut T {
        unsafe {
            let ptr = (self.as_ptr() as *mut u8).offset(T::OFFSET) as *mut T;
            ptr.as_mut().unwrap()
        }
    }
}

impl<T: PyTypeInfo> AsPyRefDispatch<T> for Py<T> {}

impl<T: PyTypeInfo + PyNativeType> AsPyRefDispatch<T> for Py<T> {
    fn as_ref_dispatch(&self, _py: Python) -> &T {
        unsafe { &*(self as *const instance::Py<T> as *const T) }
    }
    fn as_mut_dispatch(&mut self, _py: Python) -> &mut T {
        unsafe { &mut *(self as *mut _ as *mut T) }
    }
}

impl<T> AsPyRef<T> for Py<T>
where
    T: PyTypeInfo,
{
    #[inline]
    fn as_ref(&self, py: Python) -> PyRef<T> {
        PyRef::from_ref(self.as_ref_dispatch(py))
    }
    #[inline]
    fn as_mut(&mut self, py: Python) -> PyRefMut<T> {
        PyRefMut::from_mut(self.as_mut_dispatch(py))
    }
}

impl<T> ToPyObject for Py<T> {
    /// Converts `Py` instance -> PyObject.
    fn to_object(&self, py: Python) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

impl<T> IntoPyObject for Py<T> {
    /// Converts `Py` instance -> PyObject.
    /// Consumes `self` without calling `Py_DECREF()`
    #[inline]
    fn into_object(self, py: Python) -> PyObject {
        unsafe { PyObject::from_owned_ptr(py, self.into_ptr()) }
    }
}

impl<T> ToPyPointer for Py<T> {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0.as_ptr()
    }
}

impl<T> IntoPyPointer for Py<T> {
    /// Gets the underlying FFI pointer, returns a owned pointer.
    #[inline]
    #[must_use]
    fn into_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.0.as_ptr();
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
        unsafe {
            pythonrun::register_pointer(self.0);
        }
    }
}

impl<T> std::convert::From<Py<T>> for PyObject {
    #[inline]
    fn from(ob: Py<T>) -> Self {
        unsafe { PyObject::from_not_null(ob.into_non_null()) }
    }
}

impl<'a, T> std::convert::From<PyRef<'a, T>> for Py<T>
where
    T: PyTypeInfo,
{
    fn from(ob: PyRef<'a, T>) -> Self {
        unsafe { Py::from_borrowed_ptr(ob.as_ptr()) }
    }
}

impl<'a, T> std::convert::From<PyRefMut<'a, T>> for Py<T>
where
    T: PyTypeInfo,
{
    fn from(ob: PyRefMut<'a, T>) -> Self {
        unsafe { Py::from_borrowed_ptr(ob.as_ptr()) }
    }
}

impl<'a, T> std::convert::From<&'a T> for PyObject
where
    T: ToPyPointer,
{
    fn from(ob: &'a T) -> Self {
        unsafe { Py::<T>::from_borrowed_ptr(ob.as_ptr()) }.into()
    }
}

impl<'a, T> std::convert::From<&'a mut T> for PyObject
where
    T: ToPyPointer,
{
    fn from(ob: &'a mut T) -> Self {
        unsafe { Py::<T>::from_borrowed_ptr(ob.as_ptr()) }.into()
    }
}

impl<'a, T> FromPyObject<'a> for Py<T>
where
    T: ToPyPointer,
    &'a T: 'a + FromPyObject<'a>,
{
    /// Extracts `Self` from the source `PyObject`.
    fn extract(ob: &'a PyObjectRef) -> PyResult<Self> {
        unsafe {
            ob.extract::<&T>()
                .map(|val| Py::from_borrowed_ptr(val.as_ptr()))
        }
    }
}
