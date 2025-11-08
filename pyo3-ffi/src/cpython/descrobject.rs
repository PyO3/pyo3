use crate::{PyGetSetDef, PyMethodDef, PyObject, PyTypeObject};
use std::ffi::{c_char, c_int, c_void};

pub type wrapperfunc = Option<
    unsafe extern "C" fn(
        slf: *mut PyObject,
        args: *mut PyObject,
        wrapped: *mut c_void,
    ) -> *mut PyObject,
>;

pub type wrapperfunc_kwds = Option<
    unsafe extern "C" fn(
        slf: *mut PyObject,
        args: *mut PyObject,
        wrapped: *mut c_void,
        kwds: *mut PyObject,
    ) -> *mut PyObject,
>;

#[repr(C)]
pub struct wrapperbase {
    pub name: *const c_char,
    pub offset: c_int,
    pub function: *mut c_void,
    pub wrapper: wrapperfunc,
    pub doc: *const c_char,
    pub flags: c_int,
    pub name_strobj: *mut PyObject,
}

pub const PyWrapperFlag_KEYWORDS: c_int = 1;

#[repr(C)]
pub struct PyDescrObject {
    pub ob_base: PyObject,
    pub d_type: *mut PyTypeObject,
    pub d_name: *mut PyObject,
    pub d_qualname: *mut PyObject,
}

// skipped non-limited PyDescr_TYPE
// skipped non-limited PyDescr_NAME

#[repr(C)]
pub struct PyMethodDescrObject {
    pub d_common: PyDescrObject,
    pub d_method: *mut PyMethodDef,
    #[cfg(all(not(PyPy), Py_3_8))]
    pub vectorcall: Option<crate::vectorcallfunc>,
}

#[repr(C)]
pub struct PyMemberDescrObject {
    pub d_common: PyDescrObject,
    pub d_member: *mut PyGetSetDef,
}

#[repr(C)]
pub struct PyGetSetDescrObject {
    pub d_common: PyDescrObject,
    pub d_getset: *mut PyGetSetDef,
}

#[repr(C)]
pub struct PyWrapperDescrObject {
    pub d_common: PyDescrObject,
    pub d_base: *mut wrapperbase,
    pub d_wrapped: *mut c_void,
}

// skipped _PyMethodWrapper_Type

// skipped non-limited PyDescr_NewWrapper
// skipped non-limited PyDescr_IsData
