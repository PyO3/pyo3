#[cfg(not(any(Py_LIMITED_API, PyPy)))]
use crate::pyport::{Py_hash_t, Py_ssize_t};
#[cfg(not(any(Py_LIMITED_API, PyPy)))]
use std::ffi::c_void;

use std::ffi::{c_int, c_ulong};

extern "C" {
    // skipped non-limited _Py_HashDouble
    // skipped non-limited _Py_HashPointer
    // skipped non-limited _Py_HashPointerRaw

    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    pub fn _Py_HashBytes(src: *const c_void, len: Py_ssize_t) -> Py_hash_t;
}

pub const _PyHASH_MULTIPLIER: c_ulong = 1000003;

// skipped _PyHASH_BITS

// skipped non-limited _Py_HashSecret_t

// skipped Py_HASH_CUTOFF

pub const Py_HASH_EXTERNAL: c_int = 0;
pub const Py_HASH_SIPHASH24: c_int = 1;
pub const Py_HASH_FNV: c_int = 2;

// skipped Py_HASH_ALGORITHM
