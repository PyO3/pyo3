// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::err::{PyErr, PyResult};
use crate::gil;
use crate::object::PyObject;
use crate::objectprotocol::ObjectProtocol;
use crate::type_object::{PyBorrowFlagLayout, PyDowncastImpl};
use crate::{
    ffi, AsPyPointer, FromPyObject, IntoPy, IntoPyPointer, PyAny, PyCell, PyClass,
    PyClassInitializer, PyRef, PyRefMut, PyTypeInfo, Python, ToPyObject,
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

impl<T> Py<T> {
    /// Create a new instance `Py<T>`.
    ///
    /// This method is **soft-duplicated** since PyO3 0.9.0.
    /// Use [`PyCell::new`](../pycell/struct.PyCell.html#method.new) and
    /// `Py::from` instead.
    pub fn new(py: Python, value: impl Into<PyClassInitializer<T>>) -> PyResult<Py<T>>
    where
        T: PyClass,
        T::BaseLayout: PyBorrowFlagLayout<T::BaseType>,
    {
        let initializer = value.into();
        let obj = unsafe { initializer.create_cell(py)? };
        let ob = unsafe { Py::from_owned_ptr(obj as _) };
        Ok(ob)
    }

    /// Creates a `Py<T>` instance for the given FFI pointer.
    ///
    /// This moves ownership over the pointer into the `Py<T>`.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(ptr: *mut ffi::PyObject) -> Py<T> {
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
    pub unsafe fn from_owned_ptr_or_panic(ptr: *mut ffi::PyObject) -> Py<T> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Py(nonnull_ptr, PhantomData),
            None => {
                crate::err::panic_after_error();
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
    pub unsafe fn from_borrowed_ptr(ptr: *mut ffi::PyObject) -> Py<T> {
        debug_assert!(
            !ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
            format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr))
        );
        ffi::Py_INCREF(ptr);
        Py(NonNull::new_unchecked(ptr), PhantomData)
    }

    /// Gets the reference count of the `ffi::PyObject` pointer.
    #[inline]
    pub fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.0.as_ptr()) }
    }

    /// Clones self by calling `Py_INCREF()` on the ptr.
    #[inline]
    pub fn clone_ref(&self, _py: Python) -> Py<T> {
        unsafe { Py::from_borrowed_ptr(self.0.as_ptr()) }
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
/// use pyo3::ObjectProtocol;
/// let obj: PyObject = {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     py.eval("[]", None, None).unwrap().to_object(py)
/// };
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// assert_eq!(obj.as_ref(py).len().unwrap(), 0);  // PyAny implements ObjectProtocol
/// ```
///
/// `Py<T>::as_ref` returns `&PyDict`, `&PyList` or so for native types, and `&PyCell<T>`
/// for `#[pyclass]`.
/// ```
/// # use pyo3::prelude::*;
/// use pyo3::ObjectProtocol;
/// let obj: PyObject = {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     py.eval("[]", None, None).unwrap().to_object(py)
/// };
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// assert_eq!(obj.as_ref(py).len().unwrap(), 0);  // PyAny implements ObjectProtocol
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
        let any = self as *const Py<T> as *const PyAny;
        unsafe { PyDowncastImpl::unchecked_downcast(&*any) }
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
        unsafe { Py::from_borrowed_ptr(obj.as_ptr()) }
    }
}

// `&PyCell<T>` can be converted to `Py<T>`
impl<'a, T> std::convert::From<&PyCell<T>> for Py<T>
where
    T: PyClass,
{
    fn from(cell: &PyCell<T>) -> Self {
        unsafe { Py::from_borrowed_ptr(cell.as_ptr()) }
    }
}

impl<'a, T> std::convert::From<PyRef<'a, T>> for Py<T>
where
    T: PyClass,
{
    fn from(pyref: PyRef<'a, T>) -> Self {
        unsafe { Py::from_borrowed_ptr(pyref.as_ptr()) }
    }
}

impl<'a, T> std::convert::From<PyRefMut<'a, T>> for Py<T>
where
    T: PyClass,
{
    fn from(pyref: PyRefMut<'a, T>) -> Self {
        unsafe { Py::from_borrowed_ptr(pyref.as_ptr()) }
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
/// Many methods want to take anything that can be converted into a Python object. This type
/// takes care of both types types that are already Python object (i.e. implement
/// [AsPyPointer]) and those that don't (i.e. [ToPyObject] types).
/// For the [AsPyPointer] types, we just use the borrowed pointer, which is a lot faster
/// and simpler than creating a new extra object. The remaning [ToPyObject] types are
/// converted to Python objects, the owned pointer is stored and decref'd on drop.
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
/// out the correct lifetime annotation to make the compiler happy.
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

/// Helper trait to choose the right implementation for [ManagedPyRef].
pub trait ManagedPyRefDispatch: ToPyObject {
    /// Optionally converts into a Python object and stores the pointer to the python heap.
    fn to_managed_py_ref<'p>(&self, py: Python<'p>) -> ManagedPyRef<'p, Self>;

    /// Dispatch over a xdecref and a noop drop impl
    fn drop_impl(borrowed: &mut ManagedPyRef<Self>);
}

/// Case 1: It's a Rust object which still needs to be converted to a Python object.
/// This means we're storing the owned pointer that into_ptr() has given us
/// and therefore need to xdecref when we're done.
///
/// Note that the actual implementations are part of the trait declaration to avoid
/// a specialization error
impl<T: ToPyObject + ?Sized> ManagedPyRefDispatch for T {
    /// Contains the case 1 impl (with to_object) to avoid a specialization error
    default fn to_managed_py_ref<'p>(&self, py: Python<'p>) -> ManagedPyRef<'p, Self> {
        ManagedPyRef {
            data: self.to_object(py).into_ptr(),
            data_type: PhantomData,
            _py: py,
        }
    }

    /// Contains the case 1 impl (decref) to avoid a specialization error
    default fn drop_impl(borrowed: &mut ManagedPyRef<Self>) {
        unsafe { ffi::Py_DECREF(borrowed.data) };
    }
}

/// Case 2: It's an object on the Python heap, we're just storing a borrowed pointer.
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
    use super::{ManagedPyRef, Py};
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
