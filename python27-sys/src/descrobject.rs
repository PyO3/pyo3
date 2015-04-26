use libc::{c_void, c_char, c_int};
use object::{PyObject, PyTypeObject};
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

#[link(name = "python2.7")]
extern "C" {
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
    //pub fn PyDescr_NewWrapper(arg1: *mut PyTypeObject,
    //                          arg2: *mut Struct_wrapperbase,
    //                          arg3: *mut c_void) -> *mut PyObject;
    pub fn PyWrapper_New(arg1: *mut PyObject, arg2: *mut PyObject)
     -> *mut PyObject;
}


