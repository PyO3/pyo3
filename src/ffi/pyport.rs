#[cfg(not(feature = "uselibc"))]
pub type Py_uintptr_t = usize;
#[cfg(feature = "uselibc")]
pub type Py_uintptr_t = ::libc::uintptr_t;

#[cfg(not(feature = "uselibc"))]
pub type Py_intptr_t = isize;
#[cfg(feature = "uselibc")]
pub type Py_intptr_t = ::libc::intptr_t;

#[cfg(not(feature = "uselibc"))]
pub type Py_ssize_t = isize;
#[cfg(feature = "uselibc")]
pub type Py_ssize_t = ::libc::ssize_t;

#[cfg(not(feature = "uselibc"))]
pub type Py_hash_t = isize;
#[cfg(feature = "uselibc")]
pub type Py_hash_t = Py_ssize_t;

#[cfg(not(feature = "uselibc"))]
pub type Py_uhash_t = usize;
#[cfg(feature = "uselibc")]
pub type Py_uhash_t = ::libc::size_t;

pub const PY_SSIZE_T_MIN: Py_ssize_t = std::isize::MIN as Py_ssize_t;
pub const PY_SSIZE_T_MAX: Py_ssize_t = std::isize::MAX as Py_ssize_t;
