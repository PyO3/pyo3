// NB libc does not define this constant on all platforms, so we hard code it
// like CPython does.
// https://github.com/python/cpython/blob/d8b9011702443bb57579f8834f3effe58e290dfc/Include/pyport.h#L372
pub const INT_MAX: std::ffi::c_int = 2147483647;

pub type PY_UINT32_T = u32;
pub type PY_UINT64_T = u64;

pub type PY_INT32_T = i32;
pub type PY_INT64_T = i64;

pub type Py_uintptr_t = ::libc::uintptr_t;
pub type Py_intptr_t = ::libc::intptr_t;
pub type Py_ssize_t = ::libc::ssize_t;

pub type Py_hash_t = Py_ssize_t;
pub type Py_uhash_t = ::libc::size_t;

pub const PY_SSIZE_T_MIN: Py_ssize_t = Py_ssize_t::MIN;
pub const PY_SSIZE_T_MAX: Py_ssize_t = Py_ssize_t::MAX;

#[cfg(target_endian = "big")]
pub const PY_BIG_ENDIAN: usize = 1;
#[cfg(target_endian = "big")]
pub const PY_LITTLE_ENDIAN: usize = 0;

#[cfg(target_endian = "little")]
pub const PY_BIG_ENDIAN: usize = 0;
#[cfg(target_endian = "little")]
pub const PY_LITTLE_ENDIAN: usize = 1;
