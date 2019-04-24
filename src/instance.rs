// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::err::{PyErr, PyResult};
use crate::ffi;
use crate::gil;
use crate::instance;
use crate::object::PyObject;
use crate::objectprotocol::ObjectProtocol;
use crate::type_object::PyTypeCreate;
use crate::type_object::{PyTypeInfo, PyTypeObject};
use crate::types::PyAny;
use crate::{
    AsPyPointer, FromPyObject, FromPyPointer, IntoPyObject, IntoPyPointer, Python, ToPyObject,
};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::rc::Rc;

/// Types that are built into the python interpreter.
///
/// pyo3 is designed in a way that that all references to those types are bound to the GIL,
/// which is why you can get a token from all references of those types.
pub unsafe trait PyNativeType: Sized {
    fn py(&self) -> Python {
        unsafe { Python::assume_gil_acquired() }
    }
}

/// A special reference of type `T`. `PyRef<T>` refers a instance of T, which exists in the Python
/// heap as a part of a Python object.
///
/// We can't implement `AsPyPointer` or `ToPyObject` for `pyclass`es, because they're not Python
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

#[allow(clippy::cast_ptr_alignment)]
fn ref_to_ptr<T>(t: &T) -> *mut ffi::PyObject
where
    T: PyTypeInfo,
{
    unsafe { (t as *const _ as *mut u8).offset(-T::OFFSET) as *mut _ }
}

impl<'a, T: PyTypeInfo> PyRef<'a, T> {
    pub(crate) fn from_ref(r: &'a T) -> Self {
        PyRef(r, PhantomData)
    }
}

impl<'p, T> PyRef<'p, T>
where
    T: PyTypeInfo + PyTypeObject + PyTypeCreate,
{
    pub fn new(py: Python<'p>, value: T) -> PyResult<PyRef<T>> {
        let obj = T::create(py)?;
        obj.init(value);
        unsafe { Self::from_owned_ptr_or_err(py, obj.into_ptr()) }
    }
}

impl<'a, T: PyTypeInfo> AsPyPointer for PyRef<'a, T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        ref_to_ptr(self.0)
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

unsafe impl<'p, T> FromPyPointer<'p> for PyRef<'p, T>
where
    T: PyTypeInfo,
{
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        FromPyPointer::from_owned_ptr_or_opt(py, ptr).map(Self::from_ref)
    }
    unsafe fn from_borrowed_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        FromPyPointer::from_borrowed_ptr_or_opt(py, ptr).map(Self::from_ref)
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

impl<'p, T> PyRefMut<'p, T>
where
    T: PyTypeInfo + PyTypeObject + PyTypeCreate,
{
    pub fn new(py: Python<'p>, value: T) -> PyResult<PyRefMut<T>> {
        let obj = T::create(py)?;
        obj.init(value);
        unsafe { Self::from_owned_ptr_or_err(py, obj.into_ptr()) }
    }
}

impl<'a, T: PyTypeInfo> AsPyPointer for PyRefMut<'a, T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        ref_to_ptr(self.0)
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

unsafe impl<'p, T> FromPyPointer<'p> for PyRefMut<'p, T>
where
    T: PyTypeInfo,
{
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        FromPyPointer::from_owned_ptr_or_opt(py, ptr).map(Self::from_mut)
    }
    unsafe fn from_borrowed_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        FromPyPointer::from_borrowed_ptr_or_opt(py, ptr).map(Self::from_mut)
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

impl<T> Py<T> {
    /// Create new instance of T and move it under python management
    pub fn new(py: Python, value: T) -> PyResult<Py<T>>
    where
        T: PyTypeCreate,
    {
        let ob = T::create(py)?;
        ob.init(value);

        let ob = unsafe { Py::from_owned_ptr(ob.into_ptr()) };
        Ok(ob)
    }

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
trait AsPyRefDispatch<T: PyTypeInfo>: AsPyPointer {
    fn as_ref_dispatch(&self, _py: Python) -> &T {
        unsafe {
            let ptr = (self.as_ptr() as *mut u8).offset(T::OFFSET) as *mut T;
            ptr.as_ref().expect("Py has a null pointer")
        }
    }
    fn as_mut_dispatch(&mut self, _py: Python) -> &mut T {
        unsafe {
            let ptr = (self.as_ptr() as *mut u8).offset(T::OFFSET) as *mut T;
            ptr.as_mut().expect("Py has a null pointer")
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

impl<T> AsPyPointer for Py<T> {
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
            gil::register_pointer(self.0);
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
    T: AsPyPointer,
{
    fn from(ob: &'a T) -> Self {
        unsafe { Py::<T>::from_borrowed_ptr(ob.as_ptr()) }.into()
    }
}

impl<'a, T> std::convert::From<&'a mut T> for PyObject
where
    T: AsPyPointer,
{
    fn from(ob: &'a mut T) -> Self {
        unsafe { Py::<T>::from_borrowed_ptr(ob.as_ptr()) }.into()
    }
}

impl<'a, T> FromPyObject<'a> for Py<T>
where
    T: AsPyPointer,
    &'a T: 'a + FromPyObject<'a>,
{
    /// Extracts `Self` from the source `PyObject`.
    fn extract(ob: &'a PyAny) -> PyResult<Self> {
        unsafe {
            ob.extract::<&T>()
                .map(|val| Py::from_borrowed_ptr(val.as_ptr()))
        }
    }
}

/// Reference to a converted [ToPyObject].
///
/// Many methods want to take anything that can be converted into a python object. This type
/// takes care of both types types that are already python object (i.e. implement
/// [AsPyPointer]) and those that don't (i.e. [ToPyObject] types).
/// For the [AsPyPointer] types, we just use the borrowed pointer, which is a lot faster
/// and simpler than creating a new extra object. The remaning [ToPyObject] types are
/// converted to python objects, the owned pointer is stored and decref'd on drop.
///
/// # Example
///
/// ```
/// use pyo3::ffi;
/// use pyo3::{ToPyObject, AsPyPointer, PyNativeType, ManagedPyRef};
/// use pyo3::types::{PyDict, PyAny};
///
/// pub fn get_dict_item<'p>(dict: &'p PyDict, key: &impl ToPyObject) -> Option<&'p PyAny> {
///     let key = ManagedPyRef::from_to_pyobject(dict.py(), key);
///     unsafe {
///         dict.py().from_borrowed_ptr_or_opt(ffi::PyDict_GetItem(dict.as_ptr(), key.as_ptr()))
///     }
/// }
/// ```
#[repr(transparent)]
pub struct ManagedPyRef<'p, T: ToPyObject + ?Sized> {
    data: *mut ffi::PyObject,
    data_type: PhantomData<T>,
    _py: Python<'p>,
}

/// This should eventually be replaced with a generic `IntoPy` trait impl by figuring
/// out the correct lifetime annotation to make the compiler happy
impl<'p, T: ToPyObject> ManagedPyRef<'p, T> {
    pub fn from_to_pyobject(py: Python<'p>, to_pyobject: &T) -> Self {
        to_pyobject.to_managed_py_ref(py)
    }
}

impl<'p, T: ToPyObject> AsPyPointer for ManagedPyRef<'p, T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.data
    }
}

/// Helper trait to choose the right implementation for [ManagedPyRef]
pub trait ManagedPyRefDispatch: ToPyObject {
    /// Optionally converts into a python object and stores the pointer to the python heap.
    ///
    /// Contains the case 1 impl (with to_object) to avoid a specialization error
    fn to_managed_py_ref<'p>(&self, py: Python<'p>) -> ManagedPyRef<'p, Self> {
        ManagedPyRef {
            data: self.to_object(py).into_ptr(),
            data_type: PhantomData,
            _py: py,
        }
    }

    /// Dispatch over a xdecref and a noop drop impl
    ///
    /// Contains the case 1 impl (decref) to avoid a specialization error
    fn drop_impl(borrowed: &mut ManagedPyRef<Self>) {
        unsafe { ffi::Py_DECREF(borrowed.data) };
    }
}

/// Case 1: It's a rust object which still needs to be converted to a python object.
/// This means we're storing the owned pointer that into_ptr() has given us
/// and therefore need to xdecref when we're done.
///
/// Note that the actual implementations are part of the trait declaration to avoid
/// a specialization error
impl<T: ToPyObject + ?Sized> ManagedPyRefDispatch for T {}

/// Case 2: It's an object on the python heap, we're just storing a borrowed pointer.
/// The object we're getting is an owned pointer, it might have it's own drop impl.
impl<T: ToPyObject + AsPyPointer + ?Sized> ManagedPyRefDispatch for T {
    /// Use AsPyPointer to copy the pointer and store it as borrowed pointer
    fn to_managed_py_ref<'p>(&self, py: Python<'p>) -> ManagedPyRef<'p, Self> {
        ManagedPyRef {
            data: self.as_ptr(),
            data_type: PhantomData,
            _py: py,
        }
    }

    /// We have a borrowed pointer, so nothing to do here
    fn drop_impl(_: &mut ManagedPyRef<T>) {}
}

impl<'p, T: ToPyObject + ?Sized> Drop for ManagedPyRef<'p, T> {
    /// Uses the internal [ManagedPyRefDispatch] trait to get the right drop impl without causing
    /// a specialization error
    fn drop(&mut self) {
        ManagedPyRefDispatch::drop_impl(self);
    }
}

#[cfg(test)]
mod test {
    use crate::ffi;
    use crate::types::PyDict;
    use crate::{AsPyPointer, ManagedPyRef, Python};

    #[test]
    fn borrowed_py_ref_with_to_pointer() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let native = PyDict::new(py);
        let ref_count = unsafe { ffi::Py_REFCNT(native.as_ptr()) };
        let borrowed = ManagedPyRef::from_to_pyobject(py, native);
        assert_eq!(native.as_ptr(), borrowed.data);
        assert_eq!(ref_count, unsafe { ffi::Py_REFCNT(borrowed.data) });
        drop(borrowed);
        assert_eq!(ref_count, unsafe { ffi::Py_REFCNT(native.as_ptr()) });
    }

    #[test]
    fn borrowed_py_ref_with_to_object() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let convertible = (1, 2, 3);
        let borrowed = ManagedPyRef::from_to_pyobject(py, &convertible);
        let ptr = borrowed.data;
        // The refcountwould become 0 after dropping, which means the gc can free the pointer
        // and getting the refcount would be UB. This incref ensures that it remains 1
        unsafe {
            ffi::Py_INCREF(ptr);
        }
        assert_eq!(2, unsafe { ffi::Py_REFCNT(ptr) });
        drop(borrowed);
        assert_eq!(1, unsafe { ffi::Py_REFCNT(ptr) });
        unsafe {
            ffi::Py_DECREF(ptr);
        }
    }
}
