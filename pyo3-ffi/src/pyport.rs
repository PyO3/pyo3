pub const INT_MAX: std::os::raw::c_int = libc::INT_MAX;

pub type PY_UINT32_T = u32;
pub type PY_UINT64_T = u64;

pub type PY_INT32_T = i32;
pub type PY_INT64_T = i64;

pub type Py_uintptr_t = ::libc::uintptr_t;
pub type Py_intptr_t = ::libc::intptr_t;
pub type Py_ssize_t = ::libc::ssize_t;

pub type Py_hash_t = Py_ssize_t;
pub type Py_uhash_t = ::libc::size_t;

pub const PY_SSIZE_T_MIN: Py_ssize_t = isize::MIN as Py_ssize_t;
pub const PY_SSIZE_T_MAX: Py_ssize_t = isize::MAX as Py_ssize_t;

#[cfg(target_endian = "big")]
pub const PY_BIG_ENDIAN: usize = 1;
#[cfg(target_endian = "big")]
pub const PY_LITTLE_ENDIAN: usize = 0;

#[cfg(target_endian = "little")]
pub const PY_BIG_ENDIAN: usize = 0;
#[cfg(target_endian = "little")]
pub const PY_LITTLE_ENDIAN: usize = 1;
