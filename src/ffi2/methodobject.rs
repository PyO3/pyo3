use std::ptr;
use std::os::raw::{c_char, c_int};
use ffi2::object::{PyObject, PyTypeObject, Py_TYPE};

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyCFunction_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyCFunction_Check(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyCFunction_Type;
    (Py_TYPE(op) == u) as c_int
}

pub type PyCFunction =
    unsafe extern "C" fn
                              (slf: *mut PyObject, args: *mut PyObject)
                              -> *mut PyObject;
pub type PyCFunctionWithKeywords =
    unsafe extern "C" fn
                              (slf: *mut PyObject, args: *mut PyObject,
                               kwds: *mut PyObject) -> *mut PyObject;
pub type PyNoArgsFunction =
    unsafe extern "C" fn(slf: *mut PyObject)
                              -> *mut PyObject;


#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyCFunction_GetFunction(f: *mut PyObject) -> Option<PyCFunction>;
    pub fn PyCFunction_GetSelf(f: *mut PyObject) -> *mut PyObject;
    pub fn PyCFunction_GetFlags(f: *mut PyObject) -> c_int;
    pub fn PyCFunction_Call(f: *mut PyObject, args: *mut PyObject,
                            kwds: *mut PyObject) -> *mut PyObject;
}

#[repr(C)]
#[derive(Copy)]
pub struct PyMethodDef {
    pub ml_name: *const c_char,
    pub ml_meth: Option<PyCFunction>,
    pub ml_flags: c_int,
    pub ml_doc: *const c_char,
}

pub const PyMethodDef_INIT : PyMethodDef = PyMethodDef {
    ml_name: ::std::ptr::null(),
    ml_meth: None,
    ml_flags: 0,
    ml_doc: ::std::ptr::null(),
};

impl Clone for PyMethodDef {
    #[inline] fn clone(&self) -> PyMethodDef { *self }
}

/* Flag passed to newmethodobject */
pub const METH_OLDARGS  : c_int = 0x0000;
pub const METH_VARARGS  : c_int = 0x0001;
pub const METH_KEYWORDS : c_int = 0x0002;
/* METH_NOARGS and METH_O must not be combined with the flags above. */
pub const METH_NOARGS   : c_int = 0x0004;
pub const METH_O        : c_int = 0x0008;

/* METH_CLASS and METH_STATIC are a little different; these control
   the construction of methods for a class.  These cannot be used for
   functions in modules. */
pub const METH_CLASS    : c_int = 0x0010;
pub const METH_STATIC   : c_int = 0x0020;

/* METH_COEXIST allows a method to be entered eventhough a slot has
   already filled the entry.  When defined, the flag allows a separate
   method, "__contains__" for example, to coexist with a defined
   slot like sq_contains. */

pub const METH_COEXIST   : c_int = 0x0040;


#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyMethodChain {
    pub methods: *mut PyMethodDef,
    pub link: *mut PyMethodChain,
}

/*
#[repr(C)]
#[derive(Copy)]
struct PyCFunctionObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub m_ml: *mut PyMethodDef,
    pub m_self: *mut PyObject,
    pub m_module: *mut PyObject,
}
*/

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn Py_FindMethod(methods: *mut PyMethodDef, slf: *mut PyObject,
                         name: *const c_char) -> *mut PyObject;
    pub fn PyCFunction_NewEx(ml: *mut PyMethodDef, slf: *mut PyObject,
                             module: *mut PyObject) -> *mut PyObject;
    pub fn Py_FindMethodInChain(chain: *mut PyMethodChain, slf: *mut PyObject,
                                name: *const c_char) -> *mut PyObject;
    pub fn PyCFunction_ClearFreeList() -> c_int;
}

#[inline(always)]
pub unsafe fn PyCFunction_New(ml: *mut PyMethodDef, slf: *mut PyObject) -> *mut PyObject {
    PyCFunction_NewEx(ml, slf, ptr::null_mut())
}

