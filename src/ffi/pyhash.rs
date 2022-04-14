#[cfg(not(Py_LIMITED_API))]
use crate::ffi::pyport::{Py_hash_t, Py_ssize_t};
#[cfg(not(Py_LIMITED_API))]
use std::os::raw::{c_char, c_void};

use std::os::raw::{c_int, c_ulong};

extern "C" {
    // skipped non-limited _Py_HashDouble
    // skipped non-limited _Py_HashPointer
    // skipped non-limited _Py_HashPointerRaw

    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    pub fn _Py_HashBytes(src: *const c_void, len: Py_ssize_t) -> Py_hash_t;
}

pub const _PyHASH_MULTIPLIER: c_ulong = 1_000_003;

// skipped _PyHASH_BITS

// skipped non-limited _Py_HashSecret_t

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyHash_FuncDef {
    pub hash: Option<extern "C" fn(arg1: *const c_void, arg2: Py_ssize_t) -> Py_hash_t>,
    pub name: *const c_char,
    pub hash_bits: c_int,
    pub seed_bits: c_int,
}

#[cfg(not(Py_LIMITED_API))]
impl Default for PyHash_FuncDef {
    #[inline]
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

extern "C" {
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    pub fn PyHash_GetFuncDef() -> *mut PyHash_FuncDef;
}

// skipped Py_HASH_CUTOFF

pub const Py_HASH_EXTERNAL: c_int = 0;
pub const Py_HASH_SIPHASH24: c_int = 1;
pub const Py_HASH_FNV: c_int = 2;

// skipped Py_HASH_ALGORITHM
