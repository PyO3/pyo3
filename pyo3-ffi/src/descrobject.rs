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
    pub doc: *const c_char,
    pub closure: *mut c_void,
}

impl Default for PyGetSetDef {
    fn default() -> PyGetSetDef {
        PyGetSetDef {
            name: ptr::null_mut(),
            get: None,
            set: None,
            doc: ptr::null(),
            closure: ptr::null_mut(),
        }
    }
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
    #[cfg_attr(PyPy, link_name = "PyPyProperty_Type")]
    pub static mut PyProperty_Type: PyTypeObject;
}

extern "C" {
    pub fn PyDescr_NewMethod(arg1: *mut PyTypeObject, arg2: *mut PyMethodDef) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDescr_NewClassMethod")]
    pub fn PyDescr_NewClassMethod(arg1: *mut PyTypeObject, arg2: *mut PyMethodDef)
        -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDescr_NewMember")]
    pub fn PyDescr_NewMember(arg1: *mut PyTypeObject, arg2: *mut PyMemberDef) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDescr_NewGetSet")]
    pub fn PyDescr_NewGetSet(arg1: *mut PyTypeObject, arg2: *mut PyGetSetDef) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyDictProxy_New")]
    pub fn PyDictProxy_New(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyWrapper_New(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;
}
