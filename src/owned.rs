use crate::{AsPyPointer, IntoPy, IntoPyPointer, Py, PyResult, Python, ffi, type_object::PyTypeInfo, PyNativeType};
use std::fmt;

#[repr(transparent)]
pub struct PyOwned<'py, T>(Py<T>, Python<'py>);

impl<'py, T> PyOwned<'py, T> {

    // Creates a PyOwned without checking the type.
    #[inline]
    pub(crate) unsafe fn from_owned_ptr(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        Py::from_owned_ptr_or_err(py, ptr).map(|obj| Self(obj, py))
    }

    #[inline]
    pub(crate) unsafe fn from_owned_ptr_or_opt(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Option<Self> {
        Py::from_owned_ptr_or_opt(py, ptr).map(|obj| Self(obj, py))
    }

    #[inline]
    pub(crate) unsafe fn from_owned_ptr_or_panic(py: Python<'py>, ptr: *mut ffi::PyObject) -> Self {
        Self(Py::from_owned_ptr(py, ptr), py)
    }

    #[inline]
    pub(crate) unsafe fn from_borrowed_ptr(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        Py::from_borrowed_ptr_or_err(py, ptr).map(|obj| Self(obj, py))
    }

    #[inline]
    pub(crate) unsafe fn from_borrowed_ptr_or_opt(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Option<Self> {
        Py::from_borrowed_ptr_or_opt(py, ptr).map(|obj| Self(obj, py))
    }

    #[inline]
    pub(crate) unsafe fn from_borrowed_ptr_or_panic(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Self {
        Self(Py::from_borrowed_ptr(py, ptr), py)
    }

    #[inline]
    pub(crate) fn from_inner(inner: Py<T>, py: Python<'py>) -> Self {
        Self(inner, py)
    }
}

impl<T> Clone for PyOwned<'_, T>
{
    fn clone(&self) -> Self {
        Self(self.0.clone_ref(self.1), self.1)
    }
}

impl<T: PyTypeInfo> fmt::Debug for PyOwned<'_, T>
where
    T::AsRefTarget: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T: PyTypeInfo> fmt::Display for PyOwned<'_, T>
where
    T::AsRefTarget: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'py, T: PyTypeInfo> PyOwned<'py, T> {
    #[inline]
    pub fn as_ref(&self) -> &T::AsRefTarget {
        self.0.as_ref(self.1)
    }

    // // private helper to convert to owned reference world.
    // #[inline]
    // pub(crate) fn into_ref(self) -> &'py T::AsRefTarget {
    //     self.0.into_ref(self.1)
    // }
}

impl<T> From<PyOwned<'_, T>> for Py<T> {
    #[inline]
    fn from(owned: PyOwned<'_, T>) -> Py<T> {
        owned.0
    }
}

impl<T> IntoPy<Py<T>> for PyOwned<'_, T> {
    #[inline]
    fn into_py(self, _py: Python) -> Py<T> {
        self.into()
    }
}

impl<T> AsPyPointer for PyOwned<'_, T> {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0.as_ptr()
    }
}

impl<T> IntoPyPointer for PyOwned<'_, T> {
    #[inline]
    fn into_ptr(self) -> *mut ffi::PyObject {
        self.0.into_ptr()
    }
}

impl<T> PartialEq for PyOwned<'_, T>
{
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl<'py, T: PyNativeType> From<&'py T> for PyOwned<'py, T>
where
    &'py T: IntoPyPointer
{
    fn from(other: &'py T) -> Self {
        unsafe { Self::from_owned_ptr_or_panic(other.py(), other.into_ptr()) }
    }
}
