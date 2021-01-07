pub type Py_uintptr_t = ::libc::uintptr_t;
pub type Py_intptr_t = ::libc::intptr_t;
pub type Py_ssize_t = ::libc::ssize_t;

pub type Py_hash_t = Py_ssize_t;
pub type Py_uhash_t = ::libc::size_t;

pub const PY_SSIZE_T_MIN: Py_ssize_t = std::isize::MIN as Py_ssize_t;
pub const PY_SSIZE_T_MAX: Py_ssize_t = std::isize::MAX as Py_ssize_t;
