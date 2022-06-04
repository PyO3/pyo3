use crate::methodobject::PyMethodDef;
use crate::object::{PyObject, PyTypeObject};
use crate::structmember::PyMemberDef;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

pub type getter = unsafe extern "C" fn(slf: *mut PyObject, closure: *mut c_void) -> *mut PyObject;
pub type setter =
    unsafe extern "C" fn(slf: *mut PyObject, value: *mut PyObject, closure: *mut c_void) -> c_int;

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PyGetSetDef {
    pub name: *mut c_char,
    pub get: Option<getter>,
    pub set: Option<setter>,
    pub doc: *mut c_char,
    pub closure: *mut c_void,
}

impl Default for PyGetSetDef {
    fn default() -> PyGetSetDef {
        PyGetSetDef {
            name: ptr::null_mut(),
            get: None,
            set: None,
            doc: ptr::null_mut(),
            closure: ptr::null_mut(),
        }
    }
}

#[cfg(not(Py_LIMITED_API))]
pub type wrapperfunc = Option<
    unsafe extern "C" fn(
        slf: *mut PyObject,
        args: *mut PyObject,
        wrapped: *mut c_void,
    ) -> *mut PyObject,
>;

#[cfg(not(Py_LIMITED_API))]
pub type wrapperfunc_kwds = Option<
    unsafe extern "C" fn(
        slf: *mut PyObject,
        args: *mut PyObject,
        wrapped: *mut c_void,
        kwds: *mut PyObject,
    ) -> *mut PyObject,
>;

#[cfg(not(Py_LIMITED_API))]
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

#[cfg(not(Py_LIMITED_API))]
pub const PyWrapperFlag_KEYWORDS: c_int = 1;

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
pub struct PyDescrObject {
    pub ob_base: PyObject,
    pub d_type: *mut PyTypeObject,
    pub d_name: *mut PyObject,
    pub d_qualname: *mut PyObject,
}

// skipped non-limited PyDescr_TYPE
// skipped non-limited PyDescr_NAME

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
pub struct PyMethodDescrObject {
    pub d_common: PyDescrObject,
    pub d_method: *mut PyMethodDef,
    #[cfg(all(not(PyPy), Py_3_8))]
    pub vectorcall: Option<crate::vectorcallfunc>,
}

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
pub struct PyMemberDescrObject {
    pub d_common: PyDescrObject,
    pub d_member: *mut PyGetSetDef,
}

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
pub struct PyGetSetDescrObject {
    pub d_common: PyDescrObject,
    pub d_getset: *mut PyGetSetDef,
}

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
pub struct PyWrapperDescrObject {
    pub d_common: PyDescrObject,
    pub d_base: *mut wrapperbase,
    pub d_wrapped: *mut c_void,
}

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
