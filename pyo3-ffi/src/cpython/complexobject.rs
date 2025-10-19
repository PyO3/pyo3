use crate::PyObject;
use std::ffi::c_double;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Py_complex {
    pub real: c_double,
    pub imag: c_double,
}

// skipped private function _Py_c_sum
// skipped private function _Py_c_diff
// skipped private function _Py_c_neg
// skipped private function _Py_c_prod
// skipped private function _Py_c_quot
// skipped private function _Py_c_pow
// skipped private function _Py_c_abs

#[repr(C)]
pub struct PyComplexObject {
    pub ob_base: PyObject,
    pub cval: Py_complex,
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyComplex_FromCComplex")]
    pub fn PyComplex_FromCComplex(v: Py_complex) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyComplex_AsCComplex")]
    pub fn PyComplex_AsCComplex(op: *mut PyObject) -> Py_complex;
}
