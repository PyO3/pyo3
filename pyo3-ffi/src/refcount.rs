use crate::pyport::Py_ssize_t;
use crate::PyObject;
#[cfg(any(Py_3_12, all(py_sys_config = "Py_REF_DEBUG", not(Py_LIMITED_API))))]
use std::ffi::c_int;
#[cfg(all(Py_3_14, any(not(Py_GIL_DISABLED), target_pointer_width = "32")))]
use std::ffi::c_long;
#[cfg(any(Py_GIL_DISABLED, all(Py_3_12, not(Py_3_14))))]
use std::ffi::c_uint;
#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
use std::ffi::c_ulong;

#[cfg(all(Py_3_14, not(Py_3_15)))]
const _Py_STATICALLY_ALLOCATED_FLAG: c_int = 1 << 7;
#[cfg(Py_3_15)]
pub(crate) const _Py_STATICALLY_ALLOCATED_FLAG: c_int = 1 << 2;

#[cfg(all(Py_3_12, not(Py_3_14)))]
const _Py_IMMORTAL_REFCNT: Py_ssize_t = {
    if cfg!(target_pointer_width = "64") {
        c_uint::MAX as Py_ssize_t
    } else {
        (c_uint::MAX >> 2) as Py_ssize_t
    }
};

#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
const _Py_IMMORTAL_INITIAL_REFCNT: Py_ssize_t = {
    if cfg!(target_pointer_width = "64") {
        ((3 as c_ulong) << (30 as c_ulong)) as Py_ssize_t
    } else {
        ((5 as c_long) << (28 as c_long)) as Py_ssize_t
    }
};

#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
const _Py_STATIC_IMMORTAL_INITIAL_REFCNT: Py_ssize_t = {
    if cfg!(target_pointer_width = "64") {
        _Py_IMMORTAL_INITIAL_REFCNT
            | ((_Py_STATICALLY_ALLOCATED_FLAG as Py_ssize_t) << (32 as Py_ssize_t))
    } else {
        ((7 as c_long) << (28 as c_long)) as Py_ssize_t
    }
};

#[cfg(all(Py_3_14, target_pointer_width = "32"))]
const _Py_IMMORTAL_MINIMUM_REFCNT: Py_ssize_t = ((1 as c_long) << (30 as c_long)) as Py_ssize_t;

#[cfg(all(Py_3_14, target_pointer_width = "32"))]
const _Py_STATIC_IMMORTAL_MINIMUM_REFCNT: Py_ssize_t =
    ((6 as c_long) << (28 as c_long)) as Py_ssize_t;

#[cfg(all(Py_3_14, Py_GIL_DISABLED))]
const _Py_IMMORTAL_INITIAL_REFCNT: Py_ssize_t = c_uint::MAX as Py_ssize_t;

#[cfg(Py_GIL_DISABLED)]
pub(crate) const _Py_IMMORTAL_REFCNT_LOCAL: u32 = u32::MAX;

#[cfg(Py_GIL_DISABLED)]
const _Py_REF_SHARED_SHIFT: isize = 2;

pub use crate::backend::current::refcount::{
    Py_CLEAR, Py_INCREF, Py_REFCNT, Py_SETREF, Py_XDECREF, Py_XINCREF, Py_XSETREF,
};
#[cfg(not(PyRustPython))]
pub use crate::backend::current::refcount::{Py_DecRef, Py_IncRef};
#[cfg(PyRustPython)]
pub use crate::object::Py_IncRef;

#[cfg(PyRustPython)]
#[inline]
pub unsafe fn Py_DecRef(obj: *mut PyObject) {
    crate::object::Py_DECREF(obj);
}

extern_libpython! {
    #[cfg(all(Py_3_10, Py_LIMITED_API, not(PyPy)))]
    #[cfg_attr(docsrs, doc(cfg(Py_3_10)))]
    pub fn Py_NewRef(obj: *mut PyObject) -> *mut PyObject;
    #[cfg(all(Py_3_10, Py_LIMITED_API, not(PyPy)))]
    #[cfg_attr(docsrs, doc(cfg(Py_3_10)))]
    pub fn Py_XNewRef(obj: *mut PyObject) -> *mut PyObject;
}

#[cfg(all(Py_3_10, any(not(Py_LIMITED_API), PyPy)))]
#[cfg_attr(docsrs, doc(cfg(Py_3_10)))]
#[inline]
pub unsafe fn Py_NewRef(obj: *mut PyObject) -> *mut PyObject {
    Py_INCREF(obj);
    obj
}

#[cfg(all(Py_3_10, any(not(Py_LIMITED_API), PyPy)))]
#[cfg_attr(docsrs, doc(cfg(Py_3_10)))]
#[inline]
pub unsafe fn Py_XNewRef(obj: *mut PyObject) -> *mut PyObject {
    Py_XINCREF(obj);
    obj
}
