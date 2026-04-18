use crate::descrobject::{PyGetSetDef, PyMemberDef};
use crate::methodobject::PyMethodDef;
use crate::object::{PyObject, PyTypeObject};
use std::ffi::{c_char, c_int};

extern_libpython! {
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

extern_libpython! {
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

    pub fn PyMember_GetOne(addr: *const c_char, l: *mut PyMemberDef) -> *mut PyObject;
    pub fn PyMember_SetOne(addr: *mut c_char, l: *mut PyMemberDef, value: *mut PyObject) -> c_int;
}
