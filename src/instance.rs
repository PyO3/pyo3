// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::err::{PyErr, PyResult};
use crate::gil;
use crate::object::PyObject;
use crate::pycell::{PyBorrowError, PyBorrowMutError, PyCell};
use crate::type_object::PyBorrowFlagLayout;
use crate::{
    ffi, AsPyPointer, FromPyObject, IntoPy, IntoPyPointer, PyAny, PyClass, PyClassInitializer,
    PyRef, PyRefMut, PyTypeInfo, Python, ToPyObject,
};
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

/// Types that are built into the Python interpreter.
///
/// PyO3 is designed in a way that all references to those types are bound
/// to the GIL, which is why you can get a token from all references of those
/// types.
pub unsafe trait PyNativeType: Sized {
    fn py(&self) -> Python {
        unsafe { Python::assume_gil_acquired() }
    }
    /// Cast `&PyAny` to `&Self` without no type checking.
    ///
    /// # Safety
    ///
    /// `obj` must have the same layout as `*const ffi::PyObject` and must be
    /// an instance of a type corresponding to `Self`.
    unsafe fn unchecked_downcast(obj: &PyAny) -> &Self {
        &*(obj.as_ptr() as *const Self)
    }
}

/// A Python object of known type.
///
/// Accessing this object is thread-safe, since any access to its API requires a
/// `Python<'py>` GIL token.
///
/// See [the guide](https://pyo3.rs/master/types.html) for an explanation
/// of the different Python object types.
///
/// Technically, it is a safe wrapper around `NonNull<ffi::PyObject>` with
/// specified type information.
#[derive(Debug)]
#[repr(transparent)]
pub struct Py<T>(NonNull<ffi::PyObject>, PhantomData<T>);

unsafe impl<T> Send for Py<T> {}

unsafe impl<T> Sync for Py<T> {}

impl<T> Py<T>
where
    T: PyClass,
{
    /// Create a new instance `Py<T>` of a `#[pyclass]` on the Python heap.
    pub fn new(py: Python, value: impl Into<PyClassInitializer<T>>) -> PyResult<Py<T>>
    where
        T::BaseLayout: PyBorrowFlagLayout<T::BaseType>,
    {
        let initializer = value.into();
        let obj = unsafe { initializer.create_cell(py)? };
        let ob = unsafe { Py::from_owned_ptr(py, obj as _) };
        Ok(ob)
    }

    /// Immutably borrows the value `T`. This borrow lasts untill the returned `PyRef` exists.
    ///
    /// Equivalent to `self.as_ref(py).borrow()` -
    /// see [`PyCell::borrow`](../pycell/struct.PyCell.html#method.borrow)
    ///
    /// # Panics
    /// Panics if the value is currently mutably borrowed. For a non-panicking variant, use
    /// [`try_borrow`](#method.try_borrow).
    pub fn borrow<'py>(&'py self, py: Python<'py>) -> PyRef<'py, T> {
        self.as_ref(py).borrow()
    }

    /// Mutably borrows the value `T`. This borrow lasts untill the returned `PyRefMut` exists.
    ///
    /// Equivalent to `self.as_ref(py).borrow_mut()` -
    /// see [`PyCell::borrow_mut`](../pycell/struct.PyCell.html#method.borrow_mut)
    ///
    /// # Panics
    /// Panics if the value is currently mutably borrowed. For a non-panicking variant, use
    /// [`try_borrow_mut`](#method.try_borrow_mut).
    pub fn borrow_mut<'py>(&'py self, py: Python<'py>) -> PyRefMut<'py, T> {
        self.as_ref(py).borrow_mut()
    }

    /// Immutably borrows the value `T`, returning an error if the value is currently
    /// mutably borrowed. This borrow lasts untill the returned `PyRef` exists.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    /// Equivalent to `self.as_ref(py).try_borrow()` -
    /// see [`PyCell::try_borrow`](../pycell/struct.PyCell.html#method.try_borrow)
    pub fn try_borrow<'py>(&'py self, py: Python<'py>) -> Result<PyRef<'py, T>, PyBorrowError> {
        self.as_ref(py).try_borrow()
    }

    /// Mutably borrows the value `T`, returning an error if the value is currently borrowed.
    /// This borrow lasts untill the returned `PyRefMut` exists.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    ///
    /// Equivalent to `self.as_ref(py).try_borrow_mut() -
    /// see [`PyCell::try_borrow_mut`](../pycell/struct.PyCell.html#method.try_borrow_mut)
    pub fn try_borrow_mut<'py>(
        &'py self,
        py: Python<'py>,
    ) -> Result<PyRefMut<'py, T>, PyBorrowMutError> {
        self.as_ref(py).try_borrow_mut()
    }
}

impl<T> Py<T> {
    /// Creates a `Py<T>` instance for the given FFI pointer.
    ///
    /// This moves ownership over the pointer into the `Py<T>`.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(_py: Python, ptr: *mut ffi::PyObject) -> Py<T> {
        debug_assert!(
            !ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
            format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr))
        );
        Py(NonNull::new_unchecked(ptr), PhantomData)
    }

    /// Creates a `Py<T>` instance for the given FFI pointer.
    ///
    /// Panics if the pointer is NULL.
    /// Undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_owned_ptr_or_panic(_py: Python, ptr: *mut ffi::PyObject) -> Py<T> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Py(nonnull_ptr, PhantomData),
            None => {
                crate::err::panic_after_error(_py);
            }
        }
    }

    /// Construct `Py<T>` from the result of a Python FFI call that
    ///
    /// Returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is NULL.
    /// Unsafe because the pointer might be invalid.
    pub unsafe fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<Py<T>> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Ok(Py(nonnull_ptr, PhantomData)),
            None => Err(PyErr::fetch(py)),
        }
    }

    /// Creates a `Py<T>` instance for the given Python FFI pointer.
    ///
    /// Calls `Py_INCREF()` on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(_py: Python, ptr: *mut ffi::PyObject) -> Py<T> {
        debug_assert!(
            !ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
            format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr))
        );
        ffi::Py_INCREF(ptr);
        Py(NonNull::new_unchecked(ptr), PhantomData)
    }

    /// Gets the reference count of the `ffi::PyObject` pointer.
    #[inline]
    pub fn get_refcnt(&self, _py: Python) -> isize {
        unsafe { ffi::Py_REFCNT(self.0.as_ptr()) }
    }

    /// Clones self by calling `Py_INCREF()` on the ptr.
    #[inline]
    pub fn clone_ref(&self, py: Python) -> Py<T> {
        unsafe { Py::from_borrowed_ptr(py, self.0.as_ptr()) }
    }

    /// Returns the inner pointer without decreasing the refcount.
    ///
    /// This will eventually move into its own trait.
    pub(crate) fn into_non_null(self) -> NonNull<ffi::PyObject> {
        let pointer = self.0;
        mem::forget(self);
        pointer
    }
}

/// Retrieves `&'py` types from `Py<T>` or `PyObject`.
///
/// # Examples
/// `PyObject::as_ref` returns `&PyAny`.
/// ```
/// # use pyo3::prelude::*;
/// let obj: PyObject = {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     py.eval("[]", None, None).unwrap().to_object(py)
/// };
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// assert_eq!(obj.as_ref(py).len().unwrap(), 0);
/// ```
///
/// `Py<T>::as_ref` returns `&PyDict`, `&PyList` or so for native types, and `&PyCell<T>`
/// for `#[pyclass]`.
/// ```
/// # use pyo3::prelude::*;
/// let obj: PyObject = {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     py.eval("[]", None, None).unwrap().to_object(py)
/// };
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// assert_eq!(obj.as_ref(py).len().unwrap(), 0);
/// ```
pub trait AsPyRef: Sized {
    type Target;
    /// Return reference to object.
    fn as_ref<'p>(&'p self, py: Python<'p>) -> &'p Self::Target;
}

impl<T> AsPyRef for Py<T>
where
    T: PyTypeInfo,
{
    type Target = T::AsRefTarget;
    fn as_ref<'p>(&'p self, _py: Python<'p>) -> &'p Self::Target {
        let any = self.as_ptr() as *const PyAny;
        unsafe { PyNativeType::unchecked_downcast(&*any) }
    }
}

impl<T> ToPyObject for Py<T> {
    /// Converts `Py` instance -> PyObject.
    fn to_object(&self, py: Python) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

impl<T> IntoPy<PyObject> for Py<T> {
    /// Converts a `Py` instance to `PyObject`.
    /// Consumes `self` without calling `Py_DECREF()`.
    #[inline]
    fn into_py(self, _py: Python) -> PyObject {
        unsafe { PyObject::from_not_null(self.into_non_null()) }
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
        self.into_non_null().as_ptr()
    }
}

// Native types `&T` can be converted to `Py<T>`
impl<'a, T> std::convert::From<&'a T> for Py<T>
where
    T: AsPyPointer + PyNativeType,
{
    fn from(obj: &'a T) -> Self {
        unsafe { Py::from_borrowed_ptr(obj.py(), obj.as_ptr()) }
    }
}

// `&PyCell<T>` can be converted to `Py<T>`
impl<'a, T> std::convert::From<&PyCell<T>> for Py<T>
where
    T: PyClass,
{
    fn from(cell: &PyCell<T>) -> Self {
        unsafe { Py::from_borrowed_ptr(cell.py(), cell.as_ptr()) }
    }
}

impl<'a, T> std::convert::From<PyRef<'a, T>> for Py<T>
where
    T: PyClass,
{
    fn from(pyref: PyRef<'a, T>) -> Self {
        unsafe { Py::from_borrowed_ptr(pyref.py(), pyref.as_ptr()) }
    }
}

impl<'a, T> std::convert::From<PyRefMut<'a, T>> for Py<T>
where
    T: PyClass,
{
    fn from(pyref: PyRefMut<'a, T>) -> Self {
        unsafe { Py::from_borrowed_ptr(pyref.py(), pyref.as_ptr()) }
    }
}

impl<T> PartialEq for Py<T> {
    #[inline]
    fn eq(&self, o: &Py<T>) -> bool {
        self.0 == o.0
    }
}

impl<T> Clone for Py<T> {
    fn clone(&self) -> Self {
        unsafe {
            gil::register_incref(self.0);
        }
        Self(self.0, PhantomData)
    }
}

/// Dropping a `Py` instance decrements the reference count on the object by 1.
impl<T> Drop for Py<T> {
    fn drop(&mut self) {
        unsafe {
            gil::register_decref(self.0);
        }
    }
}

impl<T> std::convert::From<Py<T>> for PyObject {
    #[inline]
    fn from(ob: Py<T>) -> Self {
        unsafe { PyObject::from_not_null(ob.into_non_null()) }
    }
}

impl<'a, T> std::convert::From<&'a T> for PyObject
where
    T: AsPyPointer + PyNativeType,
{
    fn from(ob: &'a T) -> Self {
        unsafe { Py::<T>::from_borrowed_ptr(ob.py(), ob.as_ptr()) }.into()
    }
}

impl<'a, T> std::convert::From<&'a mut T> for PyObject
where
    T: AsPyPointer + PyNativeType,
{
    fn from(ob: &'a mut T) -> Self {
        unsafe { Py::<T>::from_borrowed_ptr(ob.py(), ob.as_ptr()) }.into()
    }
}

impl<'a, T> FromPyObject<'a> for Py<T>
where
    T: PyTypeInfo,
    &'a T::AsRefTarget: FromPyObject<'a>,
    T::AsRefTarget: 'a + AsPyPointer,
{
    /// Extracts `Self` from the source `PyObject`.
    fn extract(ob: &'a PyAny) -> PyResult<Self> {
        unsafe {
            ob.extract::<&T::AsRefTarget>()
                .map(|val| Py::from_borrowed_ptr(ob.py(), val.as_ptr()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::Py;
    use crate::ffi;
    use crate::types::PyDict;
    use crate::{AsPyPointer, Python};

    #[test]
    fn py_from_dict() {
        let dict = {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let native = PyDict::new(py);
            Py::from(native)
        };
        assert_eq!(unsafe { ffi::Py_REFCNT(dict.as_ptr()) }, 1);
    }
}
