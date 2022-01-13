use crate::{ffi, FromPyObject, PyAny, PyResult, Python};

/// Python 3.8 and up - __ipow__ has modulo argument correctly populated.
#[cfg(Py_3_8)]
#[repr(transparent)]
pub struct IPowModulo(*mut ffi::PyObject);

/// Python 3.7 and older - __ipow__ does not have modulo argument correctly populated.
#[cfg(not(Py_3_8))]
#[repr(transparent)]
pub struct IPowModulo(std::mem::MaybeUninit<*mut ffi::PyObject>);

/// Helper to use as pymethod ffi definition
#[allow(non_camel_case_types)]
pub type ipowfunc = unsafe extern "C" fn(
    arg1: *mut ffi::PyObject,
    arg2: *mut ffi::PyObject,
    arg3: IPowModulo,
) -> *mut ffi::PyObject;

impl IPowModulo {
    #[cfg(Py_3_8)]
    #[inline]
    pub fn extract<'a, T: FromPyObject<'a>>(self, py: Python<'a>) -> PyResult<T> {
        unsafe { py.from_borrowed_ptr::<PyAny>(self.0) }.extract()
    }

    #[cfg(not(Py_3_8))]
    #[inline]
    pub fn extract<'a, T: FromPyObject<'a>>(self, py: Python<'a>) -> PyResult<T> {
        unsafe { py.from_borrowed_ptr::<PyAny>(ffi::Py_None()) }.extract()
    }
}
