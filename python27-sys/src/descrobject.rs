use libc::{c_void, c_char, c_int};
use object::{PyObject, PyTypeObject, Py_TYPE};
use structmember::PyMemberDef;
use methodobject::PyMethodDef;

pub type getter =
    unsafe extern "C" fn
                              (slf: *mut PyObject, closure: *mut c_void)
                              -> *mut PyObject;

pub type setter =
    unsafe extern "C" fn
                              (slf: *mut PyObject, value: *mut PyObject,
                               closure: *mut c_void) -> c_int;

#[repr(C)]
#[derive(Copy)]
pub struct PyGetSetDef {
    pub name: *mut c_char,
    pub get: Option<getter>,
    pub set: Option<setter>,
    pub doc: *mut c_char,
    pub closure: *mut c_void,
}

impl Clone for PyGetSetDef {
    #[inline] fn clone(&self) -> PyGetSetDef { *self }
}

pub type wrapperfunc =
    unsafe extern "C" fn(slf: *mut PyObject, args: *mut PyObject,
        wrapped: *mut c_void) -> *mut PyObject;

pub type wrapperfunc_kwds =
    unsafe extern "C" fn(slf: *mut PyObject, args: *mut PyObject,
        wrapped: *mut c_void, kwds: *mut PyObject) -> *mut PyObject;

#[repr(C)]
#[derive(Copy)]
pub struct wrapperbase {
    pub name: *mut c_char,
    pub offset: c_int,
    pub function: *mut c_void,
    pub wrapper: Option<wrapperfunc>,
    pub doc: *mut c_char,
    pub flags: c_int,
    pub name_strobj: *mut PyObject
}

impl Clone for wrapperbase {
    #[inline] fn clone(&self) -> wrapperbase { *self }
}

pub const PyWrapperFlag_KEYWORDS : c_int = 1;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyWrapperDescr_Type: PyTypeObject;
    pub static mut PyDictProxy_Type: PyTypeObject;
    pub static mut PyGetSetDescr_Type: PyTypeObject;
    pub static mut PyMemberDescr_Type: PyTypeObject;
    pub static mut PyProperty_Type: PyTypeObject;

    pub fn PyDescr_NewMethod(arg1: *mut PyTypeObject, arg2: *mut PyMethodDef)
     -> *mut PyObject;
    pub fn PyDescr_NewClassMethod(arg1: *mut PyTypeObject,
                                  arg2: *mut PyMethodDef) -> *mut PyObject;
    pub fn PyDescr_NewMember(arg1: *mut PyTypeObject,
                             arg2: *mut PyMemberDef) -> *mut PyObject;
    pub fn PyDescr_NewGetSet(arg1: *mut PyTypeObject,
                             arg2: *mut PyGetSetDef) -> *mut PyObject;
    pub fn PyDescr_NewWrapper(arg1: *mut PyTypeObject,
                              arg2: *mut wrapperbase,
                              arg3: *mut c_void) -> *mut PyObject;
}

#[inline(always)]
pub unsafe fn PyDescr_IsData(d: *mut PyObject) -> c_int {
    (*Py_TYPE(d)).tp_descr_set.is_some() as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    //pub fn PyDictProxy_New(arg1: *mut PyObject) -> *mut PyObject;
    // PyDictProxy_New is also defined in dictobject.h
    pub fn PyWrapper_New(arg1: *mut PyObject, arg2: *mut PyObject)
     -> *mut PyObject;
}




