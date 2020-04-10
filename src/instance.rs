// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::err::{PyErr, PyResult};
use crate::gil;
use crate::type_object::PyBorrowFlagLayout;
use crate::{
    ffi, AsPyPointer, FromPy, FromPyObject, IntoPyPointer, PyCell, PyClass, PyClassInitializer,
    PyObject, PyRef, PyRefMut, PyTypeInfo, Python, ToPyObject,
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
    /// Cast `&PyObject` to `&Self` without no type checking.
    ///
    /// # Safety
    ///
    /// `obj` must have the same layout as `*const ffi::PyObject` and must be
    /// an instance of a type corresponding to `Self`.
    unsafe fn unchecked_downcast(obj: &PyObject) -> &Self {
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

    /// Construct `Py<T>` from the result of a Python FFI call that
    ///
    /// Returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is NULL.
    /// Unsafe because the pointer might be invalid.
    pub unsafe fn from_owned_ptr_or_opt(ptr: *mut ffi::PyObject) -> Option<Py<T>> {
        NonNull::new(ptr).map(|p| Py(p, PhantomData))
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

    /// Create from the inner pointer without increasing the refcount.
    ///
    /// This will eventually move into its own trait.
    pub(crate) fn from_not_null(ptr: NonNull<ffi::PyObject>) -> Self {
        Self(ptr, PhantomData)
    }

    /// Returns the inner pointer without decreasing the refcount.
    ///
    /// This will eventually move into its own trait.
    pub(crate) fn into_non_null(self) -> NonNull<ffi::PyObject> {
        let pointer = self.0;
        mem::forget(self);
        pointer
    }

    // /// Returns whether the object is considered to be None.
    // ///
    // /// This is equivalent to the Python expression `self is None`.
    // pub fn is_none(&self) -> bool {
    //     unsafe { ffi::Py_None() == self.as_ptr() }
    // }

    // /// Returns whether the object is considered to be true.
    // ///
    // /// This is equivalent to the Python expression `bool(self)`.
    // pub fn is_true(&self, py: Python) -> PyResult<bool> {
    //     let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
    //     if v == -1 {
    //         Err(PyErr::fetch(py))
    //     } else {
    //         Ok(v != 0)
    //     }
    // }

    // /// Casts the PyObject to a concrete Python object type.
    // ///
    // /// This can cast only to native Python types, not types implemented in Rust.
    // pub fn cast_as<'p, D>(&'p self, py: Python<'p>) -> Result<&'p D, PyDowncastError>
    // where
    //     D: PyTryFrom<'p>,
    // {
    //     D::try_from(self.as_ref(py))
    // }

    // /// Extracts some type from the Python object.
    // ///
    // /// This is a wrapper function around `FromPyObject::extract()`.
    // pub fn extract<'p, D>(&'p self, py: Python<'p>) -> PyResult<D>
    // where
    //     D: FromPyObject<'p>,
    // {
    //     FromPyObject::extract(self.as_ref(py))
    // }

    // /// Retrieves an attribute value.
    // ///
    // /// This is equivalent to the Python expression `self.attr_name`.
    // pub fn getattr<N>(&self, py: Python, attr_name: N) -> PyResult<PyObject>
    // where
    //     N: ToPyObject,
    // {
    //     attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
    //         PyObject::from_owned_ptr_or_err(py, ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
    //     })
    // }

    // /// Calls the object.
    // ///
    // /// This is equivalent to the Python expression `self(*args, **kwargs)`.
    // pub fn call(
    //     &self,
    //     py: Python,
    //     args: impl IntoPy<Py<PyTuple>>,
    //     kwargs: Option<&PyDict>,
    // ) -> PyResult<PyObject> {
    //     let args = args.into_py(py).into_ptr();
    //     let kwargs = kwargs.into_ptr();
    //     let result = unsafe {
    //         PyObject::from_owned_ptr_or_err(py, ffi::PyObject_Call(self.as_ptr(), args, kwargs))
    //     };
    //     unsafe {
    //         ffi::Py_XDECREF(args);
    //         ffi::Py_XDECREF(kwargs);
    //     }
    //     result
    // }

    // /// Calls the object with only positional arguments.
    // ///
    // /// This is equivalent to the Python expression `self(*args)`.
    // pub fn call1(&self, py: Python, args: impl IntoPy<Py<PyTuple>>) -> PyResult<PyObject> {
    //     self.call(py, args, None)
    // }

    // /// Calls the object without arguments.
    // ///
    // /// This is equivalent to the Python expression `self()`.
    // pub fn call0(&self, py: Python) -> PyResult<PyObject> {
    //     self.call(py, (), None)
    // }

    // /// Calls a method on the object.
    // ///
    // /// This is equivalent to the Python expression `self.name(*args, **kwargs)`.
    // pub fn call_method(
    //     &self,
    //     py: Python,
    //     name: &str,
    //     args: impl IntoPy<Py<PyTuple>>,
    //     kwargs: Option<&PyDict>,
    // ) -> PyResult<PyObject> {
    //     name.with_borrowed_ptr(py, |name| unsafe {
    //         let args = args.into_py(py).into_ptr();
    //         let kwargs = kwargs.into_ptr();
    //         let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
    //         if ptr.is_null() {
    //             return Err(PyErr::fetch(py));
    //         }
    //         let result = PyObject::from_owned_ptr_or_err(py, ffi::PyObject_Call(ptr, args, kwargs));
    //         ffi::Py_DECREF(ptr);
    //         ffi::Py_XDECREF(args);
    //         ffi::Py_XDECREF(kwargs);
    //         result
    //     })
    // }

    // /// Calls a method on the object with only positional arguments.
    // ///
    // /// This is equivalent to the Python expression `self.name(*args)`.
    // pub fn call_method1(
    //     &self,
    //     py: Python,
    //     name: &str,
    //     args: impl IntoPy<Py<PyTuple>>,
    // ) -> PyResult<PyObject> {
    //     self.call_method(py, name, args, None)
    // }

    // /// Calls a method on the object with no arguments.
    // ///
    // /// This is equivalent to the Python expression `self.name()`.
    // pub fn call_method0(&self, py: Python, name: &str) -> PyResult<PyObject> {
    //     self.call_method(py, name, (), None)
    // }
}

/// Retrieves `&'py` types from `Py<T>` or `Py<PyObject>`.
///
/// # Examples
/// `Py<T>::as_ref` returns `&PyDict`, `&PyList` or so for native types, and `&PyCell<T>`
/// for `#[pyclass]`.
/// ```
/// # use pyo3::prelude::*;
/// let obj: Py<PyObject> = {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     py.eval("[]", None, None).unwrap().to_object(py).into()
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
        let any = self.as_ptr() as *const PyObject;
        unsafe { PyNativeType::unchecked_downcast(&*any) }
    }
}

impl<T> ToPyObject for Py<T> {
    fn to_object<'p>(&self, py: Python<'p>) -> &'p PyObject {
        unsafe { py.from_owned_ptr(self.clone_ref(py).into_ptr()) }
    }
}

impl<T> FromPy<Py<T>> for Py<PyObject> {
    /// Converts a `Py` instance to `PyObject`.
    /// Consumes `self` without calling `Py_DECREF()`.
    #[inline]
    fn from_py(other: Py<T>, _py: Python) -> Self {
        Py::from_not_null(other.into_non_null())
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
    /// Extracts `Self` from the source `Py<PyObject>`.
    fn extract(ob: &'a PyObject) -> PyResult<Self> {
        unsafe {
            ob.extract::<&T::AsRefTarget>()
                .map(|val| Py::from_borrowed_ptr(val.as_ptr()))
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
