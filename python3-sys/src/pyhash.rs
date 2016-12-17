use libc::{c_void, c_char, c_int};
use pyport::{Py_ssize_t, Py_hash_t};

#[repr(C)]
#[derive(Copy)]
pub struct PyHash_FuncDef {
    pub hash: Option<extern "C" fn(arg1: *const c_void,
                                                  arg2: Py_ssize_t)
                                        -> Py_hash_t>,
    pub name: *const c_char,
    pub hash_bits: c_int,
    pub seed_bits: c_int,
}
impl Clone for PyHash_FuncDef {
    #[inline] fn clone(&self) -> Self { *self }
}
impl Default for PyHash_FuncDef {
    #[inline] fn default() -> Self { unsafe { ::core::mem::zeroed() } }
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyHash_GetFuncDef() -> *mut PyHash_FuncDef;
}

