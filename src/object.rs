// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::err::{PyDowncastError, PyErr, PyResult};
use crate::ffi;
use crate::gil;
use crate::instance::{AsPyRef, PyNativeType, PyRef, PyRefMut};
use crate::types::{PyAny, PyDict, PyTuple};
use crate::AsPyPointer;
use crate::Py;
use crate::Python;
use crate::{
    FromPyObject, IntoPy, IntoPyObject, IntoPyPointer, PyTryFrom, ToBorrowedObject, ToPyObject,
};
use std::ptr::NonNull;

/// A python object
///
/// The python object's lifetime is managed by python's garbage
/// collector.
///
/// Technically, it is a safe wrapper around `NonNull<ffi::PyObject>`.
#[derive(Debug)]
#[repr(transparent)]
pub struct PyObject(NonNull<ffi::PyObject>);

// `PyObject` is thread-safe, any python related operations require a Python<'p> token.
unsafe impl Send for PyObject {}
unsafe impl Sync for PyObject {}

impl PyObject {
    /// For internal conversions
    pub(crate) unsafe fn from_not_null(ptr: NonNull<ffi::PyObject>) -> PyObject {
        PyObject(ptr)
    }

    pub(crate) unsafe fn into_nonnull(self) -> NonNull<ffi::PyObject> {
        let res = self.0;
        std::mem::forget(self); // Avoid Drop
        res
    }

    /// Creates a `PyObject` instance for the given FFI pointer.
    /// This moves ownership over the pointer into the `PyObject`.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyObject {
        debug_assert!(
            !ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
            format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr))
        );
        PyObject(NonNull::new_unchecked(ptr))
    }

    /// Creates a `PyObject` instance for the given FFI pointer.
    /// Panics if the pointer is `null`.
    /// Undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_owned_ptr_or_panic(_py: Python, ptr: *mut ffi::PyObject) -> PyObject {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => PyObject(nonnull_ptr),
            None => {
                crate::err::panic_after_error();
            }
        }
    }

    /// Construct `PyObject` from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `Err(PyErr)` if the pointer is `null`.
    pub unsafe fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<PyObject> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Ok(PyObject(nonnull_ptr)),
            None => Err(PyErr::fetch(py)),
        }
    }

    /// Construct `PyObject` from the result of a Python FFI call that
    /// returns a new reference (owned pointer).
    /// Returns `None` if the pointer is `null`.
    pub unsafe fn from_owned_ptr_or_opt(_py: Python, ptr: *mut ffi::PyObject) -> Option<PyObject> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Some(PyObject(nonnull_ptr)),
            None => None,
        }
    }

    /// Creates a `PyObject` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyObject {
        debug_assert!(
            !ptr.is_null() && ffi::Py_REFCNT(ptr) > 0,
            format!("REFCNT: {:?} - {:?}", ptr, ffi::Py_REFCNT(ptr))
        );
        ffi::Py_INCREF(ptr);
        PyObject(NonNull::new_unchecked(ptr))
    }

    /// Creates a `PyObject` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Returns `Err(PyErr)` if the pointer is `null`.
    pub unsafe fn from_borrowed_ptr_or_err(
        py: Python,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<PyObject> {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(PyObject::from_borrowed_ptr(py, ptr))
        }
    }

    /// Creates a `PyObject` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Returns `None` if the pointer is `null`.
    pub unsafe fn from_borrowed_ptr_or_opt(
        py: Python,
        ptr: *mut ffi::PyObject,
    ) -> Option<PyObject> {
        if ptr.is_null() {
            None
        } else {
            Some(PyObject::from_borrowed_ptr(py, ptr))
        }
    }

    /// Gets the reference count of the ffi::PyObject pointer.
    pub fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.0.as_ptr()) }
    }

    /// Clone self, Calls Py_INCREF() on the ptr.
    pub fn clone_ref(&self, py: Python) -> Self {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }

    /// Returns whether the object is considered to be None.
    /// This is equivalent to the Python expression: 'is None'
    pub fn is_none(&self) -> bool {
        unsafe { ffi::Py_None() == self.as_ptr() }
    }

    /// Returns whether the object is considered to be true.
    /// This is equivalent to the Python expression: 'not not self'
    pub fn is_true(&self, py: Python) -> PyResult<bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(py))
        } else {
            Ok(v != 0)
        }
    }

    /// Casts the PyObject to a concrete Python object type.
    pub fn cast_as<D>(&self, py: Python) -> Result<&D, PyDowncastError>
    where
        D: for<'v> PyTryFrom<'v>,
    {
        D::try_from(self.as_ref(py))
    }

    /// Extracts some type from the Python object.
    /// This is a wrapper function around `FromPyObject::extract()`.
    pub fn extract<'p, D>(&'p self, py: Python) -> PyResult<D>
    where
        D: FromPyObject<'p>,
    {
        FromPyObject::extract(self.as_ref(py).into())
    }

    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    pub fn getattr<N>(&self, py: Python, attr_name: N) -> PyResult<PyObject>
    where
        N: ToPyObject,
    {
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            PyObject::from_owned_ptr_or_err(py, ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    /// Calls the object.
    /// This is equivalent to the Python expression: 'self(*args, **kwargs)'
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

    /// Calls the object without arguments.
    /// This is equivalent to the Python expression: 'self()'
    pub fn call0(&self, py: Python) -> PyResult<PyObject> {
        self.call(py, (), None)
    }

    /// Calls the object.
    /// This is equivalent to the Python expression: 'self(*args)'
    pub fn call1(&self, py: Python, args: impl IntoPy<Py<PyTuple>>) -> PyResult<PyObject> {
        self.call(py, args, None)
    }

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args, **kwargs)'
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

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name()'
    pub fn call_method0(&self, py: Python, name: &str) -> PyResult<PyObject> {
        self.call_method(py, name, (), None)
    }

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args)'
    pub fn call_method1(
        &self,
        py: Python,
        name: &str,
        args: impl IntoPy<Py<PyTuple>>,
    ) -> PyResult<PyObject> {
        self.call_method(py, name, args, None)
    }
}

impl AsPyRef<PyAny> for PyObject {
    #[inline]
    fn as_ref(&self, _py: Python) -> PyRef<PyAny> {
        unsafe { PyRef::from_ref(&*(self as *const _ as *const PyAny)) }
    }
    #[inline]
    fn as_mut(&mut self, _py: Python) -> PyRefMut<PyAny> {
        unsafe { PyRefMut::from_mut(&mut *(self as *mut _ as *mut PyAny)) }
    }
}

impl ToPyObject for PyObject {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

impl AsPyPointer for PyObject {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0.as_ptr()
    }
}

impl IntoPyPointer for PyObject {
    /// Gets the underlying FFI pointer, returns a owned pointer.
    #[inline]
    #[must_use]
    fn into_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.0.as_ptr();
        std::mem::forget(self); // Avoid Drop
        ptr
    }
}

impl PartialEq for PyObject {
    /// Checks for identity, not python's `__eq__`
    #[inline]
    fn eq(&self, o: &PyObject) -> bool {
        self.0 == o.0
    }
}

impl IntoPyObject for PyObject {
    #[inline]
    fn into_object(self, _py: Python) -> PyObject {
        self
    }
}

impl<'a> FromPyObject<'a> for PyObject {
    #[inline]
    /// Extracts `Self` from the source `PyObject`.
    fn extract(ob: &'a PyAny) -> PyResult<Self> {
        unsafe { Ok(PyObject::from_borrowed_ptr(ob.py(), ob.as_ptr())) }
    }
}

/// Dropping a `PyObject` instance decrements the reference count on the object by 1.
impl Drop for PyObject {
    fn drop(&mut self) {
        unsafe {
            gil::register_pointer(self.0);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::types::PyDict;
    use crate::PyObject;
    use crate::Python;

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
}
