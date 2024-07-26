#[cfg(not(any(PyPy, GraalPy)))]
use crate::pyport::{Py_hash_t, Py_ssize_t};
#[cfg(not(any(PyPy, GraalPy)))]
use std::os::raw::{c_char, c_int, c_void};

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyHash_FuncDef {
    pub hash: Option<extern "C" fn(arg1: *const c_void, arg2: Py_ssize_t) -> Py_hash_t>,
    pub name: *const c_char,
    pub hash_bits: c_int,
    pub seed_bits: c_int,
}

#[cfg(not(any(PyPy, GraalPy)))]
impl Default for PyHash_FuncDef {
    #[inline]
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

extern "C" {
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyHash_GetFuncDef() -> *mut PyHash_FuncDef;
}
