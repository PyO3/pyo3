use libc::{c_char, c_int, c_double};
use pyport::Py_ssize_t;
use object::*;

#[repr(C)]
#[derive(Copy, Clone)]
struct PyFloatObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub ob_fval: c_double
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyFloat_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyFloat_Check(op : *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyFloat_Type)
}

#[inline(always)]
pub unsafe fn PyFloat_CheckExact(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyFloat_Type;
    (Py_TYPE(op) == u) as c_int
}

pub const PyFloat_STR_PRECISION : c_int = 12;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyFloat_FromString(str: *mut PyObject,
                              pend: *mut *mut c_char)
     -> *mut PyObject;
    pub fn PyFloat_FromDouble(v: c_double) -> *mut PyObject;
    pub fn PyFloat_AsDouble(pyfloat: *mut PyObject) -> c_double;
    pub fn PyFloat_GetInfo() -> *mut PyObject;

    pub fn PyFloat_GetMax() -> c_double;
    pub fn PyFloat_GetMin() -> c_double;
    pub fn PyFloat_ClearFreeList() -> c_int;
}

pub unsafe fn PyFloat_AS_DOUBLE(pyfloat: *mut PyObject) -> c_double {
    (*(pyfloat as *mut PyFloatObject)).ob_fval
}

