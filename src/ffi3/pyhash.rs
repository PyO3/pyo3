use std::os::raw::{c_void, c_char, c_int};
use ffi3::pyport::{Py_ssize_t, Py_hash_t};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyHash_FuncDef {
    pub hash: Option<extern "C" fn(arg1: *const c_void,
                                                  arg2: Py_ssize_t)
                                        -> Py_hash_t>,
    pub name: *const c_char,
    pub hash_bits: c_int,
    pub seed_bits: c_int,
}

impl Default for PyHash_FuncDef {
    #[inline] fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyHash_GetFuncDef() -> *mut PyHash_FuncDef;
}