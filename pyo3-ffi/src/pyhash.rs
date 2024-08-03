use std::os::raw::c_int;

// skipped Py_HASH_CUTOFF

pub const Py_HASH_EXTERNAL: c_int = 0;
pub const Py_HASH_SIPHASH24: c_int = 1;
pub const Py_HASH_FNV: c_int = 2;

// skipped Py_HASH_ALGORITHM
