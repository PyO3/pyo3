use crate::{AsPyPointer, IntoPy, IntoPyPointer, Py, PyResult, Python, ffi, type_object::PyTypeInfo};
use std::fmt;

#[repr(transparent)]
pub struct PyOwned<'py, T>(Py<T>, Python<'py>);

impl<'py, T> PyOwned<'py, T> {
    // Creates a PyOwned without checking the type.
    pub(crate) unsafe fn from_owned_ptr(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        Py::from_owned_ptr_or_err(py, ptr).map(|obj| Self(obj, py))
    }

    pub(crate) unsafe fn from_owned_ptr_or_opt(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Option<Self> {
        Py::from_owned_ptr_or_opt(py, ptr).map(|obj| Self(obj, py))
    }

    pub(crate) unsafe fn from_owned_ptr_or_panic(py: Python<'py>, ptr: *mut ffi::PyObject) -> Self {
        Self::from_owned_ptr(py, ptr).expect("ptr is not null")
    }

    pub(crate) unsafe fn from_borrowed_ptr(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        ffi::Py_XINCREF(ptr);
        Self::from_owned_ptr(py, ptr)
    }

    pub(crate) unsafe fn from_borrowed_ptr_or_opt(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Option<Self> {
        ffi::Py_XINCREF(ptr);
        Self::from_owned_ptr_or_opt(py, ptr)
    }

    pub(crate) unsafe fn from_borrowed_ptr_or_panic(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Self {
        Self::from_borrowed_ptr(py, ptr).expect("ptr is not null")
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
    pub fn as_ref(&self) -> &T::AsRefTarget {
        self.0.as_ref(self.1)
    }

    // private helper to convert to owned reference world.
    pub(crate) fn into_ref(self) -> &'py T::AsRefTarget {
        self.0.into_ref(self.1)
    }
}

impl<T> From<PyOwned<'_, T>> for Py<T> {
    fn from(owned: PyOwned<'_, T>) -> Py<T> {
        owned.0
    }
}

impl<T> IntoPy<Py<T>> for PyOwned<'_, T> {
    fn into_py(self, _py: Python) -> Py<T> {
        self.into()
    }
}

impl<T> AsPyPointer for PyOwned<'_, T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0.as_ptr()
    }
}

impl<T> IntoPyPointer for PyOwned<'_, T> {
    fn into_ptr(self) -> *mut ffi::PyObject {
        self.0.into_ptr()
    }
}
