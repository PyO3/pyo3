// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::conversion::{PyTryFrom, ToBorrowedObject};
use crate::err::{PyDowncastError, PyErr, PyResult};
use crate::gil;
use crate::pycell::{PyBorrowError, PyBorrowMutError, PyCell};
use crate::type_object::PyBorrowFlagLayout;
use crate::types::{PyDict, PyTuple};
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

/// A Python object of known type T.
///
/// Accessing this object is thread-safe, since any access to its API requires a `Python<'py>` GIL
/// token. There are a few different ways to use the Python object contained:
///  - [`Py::as_ref`](#method.as_ref) to borrow a GIL-bound reference to the contained object.
///  - [`Py::borrow`](#method.borrow), [`Py::try_borrow`](#method.try_borrow),
///    [`Py::borrow_mut`](#method.borrow_mut), or [`Py::try_borrow_mut`](#method.try_borrow_mut),
///    to directly access a `#[pyclass]` value (which has RefCell-like behavior, see
///    [the `PyCell` guide entry](https://pyo3.rs/master/class.html#pycell-and-interior-mutability)
///    ).
///  - Use methods directly on `Py`, such as [`Py::call`](#method.call) and
///    [`Py::call_method`](#method.call_method).
///
/// See [the guide](https://pyo3.rs/master/types.html) for an explanation
/// of the different Python object types.
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
        let obj = initializer.create_cell(py)?;
        let ob = unsafe { Py::from_owned_ptr(py, obj as _) };
        Ok(ob)
    }
}

impl<T> Py<T>
where
    T: PyTypeInfo,
{
    /// Borrows a GIL-bound reference to the contained `T`. By binding to the GIL lifetime, this
    /// allows the GIL-bound reference to not require `Python` for any of its methods.
    ///
    /// For native types, this reference is `&T`. For pyclasses, this is `&PyCell<T>`.
    ///
    /// # Examples
    /// Get access to `&PyList` from `Py<PyList>`:
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::types::PyList;
    /// # Python::with_gil(|py| {
    /// let list: Py<PyList> = PyList::empty(py).into();
    /// let list: &PyList = list.as_ref(py);
    /// assert_eq!(list.len(), 0);
    /// # });
    /// ```
    ///
    /// Get access to `&PyCell<MyClass>` from `Py<MyClass>`:
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// #[pyclass]
    /// struct MyClass { }
    /// # Python::with_gil(|py| {
    /// let my_class: Py<MyClass> = Py::new(py, MyClass { }).unwrap();
    /// let my_class_cell: &PyCell<MyClass> = my_class.as_ref(py);
    /// assert!(my_class_cell.try_borrow().is_ok());
    /// # });
    /// ```
    pub fn as_ref<'py>(&'py self, _py: Python<'py>) -> &'py T::AsRefTarget {
        let any = self.as_ptr() as *const PyAny;
        unsafe { PyNativeType::unchecked_downcast(&*any) }
    }

    /// Similar to [`as_ref`](#method.as_ref), and also consumes this `Py` and registers the
    /// Python object reference in PyO3's object storage. The reference count for the Python
    /// object will not be decreased until the GIL lifetime ends.
    ///
    /// # Example
    ///
    /// Useful when returning GIL-bound references from functions. In the snippet below, note that
    /// the `'py` lifetime of the input GIL lifetime is also given to the returned reference:
    /// ```
    /// # use pyo3::prelude::*;
    /// fn new_py_any<'py>(py: Python<'py>, value: impl IntoPy<PyObject>) -> &'py PyAny {
    ///     let obj: PyObject = value.into_py(py);
    ///
    ///     // .as_ref(py) would not be suitable here, because a reference to `obj` may not be
    ///     // returned from the function.
    ///     obj.into_ref(py)
    /// }
    /// ```
    pub fn into_ref(self, py: Python) -> &T::AsRefTarget {
        unsafe { py.from_owned_ptr(self.into_ptr()) }
    }
}

impl<T> Py<T>
where
    T: PyClass,
{
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

    /// Returns whether the object is considered to be None.
    ///
    /// This is equivalent to the Python expression `self is None`.
    pub fn is_none(&self, _py: Python) -> bool {
        unsafe { ffi::Py_None() == self.as_ptr() }
    }

    /// Returns whether the object is considered to be true.
    ///
    /// This is equivalent to the Python expression `bool(self)`.
    pub fn is_true(&self, py: Python) -> PyResult<bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(py))
        } else {
            Ok(v != 0)
        }
    }

    /// Extracts some type from the Python object.
    ///
    /// This is a wrapper function around `FromPyObject::extract()`.
    pub fn extract<'p, D>(&'p self, py: Python<'p>) -> PyResult<D>
    where
        D: FromPyObject<'p>,
    {
        FromPyObject::extract(unsafe { py.from_borrowed_ptr(self.as_ptr()) })
    }

    /// Retrieves an attribute value.
    ///
    /// This is equivalent to the Python expression `self.attr_name`.
    pub fn getattr<N>(&self, py: Python, attr_name: N) -> PyResult<PyObject>
    where
        N: ToPyObject,
    {
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            PyObject::from_owned_ptr_or_err(py, ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    /// Calls the object.
    ///
    /// This is equivalent to the Python expression `self(*args, **kwargs)`.
    pub fn call(
        &self,
        py: Python,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        let args = args.into_py(py).into_ptr();
        let kwargs = kwargs.into_ptr();
        let result = unsafe {
            PyObject::from_owned_ptr_or_err(py, ffi::PyObject_Call(self.as_ptr(), args, kwargs))
        };
        unsafe {
            ffi::Py_XDECREF(args);
            ffi::Py_XDECREF(kwargs);
        }
        result
    }

    /// Calls the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self(*args)`.
    pub fn call1(&self, py: Python, args: impl IntoPy<Py<PyTuple>>) -> PyResult<PyObject> {
        self.call(py, args, None)
    }

    /// Calls the object without arguments.
    ///
    /// This is equivalent to the Python expression `self()`.
    pub fn call0(&self, py: Python) -> PyResult<PyObject> {
        cfg_if::cfg_if! {
            // TODO: Use PyObject_CallNoArgs instead after https://bugs.python.org/issue42415.
            // Once the issue is resolved, we can enable this optimization for limited API.
            if #[cfg(all(Py_3_9, not(Py_LIMITED_API)))] {
                // Optimized path on python 3.9+
                unsafe {
                    PyObject::from_owned_ptr_or_err(py, ffi::_PyObject_CallNoArg(self.as_ptr()))
                }
            } else {
                self.call(py, (), None)
            }
        }
    }

    /// Calls a method on the object.
    ///
    /// This is equivalent to the Python expression `self.name(*args, **kwargs)`.
    pub fn call_method(
        &self,
        py: Python,
        name: &str,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        name.with_borrowed_ptr(py, |name| unsafe {
            let args = args.into_py(py).into_ptr();
            let kwargs = kwargs.into_ptr();
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            if ptr.is_null() {
                return Err(PyErr::fetch(py));
            }
            let result = PyObject::from_owned_ptr_or_err(py, ffi::PyObject_Call(ptr, args, kwargs));
            ffi::Py_DECREF(ptr);
            ffi::Py_XDECREF(args);
            ffi::Py_XDECREF(kwargs);
            result
        })
    }

    /// Calls a method on the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self.name(*args)`.
    pub fn call_method1(
        &self,
        py: Python,
        name: &str,
        args: impl IntoPy<Py<PyTuple>>,
    ) -> PyResult<PyObject> {
        self.call_method(py, name, args, None)
    }

    /// Calls a method on the object with no arguments.
    ///
    /// This is equivalent to the Python expression `self.name()`.
    pub fn call_method0(&self, py: Python, name: &str) -> PyResult<PyObject> {
        cfg_if::cfg_if! {
            if #[cfg(all(Py_3_9, not(Py_LIMITED_API)))] {
                // Optimized path on python 3.9+
                unsafe {
                    let name = name.into_py(py);
                    PyObject::from_owned_ptr_or_err(py, ffi::PyObject_CallMethodNoArgs(self.as_ptr(), name.as_ptr()))
                }
            } else {
                self.call_method(py, name, (), None)
            }
        }
    }

    /// Create a `Py<T>` instance by taking ownership of the given FFI pointer.
    ///
    /// # Safety
    /// `ptr` must be a pointer to a Python object of type T.
    ///
    /// Callers must own the object referred to by `ptr`, as this function
    /// implicitly takes ownership of that object.
    ///
    /// # Panics
    /// Panics if `ptr` is null.
    #[inline]
    pub unsafe fn from_owned_ptr(py: Python, ptr: *mut ffi::PyObject) -> Py<T> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Py(nonnull_ptr, PhantomData),
            None => crate::err::panic_after_error(py),
        }
    }

    /// Deprecated alias for [`from_owned_ptr`](#method.from_owned_ptr).
    ///
    /// # Safety
    /// `ptr` must be a pointer to a Python object of type T.
    ///
    /// Callers must own the object referred to by `ptr`, as this function
    /// implicitly takes ownership of that object.
    #[inline]
    #[deprecated = "this is a deprecated alias for Py::from_owned_ptr"]
    pub unsafe fn from_owned_ptr_or_panic(py: Python, ptr: *mut ffi::PyObject) -> Py<T> {
        Py::from_owned_ptr(py, ptr)
    }

    /// Create a `Py<T>` instance by taking ownership of the given FFI pointer.
    ///
    /// If `ptr` is null then the current Python exception is fetched as a `PyErr`.
    ///
    /// # Safety
    /// If non-null, `ptr` must be a pointer to a Python object of type T.
    #[inline]
    pub unsafe fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<Py<T>> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Ok(Py(nonnull_ptr, PhantomData)),
            None => Err(PyErr::fetch(py)),
        }
    }

    /// Create a `Py<T>` instance by taking ownership of the given FFI pointer.
    ///
    /// If `ptr` is null then `None` is returned.
    ///
    /// # Safety
    /// If non-null, `ptr` must be a pointer to a Python object of type T.
    #[inline]
    pub unsafe fn from_owned_ptr_or_opt(_py: Python, ptr: *mut ffi::PyObject) -> Option<Self> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Some(Py(nonnull_ptr, PhantomData)),
            None => None,
        }
    }

    /// Create a `Py<T>` instance by creating a new reference from the given FFI pointer.
    ///
    /// # Safety
    /// `ptr` must be a pointer to a Python object of type T.
    ///
    /// # Panics
    /// Panics if `ptr` is null.
    #[inline]
    pub unsafe fn from_borrowed_ptr(py: Python, ptr: *mut ffi::PyObject) -> Py<T> {
        match Self::from_borrowed_ptr_or_opt(py, ptr) {
            Some(slf) => slf,
            None => crate::err::panic_after_error(py),
        }
    }

    /// Create a `Py<T>` instance by creating a new reference from the given FFI pointer.
    ///
    /// If `ptr` is null then the current Python exception is fetched as a `PyErr`.
    ///
    /// # Safety
    /// `ptr` must be a pointer to a Python object of type T.
    #[inline]
    pub unsafe fn from_borrowed_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<Self> {
        Self::from_borrowed_ptr_or_opt(py, ptr).ok_or_else(|| PyErr::fetch(py))
    }

    /// Create a `Py<T>` instance by creating a new reference from the given FFI pointer.
    ///
    /// If `ptr` is null then `None` is returned.
    ///
    /// # Safety
    /// `ptr` must be a pointer to a Python object of type T.
    #[inline]
    pub unsafe fn from_borrowed_ptr_or_opt(_py: Python, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|nonnull_ptr| {
            ffi::Py_INCREF(ptr);
            Py(nonnull_ptr, PhantomData)
        })
    }

    /// For internal conversions.
    ///
    /// # Safety
    /// `ptr` must point to a Python object of type T.
    #[inline]
    unsafe fn from_non_null(ptr: NonNull<ffi::PyObject>) -> Self {
        Self(ptr, PhantomData)
    }

    /// Returns the inner pointer without decreasing the refcount.
    #[inline]
    fn into_non_null(self) -> NonNull<ffi::PyObject> {
        let pointer = self.0;
        mem::forget(self);
        pointer
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
        unsafe { PyObject::from_non_null(self.into_non_null()) }
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

impl<T> std::convert::From<&'_ T> for PyObject
where
    T: AsPyPointer + PyNativeType,
{
    fn from(obj: &T) -> Self {
        unsafe { Py::from_borrowed_ptr(obj.py(), obj.as_ptr()) }
    }
}

impl<T> std::convert::From<Py<T>> for PyObject
where
    T: AsRef<PyAny>,
{
    #[inline]
    fn from(other: Py<T>) -> Self {
        unsafe { Self::from_non_null(other.into_non_null()) }
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

/// Py<T> can be used as an error when T is an Error.
///
/// However for GIL lifetime reasons, cause() cannot be implemented for Py<T>.
/// Use .as_ref() to get the GIL-scoped error if you need to inspect the cause.
impl<T> std::error::Error for Py<T>
where
    T: std::error::Error + PyTypeInfo,
    T::AsRefTarget: std::fmt::Display,
{
}

impl<T> std::fmt::Display for Py<T>
where
    T: PyTypeInfo,
    T::AsRefTarget: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let gil = Python::acquire_gil();
        let py = gil.python();
        std::fmt::Display::fmt(self.as_ref(py), f)
    }
}

impl<T> std::fmt::Debug for Py<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("Py").field(&self.0.as_ptr()).finish()
    }
}

/// A commonly-used alias for `Py<PyAny>`.
///
/// This is an owned reference a Python object without any type information. This value can also be
/// safely sent between threads.
///
/// See the documentation for [`Py`](struct.Py.html).
pub type PyObject = Py<PyAny>;

impl PyObject {
    /// Casts the PyObject to a concrete Python object type.
    ///
    /// This can cast only to native Python types, not types implemented in Rust. For a more
    /// flexible alternative, see [`Py::extract`](struct.Py.html#method.extract).
    pub fn cast_as<'p, D>(&'p self, py: Python<'p>) -> Result<&'p D, PyDowncastError>
    where
        D: PyTryFrom<'p>,
    {
        D::try_from(unsafe { py.from_borrowed_ptr::<PyAny>(self.as_ptr()) })
    }
}

#[cfg(test)]
mod test {
    use super::{Py, PyObject};
    use crate::types::PyDict;
    use crate::{ffi, AsPyPointer, Python};

    #[test]
    fn test_call_for_non_existing_method() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj: PyObject = PyDict::new(py).into();
        assert!(obj.call_method0(py, "asdf").is_err());
        assert!(obj
            .call_method(py, "nonexistent_method", (1,), None)
            .is_err());
        assert!(obj.call_method0(py, "nonexistent_method").is_err());
        assert!(obj.call_method1(py, "nonexistent_method", (1,)).is_err());
    }

    #[test]
    fn py_from_dict() {
        let dict: Py<PyDict> = {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let native = PyDict::new(py);
            Py::from(native)
        };
        assert_eq!(unsafe { ffi::Py_REFCNT(dict.as_ptr()) }, 1);
    }

    #[test]
    fn pyobject_from_py() {
        Python::with_gil(|py| {
            let dict: Py<PyDict> = PyDict::new(py).into();
            let cnt = dict.get_refcnt(py);
            let p: PyObject = dict.into();
            assert_eq!(p.get_refcnt(py), cnt);
        });
    }
}
