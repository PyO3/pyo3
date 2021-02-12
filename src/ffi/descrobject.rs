use crate::ffi::methodobject::PyMethodDef;
use crate::ffi::object::{PyObject, PyTypeObject};
#[cfg(all(not(PyPy), not(Py_LIMITED_API)))]
use crate::ffi::object::{PyObject_GenericGetDict, PyObject_GenericSetDict};
use crate::ffi::structmember::PyMemberDef;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

pub type getter = unsafe extern "C" fn(slf: *mut PyObject, closure: *mut c_void) -> *mut PyObject;
pub type setter =
    unsafe extern "C" fn(slf: *mut PyObject, value: *mut PyObject, closure: *mut c_void) -> c_int;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PyGetSetDef {
    pub name: *mut c_char,
    pub get: Option<getter>,
    pub set: Option<setter>,
    pub doc: *mut c_char,
    pub closure: *mut c_void,
}

// skipped non-limited wrapperfunc
// skipped non-limited wrapperfunc_kwds
// skipped non-limited struct wrapperbase
// skipped non-limited PyWrapperFlag_KEYWORDS

// skipped non-limited PyDescrObject
// skipped non-limited PyDescr_COMMON
// skipped non-limited PyDescr_TYPE
// skipped non-limited PyDescr_NAME
// skipped non-limited PyMethodDescrObject
// skipped non-limited PyMemberDescrObject
// skipped non-limited PyGetSetDescrObject
// skipped non-limited PyWrapperDescrObject

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyClassMethodDescr_Type")]
    pub static mut PyClassMethodDescr_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyGetSetDescr_Type")]
    pub static mut PyGetSetDescr_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyMemberDescr_Type")]
    pub static mut PyMemberDescr_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyMethodDescr_Type")]
    pub static mut PyMethodDescr_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyWrapperDescr_Type")]
    pub static mut PyWrapperDescr_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyDictProxy_Type")]
    pub static mut PyDictProxy_Type: PyTypeObject;
    // skipped non-limited _PyMethodWrapper_Type
}

extern "C" {
    pub fn PyDescr_NewMethod(arg1: *mut PyTypeObject, arg2: *mut PyMethodDef) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDescr_NewClassMethod")]
    pub fn PyDescr_NewClassMethod(arg1: *mut PyTypeObject, arg2: *mut PyMethodDef)
        -> *mut PyObject;
    pub fn PyDescr_NewMember(arg1: *mut PyTypeObject, arg2: *mut PyMemberDef) -> *mut PyObject;
    pub fn PyDescr_NewGetSet(arg1: *mut PyTypeObject, arg2: *mut PyGetSetDef) -> *mut PyObject;
    // skipped non-limited PyDescr_NewWrapper
    // skipped non-limited PyDescr_IsData
    #[cfg_attr(PyPy, link_name = "PyPyDictProxy_New")]
    pub fn PyDictProxy_New(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyWrapper_New(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyProperty_Type")]
    pub static mut PyProperty_Type: PyTypeObject;
}

/// Helper initial value of [`PyGetSetDef`] for a Python class.
///
/// Not present in `cpython/Include/descrobject`.
#[deprecated(note = "not present in Python headers; to be removed")]
pub const PyGetSetDef_INIT: PyGetSetDef = PyGetSetDef {
    name: ptr::null_mut(),
    get: None,
    set: None,
    doc: ptr::null_mut(),
    closure: ptr::null_mut(),
};

#[cfg(any(PyPy, Py_LIMITED_API))]
#[deprecated(note = "not present in Python headers; to be removed")]
#[allow(deprecated)]
pub const PyGetSetDef_DICT: PyGetSetDef = PyGetSetDef_INIT;

/// Helper initial value of [`PyGetSetDef`] for a dict-like Python class.
///
/// Not present in `cpython/Include/descrobject.h`.
// PyPy doesn't export neither PyObject_GenericGetDict/PyObject_GenericSetDict
// Py_LIMITED_API exposes PyObject_GenericSetDict but not Get.
#[cfg(all(not(PyPy), not(Py_LIMITED_API)))]
#[deprecated(note = "not present in Python headers; to be removed")]
pub const PyGetSetDef_DICT: PyGetSetDef = PyGetSetDef {
    name: "__dict__\0".as_ptr() as *mut c_char,
    get: Some(PyObject_GenericGetDict),
    set: Some(PyObject_GenericSetDict),
    doc: ptr::null_mut(),
    closure: ptr::null_mut(),
};
