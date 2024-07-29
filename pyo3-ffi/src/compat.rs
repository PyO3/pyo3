//! C API Compatibility Shims
//!
//! Some CPython C API functions added in recent versions of Python are
//! inherently safer to use than older C API constructs. This module
//! exposes functions available on all Python versions that wrap the
//! old C API on old Python versions and wrap the function directly
//! on newer Python versions.

// Unless otherwise noted, the compatibility shims are adapted from
// the pythoncapi-compat project: https://github.com/python/pythoncapi-compat

#[cfg(not(Py_3_13))]
use crate::object::PyObject;
#[cfg(not(Py_3_13))]
use std::os::raw::c_int;

#[cfg_attr(docsrs, doc(cfg(Py_3_13)))]
#[cfg(Py_3_13)]
pub use crate::dictobject::PyDict_GetItemRef;

#[cfg_attr(docsrs, doc(cfg(not(Py_3_13))))]
#[cfg(not(Py_3_13))]
pub unsafe fn PyDict_GetItemRef(
    dp: *mut PyObject,
    key: *mut PyObject,
    result: *mut *mut PyObject,
) -> c_int {
    {
        use crate::dictobject::PyDict_GetItemWithError;
        use crate::object::_Py_NewRef;
        use crate::pyerrors::PyErr_Occurred;

        let item: *mut PyObject = PyDict_GetItemWithError(dp, key);
        if !item.is_null() {
            *result = _Py_NewRef(item);
            return 1; // found
        }
        *result = std::ptr::null_mut();
        if PyErr_Occurred().is_null() {
            return 0; // not found
        }
        -1
    }
}
