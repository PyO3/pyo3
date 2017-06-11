use std::os::raw::{c_double, c_int};
use ffi2::pyport::Py_ssize_t;
use ffi2::object::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Py_complex {
    pub real: c_double,
    pub imag: c_double
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn _Py_c_sum(left: Py_complex, right: Py_complex) -> Py_complex;
    pub fn _Py_c_diff(left: Py_complex, right: Py_complex) -> Py_complex;
    pub fn _Py_c_neg(complex: Py_complex) -> Py_complex;
    pub fn _Py_c_prod(left: Py_complex, right: Py_complex) -> Py_complex;
    pub fn _Py_c_quot(dividend: Py_complex, divisor: Py_complex) -> Py_complex;
    pub fn _Py_c_pow(num: Py_complex, exp: Py_complex) -> Py_complex;
    pub fn _Py_c_abs(arg: Py_complex) -> c_double;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyComplexObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub cval: Py_complex
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyComplex_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyComplex_Check(op : *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyComplex_Type)
}

#[inline(always)]
pub unsafe fn PyComplex_CheckExact(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyComplex_Type;
    (Py_TYPE(op) == u) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyComplex_FromCComplex(v: Py_complex) -> *mut PyObject;
    pub fn PyComplex_FromDoubles(real: c_double,
                                 imag: c_double) -> *mut PyObject;
    pub fn PyComplex_RealAsDouble(op: *mut PyObject) -> c_double;
    pub fn PyComplex_ImagAsDouble(op: *mut PyObject) -> c_double;
    pub fn PyComplex_AsCComplex(op: *mut PyObject) -> Py_complex;
    

    //fn _PyComplex_FormatAdvanced(obj: *mut PyObject,
    //                                 format_spec: *mut c_char,
    //                                 format_spec_len: Py_ssize_t)
    // -> *mut PyObject;
}

